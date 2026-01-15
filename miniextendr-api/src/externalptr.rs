//! `ExternalPtr<T>` - A Box-like owned pointer that wraps R's EXTPTRSXP
//!
//! This provides ownership semantics similar to `Box<T>`, with the key difference
//! that cleanup is deferred to R's garbage collector via finalizers.
//!
//! This means you can hand ownership of Rust-allocated data to R and let its GC
//! decide when to drop it. The `tag` slot is a human-friendly type name, and the
//! `prot` slot stores both the type symbol and any R objects you want to keep
//! alive alongside the pointer. Neither slot is an R class attribute; if you
//! want an S3/S4 class, attach it yourself in R.
//!
//! `PartialEq`/`PartialOrd` compare the pointee values (like `Box<T>`). Use
//! `ptr_eq` when you care about pointer identity, and `as_ref()`/`as_mut()` for
//! explicit by-value comparisons.
//!
//! # Protection Strategies in miniextendr
//!
//! miniextendr provides three complementary protection mechanisms for different scenarios:
//!
//! | Strategy | Module | Lifetime | Release Order | Use Case |
//! |----------|--------|----------|---------------|----------|
//! | **PROTECT stack** | [`gc_protect`](crate::gc_protect) | Within `.Call` | LIFO (stack) | Temporary allocations |
//! | **Preserve list** | [`preserve`](crate::preserve) | Across `.Call`s | Any order | Long-lived R objects |
//! | **R ownership** | [`ExternalPtr`](struct@crate::externalptr::ExternalPtr) | Until R GCs | R decides | Rust data owned by R |
//!
//! ## When to Use ExternalPtr
//!
//! **Use `ExternalPtr` (this module) when:**
//! - You want R to own a Rust value
//! - The Rust value should be dropped when R garbage collects the pointer
//! - You're exposing Rust structs to R code
//!
//! **Use [`gc_protect`](crate::gc_protect) instead when:**
//! - You're allocating temporary R objects during computation
//! - Protection is short-lived (within a single `.Call`)
//!
//! **Use [`preserve`](crate::preserve) instead when:**
//! - You need R objects (not Rust values) to survive across `.Call`s
//! - You need arbitrary-order release of protections
//!
//! ## How ExternalPtr Protection Works
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  ExternalPtr<MyStruct>::new(value)                              │
//! │  ├── Rf_protect() during construction (temporary)               │
//! │  ├── R_MakeExternalPtr() creates EXTPTRSXP                      │
//! │  ├── R_RegisterCFinalizerEx() registers cleanup callback        │
//! │  └── Rf_unprotect() after construction complete                 │
//! │                                                                 │
//! │  Return to R → R now owns the EXTPTRSXP                         │
//! │  ├── SEXP is live as long as R has references                   │
//! │  └── Rust value is accessible via ExternalPtr::wrap_sexp()      │
//! │                                                                 │
//! │  R GC runs → finalizer called → Rust Drop executes              │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Type Identification
//!
//! Type identification uses R's interned symbols (`Rf_install`). Since R interns
//! symbols, the same type name always returns the same pointer, enabling fast
//! pointer comparison for type checking.
//!
//! The `tag` slot holds a symbol (type name).
//! The `prot` slot holds a VECSXP (list) with two elements:
//!   - Index 0: SYMSXP (interned symbol) for fast pointer-based type comparison
//!   - Index 1: User-protected SEXP slot (for preventing GC of R objects)
//!
//! # ExternalPtr is Not an R Native Type
//!
//! Unlike R's native atomic types (`integer`, `double`, `character`, etc.),
//! external pointers cannot be coerced to vectors or used in R's vectorized
//! operations. This is an R limitation, not a miniextendr limitation:
//!
//! ```r
//! > matrix(new("externalptr"), 1, 1)
//! Error in `as.vector()`:
//! ! cannot coerce type 'externalptr' to vector of type 'any'
//! ```
//!
//! If you need your Rust type to participate in R's vector/matrix operations,
//! consider implementing [`IntoList`](crate::list::IntoList) (via `#[derive(IntoList)]`)
//! to convert your struct to a named R list, or use ALTREP to expose Rust
//! iterators as lazy R vectors.

use std::alloc::Layout;
use std::any::TypeId;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem::{self, ManuallyDrop, MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::ptr::{self, NonNull};

use crate::ffi::{
    R_ClearExternalPtr, R_ExternalPtrAddr, R_ExternalPtrProtected, R_ExternalPtrTag,
    R_MakeExternalPtr, R_MakeExternalPtr_unchecked, R_NilValue, R_RegisterCFinalizerEx,
    R_RegisterCFinalizerEx_unchecked, Rboolean, Rf_allocVector, Rf_allocVector_unchecked,
    Rf_install, Rf_install_unchecked, Rf_protect, Rf_protect_unchecked, Rf_unprotect,
    Rf_unprotect_unchecked, SET_VECTOR_ELT, SET_VECTOR_ELT_unchecked, SEXP, SEXPTYPE, SexpExt,
    VECTOR_ELT,
};

/// A wrapper around a raw pointer that implements [`Send`].
///
/// # Safety
///
/// This is safe to send between threads because it's just a memory address.
/// The data `T` is owned and transferred to the main thread before being accessed.
type SendablePtr<T> = crate::worker::Sendable<NonNull<T>>;

/// Create a new sendable pointer from a raw pointer.
///
/// # Safety
///
/// The pointer must be non-null.
#[inline]
unsafe fn sendable_ptr_new_unchecked<T>(ptr: *mut T) -> SendablePtr<T> {
    // SAFETY: Caller guarantees ptr is non-null
    crate::worker::Sendable(unsafe { NonNull::new_unchecked(ptr) })
}

/// Get the raw pointer, consuming the sendable wrapper.
#[inline]
fn sendable_ptr_into_ptr<T>(ptr: SendablePtr<T>) -> *mut T {
    ptr.0.as_ptr()
}

/// Index of the type SYMSXP contained in the `prot` (a `VECSXP` list)
const PROT_TYPE_ID_INDEX: isize = 0;
/// Index of user-protected objects contained in the `prot` (a `VECSXP` list)
const PROT_USER_INDEX: isize = 1;
/// Length of the `prot` list (`VECSXP`)
const PROT_VEC_LEN: isize = 2;

#[inline]
fn is_type_erased<T: 'static>() -> bool {
    TypeId::of::<T>() == TypeId::of::<()>()
}

/// Get the interned R symbol for a type's name.
///
/// R interns symbols via `Rf_install`, so the same string always returns
/// the same pointer. This enables fast pointer comparison for type checking.
///
/// # Safety
///
/// Must be called from R's main thread.
#[inline]
unsafe fn type_symbol<T: TypedExternal>() -> SEXP {
    unsafe { Rf_install(T::TYPE_NAME_CSTR.as_ptr().cast()) }
}

