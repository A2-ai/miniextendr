//! ExternalPtr<T> - A Box-like owned pointer that wraps R's EXTPTRSXP
//!
//! This provides ownership semantics similar to Box<T>, with the key difference
//! that cleanup is deferred to R's garbage collector via finalizers.
//!
//! # Type Identification
//!
//! The `tag` slot holds a human-readable symbol (type name).
//! The `prot` slot holds a VECSXP (list) with two elements:
//!   - Index 0: RAWSXP containing the `StableTypeId` for fast type comparison
//!   - Index 1: User-protected SEXP slot (for preventing GC of R objects)

use std::alloc::Layout;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem::{self, ManuallyDrop, MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::ptr::{self, NonNull};

use crate::ffi::{
    R_ClearExternalPtr, R_ExternalPtrAddr, R_ExternalPtrProtected, R_ExternalPtrTag,
    R_MakeExternalPtr, R_NilValue, R_RegisterCFinalizerEx, RAW, Rboolean, Rf_allocVector,
    Rf_install, Rf_protect, Rf_unprotect, Rf_xlength, SET_VECTOR_ELT, SEXP, SEXPTYPE, TYPEOF,
    VECTOR_ELT,
};

/// Index of the StableTypeId RAWSXP in the prot VECSXP
const PROT_TYPE_ID_INDEX: isize = 0;
/// Index of user-protected objects in the prot VECSXP
const PROT_USER_INDEX: isize = 1;
/// Length of the prot VECSXP
const PROT_VEC_LEN: isize = 2;

// =============================================================================
// Stable Type Identification
// =============================================================================

/// A stable type identifier that works across different rustc versions.
///
/// Unlike `std::any::TypeId`, this uses `std::any::type_name` which provides
/// a stable string representation. We hash this at compile time for fast comparison.
///
/// Note: `type_name` output isn't guaranteed to be unique across all types,
/// but in practice it's sufficient for FFI type-checking purposes.
///
/// # Layout
///
/// This struct is `repr(C)` so it can be safely stored in a RAWSXP and
/// retrieved via pointer cast.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct StableTypeId {
    /// Hash of the type name (for fast comparison)
    hash: u64,
    /// Length of the type name
    name_len: usize,
    /// Pointer to the static type name string
    name_ptr: *const u8,
}

unsafe impl Send for StableTypeId {}
unsafe impl Sync for StableTypeId {}

impl StableTypeId {
    /// Create a new StableTypeId for type T
    #[inline]
    pub fn of<T: ?Sized + 'static>() -> Self {
        let name = std::any::type_name::<T>();
        Self {
            hash: const_hash_str(name),
            name_len: name.len(),
            name_ptr: name.as_ptr(),
        }
    }

    /// Get the type name as a string slice
    #[inline]
    pub fn name(&self) -> &'static str {
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.name_ptr, self.name_len))
        }
    }

    /// Get the hash value
    #[inline]
    pub const fn hash_value(&self) -> u64 {
        self.hash
    }

    /// Create a RAWSXP containing this StableTypeId.
    ///
    /// The returned SEXP must be protected by the caller if needed.
    #[inline]
    unsafe fn to_rawsxp(self) -> SEXP {
        let size = mem::size_of::<Self>();
        let raw = unsafe { Rf_allocVector(SEXPTYPE::RAWSXP, size as isize) };
        unsafe { ptr::copy_nonoverlapping(std::ptr::from_ref(&self).cast(), RAW(raw), size) };
        raw
    }

    /// Extract a StableTypeId from a RAWSXP.
    ///
    /// Returns `None` if:
    /// - The SEXP is not a RAWSXP
    /// - The RAWSXP is not the correct size
    #[inline]
    fn from_rawsxp(sexp: SEXP) -> Option<Self> {
        if sexp.is_null() || sexp == unsafe { R_NilValue } {
            return None;
        }

        if unsafe { TYPEOF(sexp) } != SEXPTYPE::RAWSXP {
            return None;
        }

        let expected_size = mem::size_of::<Self>();
        if unsafe { Rf_xlength(sexp) } as usize != expected_size {
            return None;
        }

        let mut result = MaybeUninit::<Self>::uninit();
        unsafe { ptr::copy_nonoverlapping(RAW(sexp), result.as_mut_ptr().cast(), expected_size) };
        Some(unsafe { result.assume_init() })
    }
}

