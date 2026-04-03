//! `ExternalPtr<T>` — a Box-like owned pointer that wraps R's EXTPTRSXP.
//!
//! This provides ownership semantics similar to `Box<T>`, with the key difference
//! that cleanup is deferred to R's garbage collector via finalizers.
//!
//! # Submodules
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`altrep_helpers`] | ALTREP data1/data2 slot access helpers + `Sidecar` marker type |
//!
//! # Core Types
//!
//! - [`ExternalPtr<T>`] — owned pointer wrapping EXTPTRSXP
//! - [`TypedExternal`] — trait for type-safe identification across packages
//! - [`ExternalSlice<T>`] — helper for slice data in external pointers
//! - [`ErasedExternalPtr`] — type-erased `ExternalPtr<()>` alias
//! - [`IntoExternalPtr`] — conversion trait for wrapping values
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
//! Type safety is enforced via `Any::downcast` (Rust's `TypeId`). R symbols
//! in the `tag` and `prot` slots are retained for display and error messages.
//!
//! Internally, data is stored as `Box<Box<dyn Any>>` — a thin pointer (fits
//! in R's `R_ExternalPtrAddr`) pointing to a fat pointer (carries the `Any`
//! vtable for runtime downcasting).
//!
//! The `tag` slot holds a symbol (type name, for display).
//! The `prot` slot holds a VECSXP (list) with two elements:
//!   - Index 0: SYMSXP (interned type ID symbol, for error messages)
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

use std::any::Any;
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
    R_MakeExternalPtr, R_MakeExternalPtr_unchecked, R_RegisterCFinalizerEx,
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
/// The data is owned and transferred to the main thread before being accessed.
type SendableAnyPtr = crate::worker::Sendable<NonNull<Box<dyn Any>>>;

/// Create a new sendable pointer from a raw `*mut Box<dyn Any>`.
///
/// # Safety
///
/// The pointer must be non-null.
#[inline]
unsafe fn sendable_any_ptr_new(ptr: *mut Box<dyn Any>) -> SendableAnyPtr {
    // SAFETY: Caller guarantees ptr is non-null
    crate::worker::Sendable(unsafe { NonNull::new_unchecked(ptr) })
}

/// Get the raw pointer, consuming the sendable wrapper.
#[inline]
fn sendable_any_ptr_into_ptr(ptr: SendableAnyPtr) -> *mut Box<dyn Any> {
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
    unsafe {
        std::str::from_utf8(std::slice::from_raw_parts(cstr.cast(), len))
            .expect("R SYMSXP PRINTNAME is not valid UTF-8")
    }
}

// region: TypedExternalPtr Trait

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

impl TypedExternal for () {
    const TYPE_NAME: &'static str = "()";
    const TYPE_NAME_CSTR: &'static [u8] = b"()\0";
    // Unit type is special - same ID as name since it's only used for type-erased ptrs
    const TYPE_ID_CSTR: &'static [u8] = b"()\0";
}
// endregion

// region: ExternalPtr<T>

/// An owned pointer stored in R's external pointer SEXP.
///
/// This is conceptually similar to `Box<T>`, but with the following differences:
/// - Memory is freed by R's GC via a registered finalizer (non-deterministic)
/// - The underlying SEXP is Copy, so aliasing must be manually prevented
/// - Type checking happens at runtime via `Any::downcast` (Rust `TypeId`)
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
///
/// # Examples
///
/// ```no_run
/// use miniextendr_api::externalptr::{ExternalPtr, TypedExternal};
///
/// struct MyData { value: f64 }
/// impl TypedExternal for MyData {
///     const TYPE_NAME: &'static str = "MyData";
///     const TYPE_NAME_CSTR: &'static [u8] = b"MyData\0";
///     const TYPE_ID_CSTR: &'static [u8] = b"my_crate::MyData\0";
/// }
///
/// let ptr = ExternalPtr::new(MyData { value: 3.14 });
/// assert_eq!(ptr.as_ref().unwrap().value, 3.14);
/// ```
#[repr(C)]
pub struct ExternalPtr<T: TypedExternal> {
    sexp: SEXP,
    /// Cached data pointer, set once at construction time.
    ///
    /// This avoids the `R_ExternalPtrAddr` FFI call on every `as_ref()`/`as_mut()`.
    /// The pointer remains valid for the lifetime of the `ExternalPtr` because:
    /// - R's finalizer only runs after R garbage-collects the SEXP (which cannot
    ///   happen while a Rust `ExternalPtr` value exists).
    /// - `R_ClearExternalPtr` is only called in methods that consume or finalize
    ///   (`into_raw`, `into_inner`, `release_any`).
    cached_ptr: NonNull<T>,
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
    /// Internally stores a `Box<Box<dyn Any>>` — a thin pointer (fits in R's
    /// `R_ExternalPtrAddr`) pointing to a fat pointer (carries the `Any` vtable
    /// for runtime type checking via `downcast`).
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
        // Get concrete pointer with full write provenance from Box::into_raw,
        // BEFORE erasing to dyn Any. This preserves mutable provenance for
        // cached_ptr (downcast_ref would give shared-reference provenance,
        // which is UB for later writes through as_mut()).
        let raw: *mut T = Box::into_raw(Box::new(x));
        // SAFETY: Box::into_raw never returns null
        let cached_ptr = unsafe { NonNull::new_unchecked(raw) };