/// Unchecked version of [`type_symbol`] - no thread safety checks.
///
/// # Safety
///
/// Must be called from R's main thread. No debug assertions.
#[inline]
unsafe fn type_symbol_unchecked<T: TypedExternal>() -> SEXP {
    unsafe { Rf_install_unchecked(T::TYPE_NAME_CSTR.as_ptr().cast()) }
}

/// Get the namespaced type ID symbol for type checking.
///
/// Uses `TYPE_ID_CSTR` which includes the module path for uniqueness.
///
/// # Safety
///
/// Must be called from R's main thread.
#[inline]
unsafe fn type_id_symbol<T: TypedExternal>() -> SEXP {
    unsafe { Rf_install(T::TYPE_ID_CSTR.as_ptr().cast()) }
}

/// Unchecked version of [`type_id_symbol`].
///
/// # Safety
///
/// Must be called from R's main thread. No debug assertions.
#[inline]
unsafe fn type_id_symbol_unchecked<T: TypedExternal>() -> SEXP {
    unsafe { Rf_install_unchecked(T::TYPE_ID_CSTR.as_ptr().cast()) }
}

/// Get the type name from a stored symbol SEXP.
///
/// # Safety
///
/// `sym` must be a valid SYMSXP.
#[inline]
unsafe fn symbol_name(sym: SEXP) -> &'static str {
    // SYMSXP's PRINTNAME is a CHARSXP
    let printname = unsafe { crate::ffi::PRINTNAME(sym) };
    let cstr = unsafe { crate::ffi::R_CHAR(printname) };
    let len = unsafe { crate::ffi::Rf_xlength(printname) as usize };
    unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(cstr.cast(), len)) }
}

// =============================================================================
// TypedExternalPtr Trait
// =============================================================================

/// Trait for types that can be stored in an ExternalPtr.
///
/// This provides the type identification needed for runtime type checking.
/// Type identification uses R's symbol interning (`Rf_install`) for fast
/// pointer-based comparison.
///
/// # Type ID vs Type Name
///
/// - `TYPE_ID_CSTR`: Namespaced identifier used for type checking (stored in `prot[0]`).
///   Format: `"<crate_name>@<crate_version>::<module_path>::<type_name>\0"`
///
///   The crate name and version ensure:
///   - Same type from same crate+version → compatible (can share ExternalPtr)
///   - Same type name from different crates → incompatible
///   - Same type from different crate versions → incompatible
///
/// - `TYPE_NAME_CSTR`: Short display name for the R tag (shown when printing).
///   Just the type identifier for readability.
pub trait TypedExternal: 'static {
    /// The type name as a static string (for debugging and display)
    const TYPE_NAME: &'static str;

    /// The type name as a null-terminated C string (for R tag display)
    const TYPE_NAME_CSTR: &'static [u8];

    /// Namespaced type ID as a null-terminated C string (for type checking).
    ///
    /// This should include the module path to prevent cross-package collisions.
    /// Use `concat!(module_path!(), "::", stringify!(Type), "\0").as_bytes()`
    /// when implementing manually, or use `#[derive(ExternalPtr)]`.
    const TYPE_ID_CSTR: &'static [u8];
}

/// Marker trait for types that should be converted to R as ExternalPtr.
///
/// When a type implements this trait (via `#[derive(ExternalPtr)]`), it gets a
/// blanket `IntoR` implementation that wraps the value in `ExternalPtr<T>`.
///
/// This allows returning the type directly from `#[miniextendr]` functions:
///
/// ```ignore
/// #[derive(ExternalPtr)]
/// struct MyData { value: i32 }
///
/// #[miniextendr]
/// fn create_data(v: i32) -> MyData {
///     MyData { value: v }  // Automatically wrapped in ExternalPtr
/// }
/// ```
pub trait IntoExternalPtr: TypedExternal {}

/// Implement TypedExternal for a type with automatic namespacing.
///
/// This macro generates:
/// - `TYPE_NAME`: Short display name (just the type identifier)
/// - `TYPE_NAME_CSTR`: Null-terminated display name for R tag
/// - `TYPE_ID_CSTR`: Namespaced ID using crate name, version, and module path
///
/// Format: `<crate_name>@<crate_version>::<module_path>::<type_name>`
#[macro_export]
macro_rules! impl_typed_external {
    ($ty:ty) => {
        impl $crate::externalptr::TypedExternal for $ty {
            const TYPE_NAME: &'static str = stringify!($ty);
            const TYPE_NAME_CSTR: &'static [u8] = concat!(stringify!($ty), "\0").as_bytes();
            const TYPE_ID_CSTR: &'static [u8] = concat!(
                env!("CARGO_PKG_NAME"),
                "@",
                env!("CARGO_PKG_VERSION"),
                "::",
                module_path!(),
                "::",
                stringify!($ty),
                "\0"
            )
            .as_bytes();
        }
    };
}

/// Implement TypedExternal for a type with a custom tag name.
///
/// Use this when you want the R tag to display a specific name
/// (e.g., without module path).
///
/// Format: `<crate_name>@<crate_version>::<module_path>::<tag>`
#[macro_export]
macro_rules! impl_typed_external_with_tag {
    ($ty:ty, $tag:expr) => {
        impl $crate::externalptr::TypedExternal for $ty {
            const TYPE_NAME: &'static str = $tag;
            const TYPE_NAME_CSTR: &'static [u8] = concat!($tag, "\0").as_bytes();
            const TYPE_ID_CSTR: &'static [u8] = concat!(
                env!("CARGO_PKG_NAME"),
                "@",
                env!("CARGO_PKG_VERSION"),
                "::",
                module_path!(),
                "::",
                $tag,
                "\0"
            )
            .as_bytes();
        }
    };
}

impl TypedExternal for () {
    const TYPE_NAME: &'static str = "()";
    const TYPE_NAME_CSTR: &'static [u8] = b"()\0";
    // Unit type is special - same ID as name since it's only used for type-erased ptrs
    const TYPE_ID_CSTR: &'static [u8] = b"()\0";
}

// =============================================================================
// ExternalPtr<T>
// =============================================================================

/// An owned pointer stored in R's external pointer SEXP.
///
/// This is conceptually similar to `Box<T>`, but with the following differences:
/// - Memory is freed by R's GC via a registered finalizer (non-deterministic)
/// - The underlying SEXP is Copy, so aliasing must be manually prevented
/// - Type checking happens at runtime via R symbol comparison in the prot slot
///
/// # Thread Safety
///
/// `ExternalPtr` is `Send` to allow returning from worker thread functions.
/// However, **concurrent access is not allowed** - R's runtime is single-threaded.
/// All R API calls are serialized through the main thread via `with_r_thread`.
///
/// # Safety
///
/// The ExternalPtr assumes exclusive ownership of the underlying data.
/// Cloning the raw SEXP without proper handling will lead to double-free.
#[repr(C)]
pub struct ExternalPtr<T: TypedExternal> {
    sexp: SEXP,
    _marker: PhantomData<T>,
}