impl PartialEq for StableTypeId {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        // Fast path: compare hashes first
        if self.hash != other.hash {
            return false;
        }
        // Slow path: compare names for hash collision safety
        self.name() == other.name()
    }
}

impl Eq for StableTypeId {}

impl fmt::Debug for StableTypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StableTypeId")
            .field("name", &self.name())
            .field("hash", &format_args!("{:#018x}", self.hash))
            .finish()
    }
}

impl Hash for StableTypeId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

/// Compile-time string hashing using FNV-1a
const fn const_hash_str(s: &str) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let bytes = s.as_bytes();
    let mut hash = FNV_OFFSET;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }
    hash
}

// =============================================================================
// TypedExternalPtr Trait
// =============================================================================

/// Trait for types that can be stored in an ExternalPtr.
///
/// This provides the stable type identification needed for runtime type checking.
pub trait TypedExternal: 'static {
    /// The stable type identifier for this type
    const TYPE_ID: StableTypeId;

    /// The type name as a C string (for R tag)
    const TYPE_NAME_CSTR: &'static [u8];
}

/// Implement TypedExternal for a type.
///
/// This macro generates both the `StableTypeId` (using `std::any::type_name`)
/// and a C-string tag name for R (also using `type_name` for consistency).
///
/// # Note
///
/// The tag name uses `std::any::type_name` at runtime to match the `StableTypeId`.
/// This means the tag will include the full module path (e.g., "mycrate::MyStruct").
#[macro_export]
macro_rules! impl_typed_external {
    ($ty:ty) => {
        impl $crate::externalptr::TypedExternal for $ty {
            const TYPE_ID: $crate::externalptr::StableTypeId =
                $crate::externalptr::StableTypeId::of::<$ty>();

            // We can't use type_name in const context for the C string,
            // so we use a function that will be called at runtime for the tag.
            // However, since TYPE_NAME_CSTR needs to be const, we use stringify
            // as a fallback. The tag is only for human debugging; type checking
            // uses TYPE_ID which is consistent.
            const TYPE_NAME_CSTR: &'static [u8] = concat!(stringify!($ty), "\0").as_bytes();
        }
    };
}

/// Implement TypedExternal for a type with a custom tag name.
///
/// Use this when you want the R tag to display a specific name
/// (e.g., without module path).
#[macro_export]
macro_rules! impl_typed_external_with_tag {
    ($ty:ty, $tag:expr) => {
        impl $crate::externalptr::TypedExternal for $ty {
            const TYPE_ID: $crate::externalptr::StableTypeId =
                $crate::externalptr::StableTypeId::of::<$ty>();
            const TYPE_NAME_CSTR: &'static [u8] = concat!($tag, "\0").as_bytes();
        }
    };
}

// =============================================================================
// ExternalPtr<T>
// =============================================================================

/// Marker type to make ExternalPtr !Send and !Sync without nightly features.
/// Contains a raw pointer which is !Send and !Sync by default.
type PhantomUnsend = PhantomData<*mut ()>;

/// An owned pointer stored in R's external pointer SEXP.
///
/// This is conceptually similar to `Box<T>`, but with the following differences:
/// - Memory is freed by R's GC via a registered finalizer (non-deterministic)
/// - The underlying SEXP is Copy, so aliasing must be manually prevented
/// - Type checking happens at runtime via StableTypeId stored in the prot slot
///
/// # Thread Safety
///
/// `ExternalPtr` is `!Send` and `!Sync` because it wraps an R SEXP, and R's
/// runtime is single-threaded. Attempting to use an `ExternalPtr` from multiple
/// threads would be undefined behavior.
///
/// # Safety
///
/// The ExternalPtr assumes exclusive ownership of the underlying data.
/// Cloning the raw SEXP without proper handling will lead to double-free.
#[repr(C)]
pub struct ExternalPtr<T: TypedExternal> {
    sexp: SEXP,
    _marker: PhantomData<T>,
    /// Makes this type !Send and !Sync
    _unsend: PhantomUnsend,
}

impl<T: TypedExternal> ExternalPtr<T> {
    // =========================================================================
    // Constructors (Box-equivalent)
    // =========================================================================