        // Re-wrap: Box::from_raw(raw) → Box<dyn Any> → Box<Box<dyn Any>>
        // The data stays at `raw`; we're just adding the Any vtable wrapper.
        let inner: Box<dyn Any> = unsafe { Box::from_raw(raw) };
        let any_raw: *mut Box<dyn Any> = Box::into_raw(Box::new(inner));

        // Wrap in Sendable so it can be sent across thread boundary
        let sendable = unsafe { sendable_any_ptr_new(any_raw) };

        // Use with_r_thread to run R API calls on main thread
        let sexp = crate::worker::with_r_thread(move || {
            let any_raw = sendable_any_ptr_into_ptr(sendable);
            unsafe { Self::create_extptr_sexp_unchecked(any_raw) }
        });

        Self {
            sexp,
            cached_ptr,
            _marker: PhantomData,
        }
    }

    /// Allocates memory on the heap and places `x` into it, without thread checks.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread. Calling from another thread
    /// is undefined behavior (R APIs are not thread-safe).
    #[inline]
    pub unsafe fn new_unchecked(x: T) -> Self {
        let raw: *mut T = Box::into_raw(Box::new(x));
        let cached_ptr = unsafe { NonNull::new_unchecked(raw) };

        let inner: Box<dyn Any> = unsafe { Box::from_raw(raw) };
        let any_raw: *mut Box<dyn Any> = Box::into_raw(Box::new(inner));

        let sexp = unsafe { Self::create_extptr_sexp_unchecked(any_raw) };
        Self {
            sexp,
            cached_ptr,
            _marker: PhantomData,
        }
    }

    /// Create an EXTPTRSXP from a `*mut Box<dyn Any>`. Must be called from main thread.
    ///
    /// The `any_raw` is a thin pointer to a heap-allocated fat pointer (`Box<dyn Any>`).
    /// R stores the thin pointer in `R_ExternalPtrAddr`.
    #[inline]
    unsafe fn create_extptr_sexp(any_raw: *mut Box<dyn Any>) -> SEXP {
        debug_assert!(
            !any_raw.is_null(),
            "create_extptr_sexp received null pointer"
        );

        let type_sym = unsafe { type_symbol::<T>() };
        let type_id_sym = unsafe { type_id_symbol::<T>() };

        let prot = unsafe { Rf_allocVector(SEXPTYPE::VECSXP, PROT_VEC_LEN) };
        unsafe { Rf_protect(prot) };
        unsafe { SET_VECTOR_ELT(prot, PROT_TYPE_ID_INDEX, type_id_sym) };

        let sexp = unsafe { R_MakeExternalPtr(any_raw.cast(), type_sym, prot) };
        unsafe { Rf_protect(sexp) };

        // Non-generic finalizer — Box<dyn Any> vtable handles the concrete drop
        unsafe { R_RegisterCFinalizerEx(sexp, Some(release_any), Rboolean::TRUE) };

        unsafe { Rf_unprotect(2) };
        sexp
    }

    /// Create an EXTPTRSXP from a `*mut Box<dyn Any>` without thread safety checks.
    ///
    /// # Safety
    ///
    /// Must be called from R's main thread. No debug assertions for thread safety.
    #[inline]
    unsafe fn create_extptr_sexp_unchecked(any_raw: *mut Box<dyn Any>) -> SEXP {
        debug_assert!(
            !any_raw.is_null(),
            "create_extptr_sexp_unchecked received null pointer"
        );

        let type_sym = unsafe { type_symbol_unchecked::<T>() };
        let type_id_sym = unsafe { type_id_symbol_unchecked::<T>() };

        let prot = unsafe { Rf_allocVector_unchecked(SEXPTYPE::VECSXP, PROT_VEC_LEN) };
        unsafe { Rf_protect_unchecked(prot) };
        unsafe { SET_VECTOR_ELT_unchecked(prot, PROT_TYPE_ID_INDEX, type_id_sym) };

        let sexp = unsafe { R_MakeExternalPtr_unchecked(any_raw.cast(), type_sym, prot) };
        unsafe { Rf_protect_unchecked(sexp) };

        // Non-generic finalizer — Box<dyn Any> vtable handles the concrete drop
        unsafe {
            R_RegisterCFinalizerEx_unchecked(sexp, Some(release_any), Rboolean::TRUE);
        };

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
    /// Re-wraps the `*mut T` in `Box<dyn Any>` for the new storage format.
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
        // Re-wrap in Box<dyn Any> → Box<Box<dyn Any>>
        let inner: Box<dyn Any> = unsafe { Box::from_raw(raw) };
        let outer: Box<Box<dyn Any>> = Box::new(inner);
        let any_raw: *mut Box<dyn Any> = Box::into_raw(outer);

        let sexp = unsafe { Self::create_extptr_sexp(any_raw) };
        Self {
            sexp,
            cached_ptr: unsafe { NonNull::new_unchecked(raw) },
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
        let inner: Box<dyn Any> = unsafe { Box::from_raw(raw) };
        let outer: Box<Box<dyn Any>> = Box::new(inner);
        let any_raw: *mut Box<dyn Any> = Box::into_raw(outer);

        let sexp = unsafe { Self::create_extptr_sexp_unchecked(any_raw) };
        Self {
            sexp,
            cached_ptr: unsafe { NonNull::new_unchecked(raw) },
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
        let ptr = this.cached_ptr.as_ptr();

        // Recover and disassemble the Box<Box<dyn Any>> wrapper.
        // We need to free the wrapper allocations without dropping the T data.
        let any_raw = unsafe { R_ExternalPtrAddr(this.sexp) as *mut Box<dyn Any> };

        // Clear the external pointer so the finalizer becomes a no-op
        unsafe { R_ClearExternalPtr(this.sexp) };

        if !any_raw.is_null() {
            // Reconstruct outer box → extract inner → leak inner (prevents T drop)
            let outer: Box<Box<dyn Any>> = unsafe { Box::from_raw(any_raw) };
            let inner: Box<dyn Any> = *outer;
            // Box::into_raw leaks the inner allocation — caller owns T via `ptr`
            let _ = Box::into_raw(inner);
        }

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
    /// Uses `Box<dyn Any>::downcast` to recover the concrete `Box<T>`,
    /// then moves the value out.
    ///
    /// Equivalent to `*boxed` (deref move) or `Box::into_inner`.
    #[inline]
    pub fn into_inner(this: Self) -> T {
        let any_raw = unsafe { R_ExternalPtrAddr(this.sexp) as *mut Box<dyn Any> };

        // Clear so finalizer is no-op
        unsafe { R_ClearExternalPtr(this.sexp) };
        mem::forget(this);

        assert!(!any_raw.is_null(), "ExternalPtr is null or cleared");
        let outer: Box<Box<dyn Any>> = unsafe { Box::from_raw(any_raw) };
        let inner: Box<dyn Any> = *outer;
        *inner
            .downcast::<T>()
            .expect("ExternalPtr type mismatch in into_inner")
    }

    // region: Pin support (Box-equivalent)

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
    // endregion

    // region: Accessors

    /// Returns a reference to the underlying value.
    ///
    /// Uses the cached pointer set at construction time, avoiding the
    /// `R_ExternalPtrAddr` FFI call on every access.
    #[inline]
    pub fn as_ref(&self) -> Option<&T> {
        // SAFETY: cached_ptr is always valid for the lifetime of ExternalPtr
        Some(unsafe { self.cached_ptr.as_ref() })
    }

    /// Returns a mutable reference to the underlying value.
    ///
    /// Uses the cached pointer set at construction time, avoiding the
    /// `R_ExternalPtrAddr` FFI call on every access.
    #[inline]
    pub fn as_mut(&mut self) -> Option<&mut T> {
        // SAFETY: cached_ptr is always valid for the lifetime of ExternalPtr
        Some(unsafe { self.cached_ptr.as_mut() })
    }

    /// Returns the raw pointer without consuming the ExternalPtr.
    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.cached_ptr.as_ptr().cast_const()
    }

    /// Returns the raw mutable pointer without consuming the ExternalPtr.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.cached_ptr.as_ptr()
    }

    /// Checks whether two `ExternalPtr`s refer to the same allocation (pointer identity).
    ///
    /// This ignores the pointee values. Use this when you need alias detection;
    /// prefer `PartialEq`/`PartialOrd` or `as_ref()` for value comparisons.
    #[inline]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        ptr::eq(
            this.cached_ptr.as_ptr().cast_const(),
            other.cached_ptr.as_ptr().cast_const(),
        )
    }
    // endregion

    // region: R-specific accessors

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
                return SEXP::null();
            }
            if prot.type_of() != SEXPTYPE::VECSXP || prot.len() < PROT_VEC_LEN as usize {
                return SEXP::null();
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
                return SEXP::null();
            }
            if prot.type_of() != SEXPTYPE::VECSXP || prot.len() < PROT_VEC_LEN as usize {
                return SEXP::null();
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
    // endregion

    // region: Type checking

    /// Attempt to wrap a SEXP as an ExternalPtr with type checking.
    ///
    /// Uses `Any::downcast_ref` for authoritative type checking (Rust `TypeId`).
    /// Falls back to R symbol comparison for type-erased `ExternalPtr<()>`.
    ///
    /// Returns `None` if:
    /// - The internal pointer is null
    /// - The stored `Box<dyn Any>` does not contain a `T`
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid EXTPTRSXP created by this library
    /// - The caller must ensure no other ExternalPtr owns this SEXP
    pub unsafe fn wrap_sexp(sexp: SEXP) -> Option<Self> {
        debug_assert_eq!(
            sexp.type_of(),
            crate::ffi::SEXPTYPE::EXTPTRSXP,
            "wrap_sexp: expected EXTPTRSXP, got {:?}",
            sexp.type_of()
        );
        let any_raw = unsafe { R_ExternalPtrAddr(sexp) as *mut Box<dyn Any> };
        if any_raw.is_null() {
            return None;
        }

        if is_type_erased::<T>() {
            // Type-erased path: skip downcast, just use the raw pointer
            // (ExternalPtr<()> doesn't care about the concrete type)
            return Some(Self {
                sexp,
                cached_ptr: unsafe { NonNull::new_unchecked(any_raw.cast::<T>()) },
                _marker: PhantomData,
            });
        }

        // Use downcast_mut (not downcast_ref) so cached_ptr gets mutable
        // provenance — shared-reference provenance from downcast_ref would
        // make later writes through as_mut() UB under Stacked Borrows.
        let any_box: &mut Box<dyn Any> = unsafe { &mut *any_raw };
        let concrete: &mut T = any_box.downcast_mut::<T>()?;

        Some(Self {
            sexp,
            cached_ptr: unsafe { NonNull::new_unchecked(ptr::from_mut(concrete)) },
            _marker: PhantomData,
        })
    }

    /// Attempt to wrap a SEXP as an ExternalPtr (unchecked version).
    ///
    /// Skips thread safety checks for performance-critical paths like ALTREP callbacks.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid EXTPTRSXP created by this library
    /// - The caller must ensure exclusive ownership
    /// - Must be called from the R main thread (guaranteed in ALTREP callbacks)
    pub unsafe fn wrap_sexp_unchecked(sexp: SEXP) -> Option<Self> {
        use crate::ffi::R_ExternalPtrAddr_unchecked;

        debug_assert_eq!(
            sexp.type_of(),
            crate::ffi::SEXPTYPE::EXTPTRSXP,
            "wrap_sexp_unchecked: expected EXTPTRSXP, got {:?}",
            sexp.type_of()
        );
        let any_raw = unsafe { R_ExternalPtrAddr_unchecked(sexp) as *mut Box<dyn Any> };
        if any_raw.is_null() {
            return None;
        }

        if is_type_erased::<T>() {
            return Some(Self {
                sexp,
                cached_ptr: unsafe { NonNull::new_unchecked(any_raw.cast::<T>()) },
                _marker: PhantomData,
            });
        }

        let any_box: &mut Box<dyn Any> = unsafe { &mut *any_raw };
        let concrete: &mut T = any_box.downcast_mut::<T>()?;

        Some(Self {
            sexp,
            cached_ptr: unsafe { NonNull::new_unchecked(ptr::from_mut(concrete)) },
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
        debug_assert_eq!(
            sexp.type_of(),
            crate::ffi::SEXPTYPE::EXTPTRSXP,
            "wrap_sexp_with_error: expected EXTPTRSXP, got {:?}",
            sexp.type_of()
        );
        let any_raw = unsafe { R_ExternalPtrAddr(sexp) as *mut Box<dyn Any> };
        if any_raw.is_null() {
            return Err(TypeMismatchError::NullPointer);
        }

        if is_type_erased::<T>() {
            return Ok(Self {
                sexp,
                cached_ptr: unsafe { NonNull::new_unchecked(any_raw.cast::<T>()) },
                _marker: PhantomData,
            });
        }

        let any_box: &mut Box<dyn Any> = unsafe { &mut *any_raw };
        match any_box.downcast_mut::<T>() {
            Some(concrete) => Ok(Self {
                sexp,
                cached_ptr: unsafe { NonNull::new_unchecked(ptr::from_mut(concrete)) },
                _marker: PhantomData,
            }),
            None => {
                // Try to get the stored type name from R symbol for error reporting
                let found = unsafe {
                    let prot = R_ExternalPtrProtected(sexp);
                    if !prot.is_null_or_nil()
                        && prot.type_of() == SEXPTYPE::VECSXP
                        && prot.len() >= PROT_VEC_LEN as usize
                    {
                        let stored_sym = VECTOR_ELT(prot, PROT_TYPE_ID_INDEX);
                        if stored_sym.type_of() == SEXPTYPE::SYMSXP {
                            symbol_name(stored_sym)
                        } else {
                            "<unknown>"
                        }
                    } else {
                        "<unknown>"
                    }
                };
                Err(TypeMismatchError::Mismatch {
                    expected: T::TYPE_NAME,
                    found,
                })
            }
        }
    }

    /// Create an ExternalPtr from an SEXP without type checking.
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid EXTPTRSXP containing a `*mut Box<dyn Any>`
    ///   wrapping a value of type `T`
    /// - The caller must ensure exclusive ownership
    #[inline]
    pub unsafe fn from_sexp_unchecked(sexp: SEXP) -> Self {
        debug_assert_eq!(
            sexp.type_of(),
            crate::ffi::SEXPTYPE::EXTPTRSXP,
            "from_sexp_unchecked: expected EXTPTRSXP, got {:?}",
            sexp.type_of()
        );
        let any_raw = unsafe { R_ExternalPtrAddr(sexp) as *mut Box<dyn Any> };
        debug_assert!(!any_raw.is_null(), "from_sexp_unchecked: null pointer");

        let cached_ptr = if is_type_erased::<T>() {
            unsafe { NonNull::new_unchecked(any_raw.cast::<T>()) }
        } else {
            let any_box: &mut Box<dyn Any> = unsafe { &mut *any_raw };
            let concrete: &mut T = unsafe { any_box.downcast_mut::<T>().unwrap_unchecked() };
            unsafe { NonNull::new_unchecked(ptr::from_mut(concrete)) }
        };

        Self {
            sexp,
            cached_ptr,
            _marker: PhantomData,
        }
    }
    // endregion

    // region: Downcast support

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
    // endregion
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

    /// Check whether the stored `Box<dyn Any>` contains a `T`.
    ///
    /// Uses `Any::is` for authoritative runtime type checking.
    #[inline]
    pub fn is<T: TypedExternal>(&self) -> bool {
        let any_raw = unsafe { R_ExternalPtrAddr(self.sexp) as *mut Box<dyn Any> };
        if any_raw.is_null() {
            return false;
        }
        let any_box: &Box<dyn Any> = unsafe { &*any_raw };
        any_box.is::<T>()
    }

    /// Downcast to an immutable reference of the stored type if it matches `T`.
    ///
    /// Uses `Any::downcast_ref` for authoritative runtime type checking.
    #[inline]
    pub fn downcast_ref<T: TypedExternal>(&self) -> Option<&T> {
        let any_raw = unsafe { R_ExternalPtrAddr(self.sexp) as *mut Box<dyn Any> };
        if any_raw.is_null() {
            return None;
        }
        let any_box: &Box<dyn Any> = unsafe { &*any_raw };
        any_box.downcast_ref::<T>()
    }

    /// Downcast to a mutable reference of the stored type if it matches `T`.
    ///
    /// Uses `Any::downcast_mut` for authoritative runtime type checking.
    #[inline]
    pub fn downcast_mut<T: TypedExternal>(&mut self) -> Option<&mut T> {
        let any_raw = unsafe { R_ExternalPtrAddr(self.sexp) as *mut Box<dyn Any> };
        if any_raw.is_null() {
            return None;
        }
        let any_box: &mut Box<dyn Any> = unsafe { &mut *any_raw };
        any_box.downcast_mut::<T>()
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
        /// Expected Rust type name from this pointer wrapper.
        expected: &'static str,
        /// Actual stored Rust type name found in pointer metadata.
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
// endregion

// region: MaybeUninit support

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
    /// Creates a new SEXP with `T`'s type information (the original
    /// `MaybeUninit<T>` SEXP becomes an orphaned shell, cleaned up by GC).
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
// endregion

// region: Trait Implementations

impl<T: TypedExternal> Deref for ExternalPtr<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        Self::as_ref(self).expect("ExternalPtr is null or cleared")
    }
}

impl<T: TypedExternal> DerefMut for ExternalPtr<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        Self::as_mut(self).expect("ExternalPtr is null or cleared")
    }
}