// SAFETY: ExternalPtr can be sent between threads because:
// 1. All R API operations are serialized through the main thread via with_r_thread
// 2. The worker thread is blocked while the main thread processes R calls
// 3. There is no concurrent access - only sequential hand-off between threads
unsafe impl<T: TypedExternal + Send> Send for ExternalPtr<T> {}

impl<T: TypedExternal> ExternalPtr<T> {
    /// Allocates memory on the heap and places `x` into it.
    ///
    /// This function can be called from any thread:
    /// - If called from R's main thread, creates the ExternalPtr directly
    /// - If called from the worker thread (during `run_on_worker`), automatically
    ///   sends the R API calls to the main thread via [`with_r_thread`]
    ///
    /// # Panics
    ///
    /// Panics if called from a non-main thread outside of a `run_on_worker` context.
    ///
    /// Equivalent to `Box::new`.
    ///
    /// [`with_r_thread`]: crate::worker::with_r_thread
    #[inline]
    pub fn new(x: T) -> Self {
        // Box the value first (this is thread-safe)
        let ptr = Box::into_raw(Box::new(x));
        // Wrap in SendablePtr so it can be sent across thread boundary
        // SAFETY: Box::into_raw never returns null
        let sendable_ptr = unsafe { sendable_ptr_new_unchecked(ptr) };

        // Use with_r_thread to run R API calls on main thread
        let sexp = crate::worker::with_r_thread(move || {
            // This runs on main thread - unwrap the pointer
            let ptr: *mut T = sendable_ptr_into_ptr(sendable_ptr);
            // Use _unchecked since with_r_thread guarantees we're on main thread
            unsafe { Self::create_extptr_sexp_unchecked(ptr) }
        });

        Self {
            sexp,
            _marker: PhantomData,
        }
    }

    /// Allocates memory on the heap and places `x` into it, without thread checks.
    ///
    /// This version skips the debug thread-safety assertions in R API calls.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread. Calling from another thread
    /// is undefined behavior (R APIs are not thread-safe).
    ///
    /// Use this in contexts where you know you're on main thread:
    /// - ALTREP callbacks
    /// - Inside `#[miniextendr(unsafe(main_thread))]` functions
    /// - Inside `extern "C-unwind"` functions called directly by R
    #[inline]
    pub unsafe fn new_unchecked(x: T) -> Self {
        let ptr = Box::into_raw(Box::new(x));
        let sexp = unsafe { Self::create_extptr_sexp_unchecked(ptr) };
        Self {
            sexp,
            _marker: PhantomData,
        }
    }

    /// Create an EXTPTRSXP from a raw pointer. Must be called from main thread.
    ///
    /// This is the internal function that actually calls R APIs.
    #[inline]
    unsafe fn create_extptr_sexp(ptr: *mut T) -> SEXP {
        debug_assert!(!ptr.is_null(), "create_extptr_sexp received null pointer");

        // Create two symbols:
        // - type_sym (TYPE_NAME_CSTR): short name for display in R
        // - type_id_sym (TYPE_ID_CSTR): namespaced ID for type checking
        let type_sym = unsafe { type_symbol::<T>() };
        let type_id_sym = unsafe { type_id_symbol::<T>() };

        // Create the prot VECSXP: [type_id_symbol, user_protected]
        let prot = unsafe { Rf_allocVector(SEXPTYPE::VECSXP, PROT_VEC_LEN) };
        unsafe { Rf_protect(prot) };

        // Store namespaced type ID in slot 0 for type checking
        unsafe { SET_VECTOR_ELT(prot, PROT_TYPE_ID_INDEX, type_id_sym) };
        // Slot 1 (user protected) starts as R_NilValue (already default)

        // Create the external pointer with short display tag and prot
        let sexp = unsafe { R_MakeExternalPtr(ptr.cast(), type_sym, prot) };
        unsafe { Rf_protect(sexp) };

        // Register the C finalizer that will call drop
        unsafe { R_RegisterCFinalizerEx(sexp, Some(release_raw::<T>), Rboolean::TRUE) };

        unsafe { Rf_unprotect(2) };

        sexp
    }

    /// Create an EXTPTRSXP from a raw pointer without thread safety checks.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread. No debug assertions for thread safety.
    #[inline]
    unsafe fn create_extptr_sexp_unchecked(ptr: *mut T) -> SEXP {
        debug_assert!(
            !ptr.is_null(),
            "create_extptr_sexp_unchecked received null pointer"
        );

        // Create two symbols:
        // - type_sym (TYPE_NAME_CSTR): short name for display in R
        // - type_id_sym (TYPE_ID_CSTR): namespaced ID for type checking
        let type_sym = unsafe { type_symbol_unchecked::<T>() };
        let type_id_sym = unsafe { type_id_symbol_unchecked::<T>() };

        // Create the prot VECSXP: [type_id_symbol, user_protected]
        let prot = unsafe { Rf_allocVector_unchecked(SEXPTYPE::VECSXP, PROT_VEC_LEN) };
        unsafe { Rf_protect_unchecked(prot) };

        // Store namespaced type ID in slot 0 for type checking
        unsafe { SET_VECTOR_ELT_unchecked(prot, PROT_TYPE_ID_INDEX, type_id_sym) };
        // Slot 1 (user protected) starts as R_NilValue (already default)

        // Create the external pointer with short display tag and prot
        let sexp = unsafe { R_MakeExternalPtr_unchecked(ptr.cast(), type_sym, prot) };
        unsafe { Rf_protect_unchecked(sexp) };

        // Register the C finalizer that will call drop
        unsafe { R_RegisterCFinalizerEx_unchecked(sexp, Some(release_raw::<T>), Rboolean::TRUE) };

        unsafe { Rf_unprotect_unchecked(2) };

        sexp
    }

    /// Constructs a new `ExternalPtr` with uninitialized contents.
    ///
    /// Equivalent to `Box::new_uninit`.
    #[inline]
    pub fn new_uninit() -> ExternalPtr<MaybeUninit<T>>
    where
        MaybeUninit<T>: TypedExternal,
    {
        ExternalPtr::new(MaybeUninit::uninit())
    }

    /// Constructs a new `ExternalPtr` with zeroed contents.
    ///
    /// Equivalent to `Box::new_zeroed`.
    #[inline]
    pub fn new_zeroed() -> ExternalPtr<MaybeUninit<T>>
    where
        MaybeUninit<T>: TypedExternal,
    {
        ExternalPtr::new(MaybeUninit::zeroed())
    }

    /// Constructs an ExternalPtr from a raw pointer.
    ///
    /// # Safety
    ///
    /// - `raw` must have been allocated via `Box::into_raw` or equivalent
    /// - `raw` must not be null
    /// - Caller transfers ownership to the ExternalPtr
    /// - Must be called from R's main thread
    ///
    /// Equivalent to `Box::from_raw`.
    #[inline]
    pub unsafe fn from_raw(raw: *mut T) -> Self {
        let sexp = unsafe { Self::create_extptr_sexp(raw) };
        Self {
            sexp,
            _marker: PhantomData,
        }
    }