    /// Allocates memory on the heap and places `x` into it.
    ///
    /// Equivalent to `Box::new`.
    #[inline]
    pub fn new(x: T) -> Self {
        let ptr = Box::into_raw(Box::new(x));
        unsafe { Self::from_raw(ptr) }
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
    ///
    /// Equivalent to `Box::from_raw`.
    #[inline]
    pub unsafe fn from_raw(raw: *mut T) -> Self {
        debug_assert!(
            !raw.is_null(),
            "ExternalPtr::from_raw received null pointer"
        );

        // Create the tag symbol for human-readable type identification
        let tag = unsafe { Rf_install(T::TYPE_NAME_CSTR.as_ptr().cast()) };

        // Create the prot VECSXP: [type_id_rawsxp, user_protected]
        let prot = unsafe { Rf_allocVector(SEXPTYPE::VECSXP, PROT_VEC_LEN) };
        unsafe { Rf_protect(prot) };

        // Create a RAWSXP containing the StableTypeId for fast type checking
        let type_id = T::TYPE_ID;
        let type_id_raw = unsafe { type_id.to_rawsxp() };
        unsafe { Rf_protect(type_id_raw) };

        // Store type ID in slot 0
        unsafe { SET_VECTOR_ELT(prot, PROT_TYPE_ID_INDEX, type_id_raw) };
        // Slot 1 (user protected) starts as R_NilValue (already default)

        // Create the external pointer with tag and prot
        let sexp = unsafe { R_MakeExternalPtr(raw.cast(), tag, prot) };
        unsafe { Rf_protect(sexp) };

        // Register the C finalizer that will call drop
        unsafe { R_RegisterCFinalizerEx(sexp, Some(release_raw::<T>), Rboolean::TRUE) };

        unsafe { Rf_unprotect(3) };

        Self {
            sexp,
            _marker: PhantomData,
            _unsend: PhantomData,
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

    /// Returns the protected SEXP slot (user-protected objects).
    ///
    /// This returns the user-protected object stored in the prot VECSXP,
    /// not the VECSXP itself.
    #[inline]
    pub fn protected(&self) -> SEXP {
        unsafe {
            let prot = R_ExternalPtrProtected(self.sexp);
            if prot.is_null() || prot == R_NilValue {
                return R_NilValue;
            }
            if TYPEOF(prot) != SEXPTYPE::VECSXP || Rf_xlength(prot) < PROT_VEC_LEN {
                return R_NilValue;
            }
            VECTOR_ELT(prot, PROT_USER_INDEX)
        }
    }

    /// Sets the user-protected SEXP slot.
    ///
    /// Use this to prevent R objects from being GC'd while this ExternalPtr exists.
    /// The type ID stored in prot slot 0 is preserved.
    ///
    /// Returns `false` if the prot structure is malformed (should not happen
    /// for ExternalPtrs created by this library).
    #[inline]
    pub unsafe fn set_protected(&self, user_prot: SEXP) -> bool {
        unsafe {
            let prot = R_ExternalPtrProtected(self.sexp);
            if prot.is_null() || prot == R_NilValue {
                debug_assert!(false, "ExternalPtr prot slot is null or R_NilValue");
                return false;
            }
            if TYPEOF(prot) != SEXPTYPE::VECSXP || Rf_xlength(prot) < PROT_VEC_LEN {
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

    /// Attempt to create an ExternalPtr from an SEXP with type checking.
    ///
    /// Returns `None` if:
    /// - The internal pointer is null
    /// - The `prot` slot doesn't contain a valid VECSXP with StableTypeId
    /// - The StableTypeId doesn't match T's type
    ///
    /// # Safety
    ///
    /// - `sexp` must be a valid EXTPTRSXP
    /// - The caller must ensure no other ExternalPtr owns this SEXP
    pub unsafe fn try_from_sexp(sexp: SEXP) -> Option<Self> {
        // Check if pointer is null
        let ptr = unsafe { R_ExternalPtrAddr(sexp) };
        if ptr.is_null() {
            return None;
        }

        // Extract prot VECSXP
        let prot = unsafe { R_ExternalPtrProtected(sexp) };
        if prot.is_null() || prot == unsafe { R_NilValue } {
            return None;
        }
        if unsafe { TYPEOF(prot) } != SEXPTYPE::VECSXP || unsafe { Rf_xlength(prot) } < PROT_VEC_LEN
        {
            return None;
        }

        // Extract StableTypeId RAWSXP from slot 0
        let type_id_raw = unsafe { VECTOR_ELT(prot, PROT_TYPE_ID_INDEX) };
        let stored_type_id = StableTypeId::from_rawsxp(type_id_raw)?;

        // Compare with expected type
        if stored_type_id != T::TYPE_ID {
            return None;
        }

        Some(Self {
            sexp,
            _marker: PhantomData,
            _unsend: PhantomData,
        })
    }

    /// Attempt to create an ExternalPtr, returning an error with type info on mismatch.
    ///
    /// # Safety
    ///
    /// Same as `try_from_sexp`.
    pub unsafe fn try_from_sexp_with_error(sexp: SEXP) -> Result<Self, TypeMismatchError> {
        // Check if pointer is null
        let ptr = unsafe { R_ExternalPtrAddr(sexp) };
        if ptr.is_null() {
            return Err(TypeMismatchError::NullPointer);
        }

        // Extract prot VECSXP
        let prot = unsafe { R_ExternalPtrProtected(sexp) };
        if prot.is_null() || prot == unsafe { R_NilValue } {
            return Err(TypeMismatchError::InvalidTypeId);
        }
        if unsafe { TYPEOF(prot) } != SEXPTYPE::VECSXP || unsafe { Rf_xlength(prot) } < PROT_VEC_LEN
        {
            return Err(TypeMismatchError::InvalidTypeId);
        }

        // Extract StableTypeId RAWSXP from slot 0
        let type_id_raw = unsafe { VECTOR_ELT(prot, PROT_TYPE_ID_INDEX) };
        let stored_type_id = match StableTypeId::from_rawsxp(type_id_raw) {
            Some(id) => id,
            None => return Err(TypeMismatchError::InvalidTypeId),
        };

        // Compare with expected type
        let expected = T::TYPE_ID;
        if stored_type_id != expected {
            return Err(TypeMismatchError::Mismatch {
                expected: expected.name(),
                found: stored_type_id.name(),
            });
        }

        Ok(Self {
            sexp,
            _marker: PhantomData,
            _unsend: PhantomData,
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
            _unsend: PhantomData,
        }
    }

    // =========================================================================
    // Downcast support
    // =========================================================================

    /// Returns the StableTypeId for type T.
    #[inline]
    pub fn type_id() -> StableTypeId {
        T::TYPE_ID
    }

    /// Returns the StableTypeId stored in this ExternalPtr's prot slot.
    ///
    /// Returns `None` if the prot slot doesn't contain a valid StableTypeId.
    #[inline]
    pub fn stored_type_id(&self) -> Option<StableTypeId> {
        unsafe {
            let prot = R_ExternalPtrProtected(self.sexp);
            if prot.is_null() || prot == R_NilValue {
                return None;
            }
            if TYPEOF(prot) != SEXPTYPE::VECSXP || Rf_xlength(prot) < PROT_VEC_LEN {
                return None;
            }
            let type_id_raw = VECTOR_ELT(prot, PROT_TYPE_ID_INDEX);
            StableTypeId::from_rawsxp(type_id_raw)
        }
    }
}

/// Error returned when type checking fails in `try_from_sexp_with_error`.
///
/// # Lifetime of `found` type name
///
/// The `found` field in `Mismatch` contains a `&'static str` that comes from
/// deserializing the `StableTypeId` stored in the SEXP. This works because
/// `std::any::type_name` returns `&'static str` pointing to static memory.
///
/// **Warning**: If an `ExternalPtr` is somehow serialized and deserialized
/// across process boundaries, the `name_ptr` in `StableTypeId` will be invalid.
/// This crate assumes same-process usage only.
#[derive(Debug, Clone)]
pub enum TypeMismatchError {
    /// The external pointer's address was null.
    NullPointer,
    /// The prot slot didn't contain a valid StableTypeId.
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

// =============================================================================
// Erased ExternalPtr (for downcasting)
// =============================================================================

/// A type-erased external pointer.
///
/// This is useful when you need to store external pointers of different types
/// in a collection, or when interfacing with R where the type is not known
/// at compile time.
///
/// # Thread Safety
///
/// Like `ExternalPtr`, this is `!Send` and `!Sync` because it wraps an R SEXP.
pub struct ErasedExternalPtr {
    sexp: SEXP,
    /// Makes this type !Send and !Sync
    _unsend: PhantomUnsend,
}

impl ErasedExternalPtr {
    /// Create an erased pointer from any ExternalPtr.
    #[inline]
    pub fn new<T: TypedExternal>(ptr: ExternalPtr<T>) -> Self {
        let sexp = ptr.sexp;
        mem::forget(ptr); // Don't run ExternalPtr's drop
        Self {
            sexp,
            _unsend: PhantomData,
        }
    }

    /// Create from a raw SEXP.
    ///
    /// # Safety
    ///
    /// The SEXP must be a valid EXTPTRSXP with a StableTypeId in prot.
    #[inline]
    pub unsafe fn from_sexp(sexp: SEXP) -> Self {
        Self {
            sexp,
            _unsend: PhantomData,
        }
    }

    /// Get the underlying SEXP.
    #[inline]
    pub fn as_sexp(&self) -> SEXP {
        self.sexp
    }

    /// Get the stored type ID, if valid.
    #[inline]
    pub fn type_id(&self) -> Option<StableTypeId> {
        unsafe {
            let prot = R_ExternalPtrProtected(self.sexp);
            if prot.is_null() || prot == R_NilValue {
                return None;
            }
            if TYPEOF(prot) != SEXPTYPE::VECSXP || Rf_xlength(prot) < PROT_VEC_LEN {
                return None;
            }
            let type_id_raw = VECTOR_ELT(prot, PROT_TYPE_ID_INDEX);
            StableTypeId::from_rawsxp(type_id_raw)
        }
    }

    /// Returns the user-protected SEXP slot.
    #[inline]
    pub fn protected(&self) -> SEXP {
        unsafe {
            let prot = R_ExternalPtrProtected(self.sexp);
            if prot.is_null() || prot == R_NilValue {
                return R_NilValue;
            }
            if TYPEOF(prot) != SEXPTYPE::VECSXP || Rf_xlength(prot) < PROT_VEC_LEN {
                return R_NilValue;
            }
            VECTOR_ELT(prot, PROT_USER_INDEX)
        }
    }

    /// Sets the user-protected SEXP slot.
    ///
    /// Returns `false` if the prot structure is malformed.
    #[inline]
    pub unsafe fn set_protected(&self, user_prot: SEXP) -> bool {
        unsafe {
            let prot = R_ExternalPtrProtected(self.sexp);
            if prot.is_null() || prot == R_NilValue {
                return false;
            }
            if TYPEOF(prot) != SEXPTYPE::VECSXP || Rf_xlength(prot) < PROT_VEC_LEN {
                return false;
            }
            SET_VECTOR_ELT(prot, PROT_USER_INDEX, user_prot);
            true
        }
    }

    /// Check if this pointer holds type T.
    #[inline]
    pub fn is<T: TypedExternal>(&self) -> bool {
        self.type_id().map(|id| id == T::TYPE_ID).unwrap_or(false)
    }

    /// Attempt to downcast to a concrete ExternalPtr type.
    ///
    /// Returns `Err(self)` if the type doesn't match.
    #[inline]
    pub fn downcast<T: TypedExternal>(self) -> Result<ExternalPtr<T>, Self> {
        if self.is::<T>() {
            let sexp = self.sexp;
            Ok(ExternalPtr {
                sexp,
                _marker: PhantomData,
                _unsend: PhantomData,
            })
        } else {
            Err(self)
        }
    }

    /// Attempt to get a reference to the inner value as type T.
    #[inline]
    pub fn downcast_ref<T: TypedExternal>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe {
                let ptr = R_ExternalPtrAddr(self.sexp).cast::<T>();
                if ptr.is_null() { None } else { Some(&*ptr) }
            }
        } else {
            None
        }
    }

    /// Attempt to get a mutable reference to the inner value as type T.
    #[inline]
    pub fn downcast_mut<T: TypedExternal>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            unsafe {
                let ptr = R_ExternalPtrAddr(self.sexp).cast::<T>();
                if ptr.is_null() { None } else { Some(&mut *ptr) }
            }
        } else {
            None
        }
    }
}

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
    if std::ptr::addr_eq(sexp, unsafe { R_NilValue }) {
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

    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

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
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stable_type_id() {
        let id1 = StableTypeId::of::<i32>();
        let id2 = StableTypeId::of::<i32>();
        let id3 = StableTypeId::of::<u32>();

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert_eq!(id1.name(), "i32");
    }

    #[test]
    fn test_const_hash() {
        let h1 = const_hash_str("hello");
        let h2 = const_hash_str("hello");
        let h3 = const_hash_str("world");

        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }
}