impl<T: TypedExternal> AsRef<T> for ExternalPtr<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        Self::as_ref(self).expect("ExternalPtr is null or cleared")
    }
}

impl<T: TypedExternal> AsMut<T> for ExternalPtr<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        Self::as_mut(self).expect("ExternalPtr is null or cleared")
    }
}

impl<T: TypedExternal> std::borrow::Borrow<T> for ExternalPtr<T> {
    #[inline]
    fn borrow(&self) -> &T {
        Self::as_ref(self).expect("ExternalPtr is null or cleared")
    }
}

impl<T: TypedExternal> std::borrow::BorrowMut<T> for ExternalPtr<T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        Self::as_mut(self).expect("ExternalPtr is null or cleared")
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
// endregion

// region: Finalizer

/// Non-generic C finalizer called by R's garbage collector.
///
/// Since `ExternalPtr` stores `Box<Box<dyn Any>>`, the `Any` vtable carries
/// the concrete type's drop function. No generic parameter needed — one
/// finalizer function handles all `ExternalPtr<T>` types.
extern "C-unwind" fn release_any(sexp: SEXP) {
    if sexp.is_null() {
        return;
    }
    if sexp.is_nil() {
        return;
    }

    let any_raw = unsafe { R_ExternalPtrAddr(sexp) as *mut Box<dyn Any> };

    // Guard against double-finalization
    if any_raw.is_null() {
        return;
    }

    // Clear the external pointer first (prevents double-free if called again)
    unsafe { R_ClearExternalPtr(sexp) };

    // Reconstruct the outer Box<Box<dyn Any>> and let it drop.
    // This drops the outer Box, then the inner Box<dyn Any>, which
    // uses the vtable to drop the concrete T value.
    drop(unsafe { Box::from_raw(any_raw) });
}
// endregion

// region: Utility: ExternalSlice (helper for slice data)

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
// endregion

mod altrep_helpers;
pub use altrep_helpers::*;