    /// Constructs an ExternalPtr from a raw pointer, without thread checks.
    ///
    /// # Safety
    ///
    /// - `raw` must have been allocated via `Box::into_raw` or equivalent
    /// - `raw` must not be null
    /// - Caller transfers ownership to the ExternalPtr
    /// - Must be called from R's main thread (no debug assertions)
    #[inline]
    pub unsafe fn from_raw_unchecked(raw: *mut T) -> Self {
        let sexp = unsafe { Self::create_extptr_sexp_unchecked(raw) };
        Self {
            sexp,
            _marker: PhantomData,
        }
    }

    /// Consumes the ExternalPtr, returning a raw pointer.
    ///
    /// The caller is responsible for the memory, and the finalizer is
    /// effectively orphaned (will do nothing since we clear the pointer).
    ///
    /// Equivalent to `Box::into_raw`.
    #[inline]
    pub fn into_raw(this: Self) -> *mut T {
        let ptr = unsafe { R_ExternalPtrAddr(this.sexp).cast() };

        // Clear the external pointer so the finalizer becomes a no-op
        unsafe { R_ClearExternalPtr(this.sexp) };

        // Don't run our Drop
        mem::forget(this);

        ptr
    }

    /// Consumes the ExternalPtr, returning a `NonNull` pointer.
    ///
    /// Equivalent to `Box::into_non_null`.
    #[inline]
    pub fn into_non_null(this: Self) -> NonNull<T> {
        unsafe { NonNull::new_unchecked(Self::into_raw(this)) }
    }

    /// Consumes and leaks the ExternalPtr, returning a mutable reference.
    ///
    /// The memory will never be freed (from Rust's perspective; R's GC
    /// finalizer is neutralized).
    ///
    /// Equivalent to `Box::leak`.
    #[inline]
    pub fn leak<'a>(this: Self) -> &'a mut T
    where
        T: 'a,
    {
        unsafe { &mut *Self::into_raw(this) }
    }

    /// Consumes the ExternalPtr, returning the wrapped value.
    ///
    /// This moves the value out of the ExternalPtr and deallocates
    /// the underlying memory. The R finalizer becomes a no-op.
    ///
    /// Equivalent to `*boxed` (deref move) or `Box::into_inner`.
    #[inline]
    pub fn into_inner(this: Self) -> T {
        // Get the raw pointer and prevent finalizer from running
        let ptr = Self::into_raw(this);

        // Read the value out
        let value = unsafe { ptr::read(ptr) };

        // Deallocate the memory (Box handles the layout)
        unsafe {
            let layout = Layout::new::<T>();
            if layout.size() > 0 {
                std::alloc::dealloc(ptr.cast(), layout);
            }
        }

        value
    }

    // =========================================================================
    // Pin support (Box-equivalent)
    // =========================================================================

    /// Constructs a new `Pin<ExternalPtr<T>>`.
    ///
    /// Equivalent to `Box::pin`.
    ///
    /// # Note
    ///
    /// Unlike `Box::pin`, this requires `T: Unpin` because `ExternalPtr`
    /// implements `DerefMut` unconditionally. For `!Unpin` types, use
    /// `ExternalPtr::new` and manage pinning guarantees manually.
    #[inline]
    pub fn pin(x: T) -> Pin<Self>
    where
        T: Unpin,
    {
        // SAFETY: T: Unpin, so pinning is always safe
        Pin::new(Self::new(x))
    }

    /// Constructs a new `Pin<ExternalPtr<T>>` without requiring `Unpin`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the pinning invariants are upheld:
    /// - The data will not be moved out of the `ExternalPtr`
    /// - The data will not be accessed mutably in ways that would move it
    ///
    /// Since `ExternalPtr` implements `DerefMut`, using this with `!Unpin`
    /// types requires careful handling to avoid moving the inner value.
    #[inline]
    pub fn pin_unchecked(x: T) -> Pin<Self> {
        unsafe { Pin::new_unchecked(Self::new(x)) }
    }

    /// Converts a `ExternalPtr<T>` into a `Pin<ExternalPtr<T>>`.
    ///
    /// Equivalent to `Box::into_pin`.
    #[inline]
    pub fn into_pin(this: Self) -> Pin<Self>
    where
        T: Unpin,
    {
        // SAFETY: T: Unpin, so it's always safe to pin
        Pin::new(this)
    }

    // =========================================================================
    // Accessors
    // =========================================================================

    /// Returns a reference to the underlying value.
    #[inline]
    pub fn as_ref(&self) -> Option<&T> {
        unsafe { R_ExternalPtrAddr(self.sexp).cast::<T>().as_ref() }
    }

    /// Returns a mutable reference to the underlying value.
    #[inline]
    pub fn as_mut(&mut self) -> Option<&mut T> {
        unsafe { R_ExternalPtrAddr(self.sexp).cast::<T>().as_mut() }
    }

    /// Returns the raw pointer without consuming the ExternalPtr.
    #[inline]
    pub fn as_ptr(&self) -> *const T {
        unsafe { R_ExternalPtrAddr(self.sexp).cast() }
    }

    /// Returns the raw mutable pointer without consuming the ExternalPtr.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        unsafe { R_ExternalPtrAddr(self.sexp).cast() }
    }

    /// Checks whether two `ExternalPtr`s refer to the same allocation (pointer identity).
    ///
    /// This ignores the pointee values. Use this when you need alias detection;
    /// prefer `PartialEq`/`PartialOrd` or `as_ref()` for value comparisons.
    #[inline]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        let a = unsafe { R_ExternalPtrAddr(this.sexp).cast::<()>().cast_const() };
        let b = unsafe { R_ExternalPtrAddr(other.sexp).cast::<()>().cast_const() };
        ptr::eq(a, b)
    }

    // =========================================================================
    // R-specific accessors
    // =========================================================================

    /// Returns the underlying SEXP.
    ///
    /// # Warning
    ///
    /// The returned SEXP must not be duplicated or the finalizer will double-free.
    #[inline]
    pub fn as_sexp(&self) -> SEXP {
        self.sexp
    }

    /// Returns the tag SEXP (type identifier symbol).
    #[inline]
    pub fn tag(&self) -> SEXP {
        unsafe { R_ExternalPtrTag(self.sexp) }
    }

    /// Returns the tag SEXP (unchecked version).
    ///
    /// Skips thread safety checks for performance-critical paths.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. Only use in ALTREP callbacks
    /// or other contexts where you're certain you're on the main thread.
    #[inline]
    pub unsafe fn tag_unchecked(&self) -> SEXP {
        unsafe { crate::ffi::R_ExternalPtrTag_unchecked(self.sexp) }
    }

    /// Returns the protected SEXP slot (user-protected objects).
    ///
    /// This returns the user-protected object stored in the prot VECSXP,
    /// not the VECSXP itself.
    #[inline]
    pub fn protected(&self) -> SEXP {
        unsafe {
            let prot = R_ExternalPtrProtected(self.sexp);
            if prot.is_null_or_nil() {
                return R_NilValue;
            }
            if prot.type_of() != SEXPTYPE::VECSXP || prot.len() < PROT_VEC_LEN as usize {
                return R_NilValue;
            }
            VECTOR_ELT(prot, PROT_USER_INDEX)
        }
    }

    /// Returns the protected SEXP slot (unchecked version).
    ///
    /// Skips thread safety checks for performance-critical paths.
    ///
    /// # Safety
    ///
    /// Must be called from the R main thread. Only use in ALTREP callbacks
    /// or other contexts where you're certain you're on the main thread.
    #[inline]
    pub unsafe fn protected_unchecked(&self) -> SEXP {
        use crate::ffi::{R_ExternalPtrProtected_unchecked, VECTOR_ELT_unchecked};

        unsafe {
            let prot = R_ExternalPtrProtected_unchecked(self.sexp);
            if prot.is_null_or_nil() {
                return R_NilValue;
            }
            if prot.type_of() != SEXPTYPE::VECSXP || prot.len() < PROT_VEC_LEN as usize {
                return R_NilValue;
            }
            VECTOR_ELT_unchecked(prot, PROT_USER_INDEX)
        }
    }

    /// Sets the user-protected SEXP slot.
    ///
    /// Use this to prevent R objects from being GC'd while this ExternalPtr exists.
    /// The type ID stored in prot slot 0 is preserved.
    ///
    /// Returns `false` if the prot structure is malformed (should not happen
    /// for ExternalPtrs created by this library).
    ///
    /// # Safety
    ///
    /// - `user_prot` must be a valid SEXP or R_NilValue
    /// - Must be called from the R main thread
    #[inline]
    pub unsafe fn set_protected(&self, user_prot: SEXP) -> bool {
        unsafe {
            let prot = R_ExternalPtrProtected(self.sexp);
            if prot.is_null_or_nil() {
                debug_assert!(false, "ExternalPtr prot slot is null or R_NilValue");
                return false;
            }
            if prot.type_of() != SEXPTYPE::VECSXP || prot.len() < PROT_VEC_LEN as usize {
                debug_assert!(
                    false,
                    "ExternalPtr prot slot is not a VECSXP of expected length"
                );
                return false;
            }
            SET_VECTOR_ELT(prot, PROT_USER_INDEX, user_prot);
            true
        }
    }

    /// Returns the raw prot VECSXP (contains both type ID and user protected).
    ///
    /// Prefer using `protected()` for user data and `stored_type_id()` for type info.
    #[inline]
    pub fn prot_raw(&self) -> SEXP {
        unsafe { R_ExternalPtrProtected(self.sexp) }
    }

    /// Checks if the internal pointer is null (already finalized or cleared).
    #[inline]
    pub fn is_null(&self) -> bool {
        unsafe { R_ExternalPtrAddr(self.sexp).is_null() }
    }

    // =========================================================================
    // Type checking
    // =========================================================================

    /// Attempt to wrap a SEXP as an ExternalPtr with type checking.
    ///
    /// Returns `None` if:
    /// - The internal pointer is null
    /// - The `prot` slot doesn't contain a valid VECSXP with type symbol
    /// - The type symbol doesn't match T's type
    ///
    /// This is a low-level method. For automatic conversions in `#[miniextendr]`
    /// functions, use the [`TryFromSexp`] trait which requires `T: Send`.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid EXTPTRSXP
    /// - The caller must ensure no other ExternalPtr owns this SEXP
    ///
    /// [`TryFromSexp`]: crate::TryFromSexp
    pub unsafe fn wrap_sexp(sexp: SEXP) -> Option<Self> {
        // Check if pointer is null
        let ptr = unsafe { R_ExternalPtrAddr(sexp) };
        if ptr.is_null() {
            return None;
        }

        // Extract prot VECSXP
        let prot = unsafe { R_ExternalPtrProtected(sexp) };
        if prot.is_null_or_nil() {
            return None;
        }
        if prot.type_of() != SEXPTYPE::VECSXP || prot.len() < PROT_VEC_LEN as usize {
            return None;
        }

        // Extract type symbol from slot 0
        let stored_sym = unsafe { VECTOR_ELT(prot, PROT_TYPE_ID_INDEX) };
        if stored_sym.type_of() != SEXPTYPE::SYMSXP {
            return None;
        }

        // Compare symbols by pointer (R interns symbols)
        let expected_sym = unsafe { type_id_symbol::<T>() };
        if !is_type_erased::<T>() && !std::ptr::eq(stored_sym.0, expected_sym.0) {
            return None;
        }

        Some(Self {
            sexp,
            _marker: PhantomData,
        })
    }

    /// Attempt to wrap a SEXP as an ExternalPtr (unchecked version).
    ///
    /// Skips thread safety checks for performance-critical paths like ALTREP callbacks.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid EXTPTRSXP
    /// - The caller must ensure exclusive ownership
    /// - Must be called from the R main thread (guaranteed in ALTREP callbacks)
    pub unsafe fn wrap_sexp_unchecked(sexp: SEXP) -> Option<Self> {
        use crate::ffi::{
            R_ExternalPtrAddr_unchecked, R_ExternalPtrProtected_unchecked, VECTOR_ELT_unchecked,
        };

        // Check if pointer is null
        let ptr = unsafe { R_ExternalPtrAddr_unchecked(sexp) };
        if ptr.is_null() {
            return None;
        }

        // Extract prot VECSXP
        let prot = unsafe { R_ExternalPtrProtected_unchecked(sexp) };
        if prot.is_null_or_nil() {
            return None;
        }
        if prot.type_of() != SEXPTYPE::VECSXP || prot.len() < PROT_VEC_LEN as usize {
            return None;
        }

        // Extract type symbol from slot 0
        let stored_sym = unsafe { VECTOR_ELT_unchecked(prot, PROT_TYPE_ID_INDEX) };
        if stored_sym.type_of() != SEXPTYPE::SYMSXP {
            return None;
        }

        // Compare symbols by pointer (R interns symbols)
        let expected_sym = unsafe { type_id_symbol::<T>() };
        if !is_type_erased::<T>() && !std::ptr::eq(stored_sym.0, expected_sym.0) {
            return None;
        }

        Some(Self {
            sexp,
            _marker: PhantomData,
        })
    }

    /// Attempt to wrap a SEXP as an ExternalPtr, returning an error with type info on mismatch.
    ///
    /// This is used by the [`TryFromSexp`] trait implementation.
    ///
    /// # Safety
    ///
    /// Same as [`wrap_sexp`](Self::wrap_sexp).
    ///
    /// [`TryFromSexp`]: crate::TryFromSexp
    pub unsafe fn wrap_sexp_with_error(sexp: SEXP) -> Result<Self, TypeMismatchError> {
        // Check if pointer is null
        let ptr = unsafe { R_ExternalPtrAddr(sexp) };
        if ptr.is_null() {
            return Err(TypeMismatchError::NullPointer);
        }

        // Extract prot VECSXP
        let prot = unsafe { R_ExternalPtrProtected(sexp) };
        if prot.is_null_or_nil() {
            return Err(TypeMismatchError::InvalidTypeId);
        }
        if prot.type_of() != SEXPTYPE::VECSXP || prot.len() < PROT_VEC_LEN as usize {
            return Err(TypeMismatchError::InvalidTypeId);
        }

        // Extract type symbol from slot 0
        let stored_sym = unsafe { VECTOR_ELT(prot, PROT_TYPE_ID_INDEX) };
        if stored_sym.type_of() != SEXPTYPE::SYMSXP {
            return Err(TypeMismatchError::InvalidTypeId);
        }

        // Compare symbols by pointer (R interns symbols)
        let expected_sym = unsafe { type_id_symbol::<T>() };
        if !is_type_erased::<T>() && !std::ptr::eq(stored_sym.0, expected_sym.0) {
            return Err(TypeMismatchError::Mismatch {
                expected: T::TYPE_NAME,
                found: unsafe { symbol_name(stored_sym) },
            });
        }

        Ok(Self {
            sexp,
            _marker: PhantomData,
        })
    }

    /// Create an ExternalPtr from an SEXP without type checking.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid EXTPTRSXP containing a `*mut T`
    /// - The caller must ensure exclusive ownership
    #[inline]
    pub unsafe fn from_sexp_unchecked(sexp: SEXP) -> Self {
        Self {
            sexp,
            _marker: PhantomData,
        }
    }

    // =========================================================================
    // Downcast support
    // =========================================================================

    /// Returns the type name for type T.
    #[inline]
    pub fn type_name() -> &'static str {
        T::TYPE_NAME
    }

    /// Returns the type name stored in this ExternalPtr's prot slot.
    ///
    /// Returns `None` if the prot slot doesn't contain a valid type symbol.
    #[inline]
    pub fn stored_type_name(&self) -> Option<&'static str> {
        unsafe {
            let prot = R_ExternalPtrProtected(self.sexp);
            if prot.is_null_or_nil() {
                return None;
            }
            if prot.type_of() != SEXPTYPE::VECSXP || prot.len() < PROT_VEC_LEN as usize {
                return None;
            }
            let stored_sym = VECTOR_ELT(prot, PROT_TYPE_ID_INDEX);
            if stored_sym.type_of() != SEXPTYPE::SYMSXP {
                return None;
            }
            Some(symbol_name(stored_sym))
        }
    }
}

impl ExternalPtr<()> {
    /// Create a type-erased ExternalPtr from an EXTPTRSXP without checking the stored type.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid EXTPTRSXP
    /// - Caller must ensure exclusive ownership semantics are upheld
    #[inline]
    pub unsafe fn from_sexp(sexp: SEXP) -> Self {
        debug_assert!(sexp.type_of() == SEXPTYPE::EXTPTRSXP);
        unsafe { Self::from_sexp_unchecked(sexp) }
    }

    /// Check whether the stored type symbol matches `T`.
    #[inline]
    pub fn is<T: TypedExternal>(&self) -> bool {
        if self.is_null() {
            return false;
        }
        unsafe {
            let prot = R_ExternalPtrProtected(self.sexp);
            if prot.is_null_or_nil() || prot.type_of() != SEXPTYPE::VECSXP {
                return false;
            }
            let stored_sym = VECTOR_ELT(prot, PROT_TYPE_ID_INDEX);
            if stored_sym.type_of() != SEXPTYPE::SYMSXP {
                return false;
            }
            // Must use type_id_symbol (namespaced) to match what new() stores
            let expected_sym = type_id_symbol::<T>();
            std::ptr::eq(stored_sym.0, expected_sym.0)
        }
    }

    /// Downcast to an immutable reference of the stored type if it matches `T`.
    #[inline]
    pub fn downcast_ref<T: TypedExternal>(&self) -> Option<&T> {
        if !self.is::<T>() {
            return None;
        }
        unsafe { R_ExternalPtrAddr(self.sexp).cast::<T>().as_ref() }
    }

    /// Downcast to a mutable reference of the stored type if it matches `T`.
    #[inline]
    pub fn downcast_mut<T: TypedExternal>(&mut self) -> Option<&mut T> {
        if !self.is::<T>() {
            return None;
        }
        unsafe { R_ExternalPtrAddr(self.sexp).cast::<T>().as_mut() }
    }
}

/// Error returned when type checking fails in `try_from_sexp_with_error`.
///
/// The `found` field in `Mismatch` contains a `&'static str` from R's
/// interned symbol table, which persists for the R session lifetime.
#[derive(Debug, Clone)]
pub enum TypeMismatchError {
    /// The external pointer's address was null.
    NullPointer,
    /// The prot slot didn't contain a valid type symbol.
    InvalidTypeId,
    /// The stored type doesn't match the expected type.
    Mismatch {
        expected: &'static str,
        found: &'static str,
    },
}

impl fmt::Display for TypeMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NullPointer => write!(f, "external pointer is null"),
            Self::InvalidTypeId => write!(f, "external pointer has no valid type id"),
            Self::Mismatch { expected, found } => {
                write!(
                    f,
                    "type mismatch: expected `{}`, found `{}`",
                    expected, found
                )
            }
        }
    }
}

impl std::error::Error for TypeMismatchError {}

// =============================================================================
// MaybeUninit support
// =============================================================================

// We need a separate TypedExternal impl for MaybeUninit<T>
// This is typically done via blanket impl or macro

impl<T: TypedExternal> ExternalPtr<MaybeUninit<T>>
where
    MaybeUninit<T>: TypedExternal,
{
    /// Converts to `ExternalPtr<T>`.
    ///
    /// # Safety
    ///
    /// The value must have been initialized.
    ///
    /// # Implementation Note
    ///
    /// This method creates a *new* SEXP with `T`'s type information, leaving
    /// the original `MaybeUninit<T>` SEXP as an orphaned empty shell in R's heap.
    /// This is necessary because the type ID stored in the prot slot must match
    /// the actual type. The orphaned SEXP will be cleaned up by R's GC eventually.
    ///
    /// If you need to avoid this overhead, consider using `ExternalPtr<T>::new`
    /// directly and initializing in place via `as_mut`.
    ///
    /// Equivalent to `Box::assume_init`.
    #[inline]
    pub fn assume_init(self) -> ExternalPtr<T> {
        // Get the raw pointer (this clears the original SEXP, making its finalizer a no-op)
        let ptr = Self::into_raw(self).cast();

        // Create a new ExternalPtr with T's type info
        unsafe { ExternalPtr::from_raw(ptr) }
    }

    /// Writes a value and converts to initialized.
    ///
    /// See `assume_init` for notes about SEXP behavior.
    ///
    /// Equivalent to `Box::write`.
    #[inline]
    pub fn write(mut self, value: T) -> ExternalPtr<T> {
        unsafe {
            (*Self::as_mut_ptr(&mut self)).write(value);
            self.assume_init()
        }
    }
}
/// Type-erased `ExternalPtr` for cases where the concrete `T` is not needed.
pub type ErasedExternalPtr = ExternalPtr<()>;

// =============================================================================
// Trait Implementations
// =============================================================================

impl<T: TypedExternal> Deref for ExternalPtr<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        Self::as_ref(self).unwrap()
    }
}

impl<T: TypedExternal> DerefMut for ExternalPtr<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        Self::as_mut(self).unwrap()
    }
}

impl<T: TypedExternal> AsRef<T> for ExternalPtr<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        Self::as_ref(self).unwrap()
    }
}

impl<T: TypedExternal> AsMut<T> for ExternalPtr<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        Self::as_mut(self).unwrap()
    }
}

impl<T: TypedExternal> std::borrow::Borrow<T> for ExternalPtr<T> {
    #[inline]
    fn borrow(&self) -> &T {
        Self::as_ref(self).unwrap()
    }
}

impl<T: TypedExternal> std::borrow::BorrowMut<T> for ExternalPtr<T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        Self::as_mut(self).unwrap()
    }
}

impl<T: TypedExternal + Clone> Clone for ExternalPtr<T> {
    /// Deep clones the inner value into a new ExternalPtr.
    ///
    /// This creates a completely independent ExternalPtr with its own
    /// heap allocation and finalizer.
    #[inline]
    fn clone(&self) -> Self {
        Self::new((**self).clone())
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        (**self).clone_from(&**source);
    }
}

impl<T: TypedExternal + Default> Default for ExternalPtr<T> {
    /// Creates an ExternalPtr containing the default value of T.
    #[inline]
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: TypedExternal + fmt::Debug> fmt::Debug for ExternalPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: TypedExternal + fmt::Display> fmt::Display for ExternalPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: TypedExternal> fmt::Pointer for ExternalPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&Self::as_ptr(self), f)
    }
}

impl<T: TypedExternal + PartialEq> PartialEq for ExternalPtr<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T: TypedExternal + Eq> Eq for ExternalPtr<T> {}

impl<T: TypedExternal + PartialOrd> PartialOrd for ExternalPtr<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (**self).partial_cmp(&**other)
    }
}

impl<T: TypedExternal + Ord> Ord for ExternalPtr<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (**self).cmp(&**other)
    }
}

impl<T: TypedExternal + Hash> Hash for ExternalPtr<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T: TypedExternal + std::iter::Iterator> std::iter::Iterator for ExternalPtr<T> {
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        (**self).next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        (**self).nth(n)
    }
}

impl<T: TypedExternal + std::iter::DoubleEndedIterator> std::iter::DoubleEndedIterator
    for ExternalPtr<T>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        (**self).next_back()
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        (**self).nth_back(n)
    }
}

impl<T: TypedExternal + std::iter::ExactSizeIterator> std::iter::ExactSizeIterator
    for ExternalPtr<T>
{
    fn len(&self) -> usize {
        (**self).len()
    }
}

impl<T: TypedExternal + std::iter::FusedIterator> std::iter::FusedIterator for ExternalPtr<T> {}

impl<T: TypedExternal> From<T> for ExternalPtr<T> {
    #[inline]
    fn from(t: T) -> Self {
        Self::new(t)
    }
}

impl<T: TypedExternal> From<Box<T>> for ExternalPtr<T> {
    #[inline]
    fn from(boxed: Box<T>) -> Self {
        unsafe { Self::from_raw(Box::into_raw(boxed)) }
    }
}

// Note: We intentionally don't implement Drop for ExternalPtr.
// The R finalizer handles cleanup. If you need deterministic cleanup,
// use `into_raw` and manage it yourself.

// However, we need to think about what happens if the ExternalPtr is dropped
// in Rust without going through R's GC. This is a design decision.

// Option 1: No-op Drop (current) - R GC handles it eventually
// Option 2: Clear the pointer and let the value leak
// Option 3: Actually drop the value now

// For now, we implement a no-op. R will clean up.
impl<T: TypedExternal> Drop for ExternalPtr<T> {
    fn drop(&mut self) {
        // The finalizer registered with R will handle cleanup.
        // We don't do anything here to avoid double-free.
        //
        // If you want deterministic cleanup, use:
        //   let _ = ExternalPtr::into_inner(ptr);
        // or
        //   drop(unsafe { Box::from_raw(ExternalPtr::into_raw(ptr)) });
    }
}

// =============================================================================
// Finalizer
// =============================================================================

/// C finalizer function called by R's garbage collector.
///
/// This function is registered with R_RegisterCFinalizerEx and called when
/// the EXTPTRSXP is garbage collected.
extern "C-unwind" fn release_raw<T>(sexp: SEXP) {
    if sexp.is_null() {
        return;
    }
    if std::ptr::addr_eq(sexp.0, unsafe { R_NilValue.0 }) {
        return;
    }

    let ptr = unsafe { R_ExternalPtrAddr(sexp).cast::<T>() };

    // Guard against double-finalization
    if ptr.is_null() {
        return;
    }

    // Clear the external pointer first (prevents double-free if called again)
    unsafe { R_ClearExternalPtr(sexp) };

    // Reconstruct the Box and let it drop
    drop(unsafe { Box::from_raw(ptr) });
}

// =============================================================================
// Utility: ExternalSlice (helper for slice data)
// =============================================================================

/// A slice stored as a standalone struct, suitable for wrapping in ExternalPtr.
///
/// This is analogous to the data inside a `Box<[T]>`, but stores capacity
/// for proper deallocation when created from a `Vec`.
///
/// # Usage
///
/// To use with `ExternalPtr`, implement `TypedExternal` for your specific
/// `ExternalSlice<YourType>`:
///
/// ```ignore
/// impl_typed_external!(ExternalSlice<MyElement>);
/// let ptr = ExternalPtr::new(ExternalSlice::new(vec![1, 2, 3]));
/// ```
#[repr(C)]
pub struct ExternalSlice<T: 'static> {
    ptr: NonNull<T>,
    len: usize,
    capacity: usize,
}

impl<T: 'static> ExternalSlice<T> {
    /// Create an external slice from a `Vec`, preserving its allocation.
    pub fn new(slice: Vec<T>) -> Self {
        let mut vec = ManuallyDrop::new(slice);
        Self {
            ptr: unsafe { NonNull::new_unchecked(vec.as_mut_ptr()) },
            len: vec.len(),
            capacity: vec.capacity(),
        }
    }

    /// Create from a boxed slice (capacity == len).
    pub fn from_boxed(boxed: Box<[T]>) -> Self {
        let len = boxed.len();
        let ptr = Box::into_raw(boxed).cast();
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            len,
            capacity: len,
        }
    }

    /// Borrow the contents as a shared slice.
    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    /// Borrow the contents as a mutable slice.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    /// Number of elements in the slice.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the slice is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Capacity of the underlying allocation.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl<T: 'static> Drop for ExternalSlice<T> {
    fn drop(&mut self) {
        unsafe {
            let _ = Vec::from_raw_parts(self.ptr.as_ptr(), self.len, self.capacity);
        }
    }
}

// =============================================================================
// ALTREP Helpers
// =============================================================================

/// Extract the ALTREP data1 slot as a typed `ExternalPtr<T>`.
///
/// This is a convenience function for ALTREP implementations that store
/// their data in an `ExternalPtr` in the data1 slot.
///
/// # Safety
///
/// - `x` must be a valid ALTREP SEXP
/// - Must be called from the R main thread
///
/// # Example
///
/// ```ignore
/// impl Altrep for MyAltrepClass {
///     const HAS_LENGTH: bool = true;
///     fn length(x: SEXP) -> R_xlen_t {
///         match unsafe { altrep_data1_as::<MyData>(x) } {
///             Some(ext) => ext.data.len() as R_xlen_t,
///             None => 0,
///         }
///     }
/// }
/// ```
#[inline]
pub unsafe fn altrep_data1_as<T: TypedExternal>(x: SEXP) -> Option<ExternalPtr<T>> {
    unsafe { ExternalPtr::wrap_sexp(crate::ffi::R_altrep_data1(x)) }
}

/// Extract the ALTREP data1 slot (unchecked version).
///
/// Skips thread safety checks for performance-critical ALTREP callbacks.
///
/// # Safety
///
/// - `x` must be a valid ALTREP SEXP
/// - Must be called from the R main thread (guaranteed in ALTREP callbacks)
#[inline]
pub unsafe fn altrep_data1_as_unchecked<T: TypedExternal>(x: SEXP) -> Option<ExternalPtr<T>> {
    use crate::ffi::R_altrep_data1_unchecked;
    unsafe { ExternalPtr::wrap_sexp_unchecked(R_altrep_data1_unchecked(x)) }
}

/// Extract the ALTREP data2 slot as a typed `ExternalPtr<T>`.
///
/// Similar to `altrep_data1_as`, but for the data2 slot.
///
/// # Safety
///
/// - `x` must be a valid ALTREP SEXP
/// - Must be called from the R main thread
#[inline]
pub unsafe fn altrep_data2_as<T: TypedExternal>(x: SEXP) -> Option<ExternalPtr<T>> {
    unsafe { ExternalPtr::wrap_sexp(crate::ffi::R_altrep_data2(x)) }
}

/// Extract the ALTREP data2 slot (unchecked version).
///
/// Skips thread safety checks for performance-critical ALTREP callbacks.
///
/// # Safety
///
/// - `x` must be a valid ALTREP SEXP
/// - Must be called from the R main thread (guaranteed in ALTREP callbacks)
#[inline]
pub unsafe fn altrep_data2_as_unchecked<T: TypedExternal>(x: SEXP) -> Option<ExternalPtr<T>> {
    use crate::ffi::R_altrep_data2_unchecked;
    unsafe { ExternalPtr::wrap_sexp_unchecked(R_altrep_data2_unchecked(x)) }
}

/// Get a mutable reference to data in ALTREP data1 slot via `ErasedExternalPtr`.
///
/// This is useful for ALTREP methods that need to mutate the underlying data.
///
/// # Safety
///
/// - `x` must be a valid ALTREP SEXP
/// - Must be called from the R main thread
/// - The caller must ensure no other references to the data exist
///
/// # Example
///
/// ```ignore
/// fn dataptr(x: SEXP, _writable: bool) -> *mut c_void {
///     match unsafe { altrep_data1_mut::<MyData>(x) } {
///         Some(data) => data.buffer.as_mut_ptr().cast(),
///         None => core::ptr::null_mut(),
///     }
/// }
/// ```
#[inline]
pub unsafe fn altrep_data1_mut<T: TypedExternal>(x: SEXP) -> Option<&'static mut T> {
    unsafe {
        let mut erased = ErasedExternalPtr::from_sexp(crate::ffi::R_altrep_data1(x));
        // Transmute the lifetime to 'static - this is safe because:
        // 1. The ExternalPtr is protected by R's GC as part of the ALTREP object
        // 2. The ALTREP object `x` is kept alive by R during the callback
        erased.downcast_mut::<T>().map(|r| std::mem::transmute(r))
    }
}

/// Get a mutable reference to data in ALTREP data1 slot (unchecked version).
///
/// Skips thread safety checks for performance-critical ALTREP callbacks.
///
/// # Safety
///
/// - `x` must be a valid ALTREP SEXP
/// - Must be called from the R main thread (guaranteed in ALTREP callbacks)
/// - The caller must ensure no other references to the data exist
#[inline]
pub unsafe fn altrep_data1_mut_unchecked<T: TypedExternal>(x: SEXP) -> Option<&'static mut T> {
    use crate::ffi::R_altrep_data1_unchecked;
    unsafe {
        let mut erased = ErasedExternalPtr::from_sexp(R_altrep_data1_unchecked(x));
        erased.downcast_mut::<T>().map(|r| std::mem::transmute(r))
    }
}

// Tests for ExternalPtr require R runtime, so they are in rpkg/src/rust/lib.rs

// =============================================================================
// Sidecar Marker Type for #[r_data] Fields
// =============================================================================

/// Marker type for enabling R sidecar accessors in an `ExternalPtr` struct.
///
/// When used with `#[derive(ExternalPtr)]` and `#[r_data]`, this field acts as
/// a selector that enables R-facing accessors for sibling `#[r_data]` fields.
///
/// # Supported Field Types
///
/// - **`SEXP`** - Raw SEXP access, no conversion
/// - **`i32`, `f64`, `bool`, `u8`** - Zero-overhead scalars (stored directly in R)
/// - **Any `IntoR` type** - Automatic conversion (e.g., `String`, `Vec<T>`)
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::ffi::SEXP;
///
/// #[derive(ExternalPtr)]
/// pub struct MyType {
///     pub x: i32,
///
///     /// Selector field - enables R wrapper generation
///     #[r_data]
///     r: RSidecar,
///
///     /// Raw SEXP slot - MyType_get_raw() / MyType_set_raw()
///     #[r_data]
///     pub raw: SEXP,
///
///     /// Zero-overhead scalar - MyType_get_count() / MyType_set_count()
///     #[r_data]
///     pub count: i32,
///
///     /// Conversion type - MyType_get_name() / MyType_set_name()
///     #[r_data]
///     pub name: String,
/// }
/// ```
///
/// # Design Notes
///
/// - `RSidecar` is a ZST (zero-sized type) - no runtime cost
/// - Only `pub` `#[r_data]` fields get R wrapper functions generated
/// - Multiple `RSidecar` fields in one struct is a compile error
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct RSidecar;
