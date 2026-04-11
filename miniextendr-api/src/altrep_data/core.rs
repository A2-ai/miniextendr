//! Core ALTREP data traits and helpers.
//!
//! Defines shared infrastructure used by all ALTREP families:
//!
//! - [`AltrepLen`] — length query trait
//! - [`AltrepDataptr`] / [`AltrepSerialize`] / [`AltrepExtractSubset`] — optional capabilities
//! - [`InferBase`] — maps data types to their R base type + class registration
//! - [`Sortedness`] — sort-order metadata constants
//! - [`Logical`] — three-valued logical (TRUE/FALSE/NA)
//! - [`fill_region`] — helper for `get_region` implementations

use crate::altrep_traits::{
    KNOWN_UNSORTED, SORTED_DECR, SORTED_DECR_NA_1ST, SORTED_INCR, SORTED_INCR_NA_1ST,
    UNKNOWN_SORTEDNESS,
};
use crate::ffi::SEXP;

/// Helper for ALTREP `get_region` implementations.
///
/// R guarantees that the caller-provided buffer is at least `len` long. This
/// helper clamps the requested range to the vector's total length and the
/// actual buffer length, then fills `out` with values from the provided
/// element accessor.
#[inline]
pub(crate) fn fill_region<T>(
    start: usize,
    len: usize,
    total_len: usize,
    out: &mut [T],
    mut elt: impl FnMut(usize) -> T,
) -> usize {
    let n = len.min(out.len()).min(total_len.saturating_sub(start));
    for (i, slot) in out.iter_mut().enumerate().take(n) {
        *slot = elt(start + i);
    }
    n
}

/// Base trait for ALTREP data types. All ALTREP types must provide length.
pub trait AltrepLen {
    /// Returns the length of this ALTREP vector.
    fn len(&self) -> usize;

    /// Returns true if the vector is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// region: Logical value type

/// Logical value: TRUE, FALSE, or NA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Logical {
    /// Logical false.
    False,
    /// Logical true.
    True,
    /// Missing logical value.
    Na,
}

impl Logical {
    /// Convert to R's integer representation.
    #[inline]
    pub fn to_r_int(self) -> i32 {
        self.into()
    }

    /// Convert from R's integer representation.
    #[inline]
    pub fn from_r_int(i: i32) -> Self {
        i.into()
    }

    /// Convert from Rust bool (no NA representation).
    #[inline]
    pub fn from_bool(b: bool) -> Self {
        b.into()
    }
}

/// Convert Logical to R's integer representation.
impl From<Logical> for i32 {
    fn from(logical: Logical) -> i32 {
        match logical {
            Logical::False => 0,
            Logical::True => 1,
            Logical::Na => i32::MIN,
        }
    }
}

/// Convert from R's integer representation to Logical.
impl From<i32> for Logical {
    fn from(i: i32) -> Self {
        match i {
            0 => Logical::False,
            i32::MIN => Logical::Na,
            _ => Logical::True,
        }
    }
}

/// Convert from Rust bool to Logical (no NA representation).
impl From<bool> for Logical {
    fn from(b: bool) -> Self {
        if b { Logical::True } else { Logical::False }
    }
}

/// Convert from RLogical (FFI type) to Logical (semantic type).
impl From<crate::ffi::RLogical> for Logical {
    fn from(r: crate::ffi::RLogical) -> Self {
        Logical::from_r_int(r.to_i32())
    }
}

/// Convert from Logical (semantic type) to RLogical (FFI type).
impl From<Logical> for crate::ffi::RLogical {
    fn from(l: Logical) -> Self {
        crate::ffi::RLogical::from_i32(l.to_r_int())
    }
}
// endregion

// region: Sortedness hint

/// Sortedness hint for ALTREP vectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sortedness {
    /// Unknown sortedness.
    Unknown,
    /// Known to be unsorted.
    ///
    /// This corresponds to `KNOWN_UNSORTED` in R.
    KnownUnsorted,
    /// Sorted in increasing order (may have ties).
    Increasing,
    /// Sorted in decreasing order (may have ties).
    Decreasing,
    /// Sorted in increasing order, with NAs first.
    ///
    /// This corresponds to `SORTED_INCR_NA_1ST` in R.
    IncreasingNaFirst,
    /// Sorted in decreasing order, with NAs first.
    ///
    /// This corresponds to `SORTED_DECR_NA_1ST` in R.
    DecreasingNaFirst,
}

impl Sortedness {
    /// Convert to R's integer representation.
    #[inline]
    pub fn to_r_int(self) -> i32 {
        self.into()
    }

    /// Convert from R's integer representation.
    #[inline]
    pub fn from_r_int(i: i32) -> Self {
        i.into()
    }
}

/// Convert Sortedness to R's integer representation.
impl From<Sortedness> for i32 {
    fn from(s: Sortedness) -> i32 {
        match s {
            Sortedness::Unknown => UNKNOWN_SORTEDNESS,
            Sortedness::KnownUnsorted => KNOWN_UNSORTED,
            Sortedness::Increasing => SORTED_INCR,
            Sortedness::Decreasing => SORTED_DECR,
            Sortedness::IncreasingNaFirst => SORTED_INCR_NA_1ST,
            Sortedness::DecreasingNaFirst => SORTED_DECR_NA_1ST,
        }
    }
}

/// Convert R's integer sortedness code to Sortedness.
impl From<i32> for Sortedness {
    fn from(i: i32) -> Self {
        match i {
            KNOWN_UNSORTED => Sortedness::KnownUnsorted,
            SORTED_INCR => Sortedness::Increasing,
            SORTED_DECR => Sortedness::Decreasing,
            SORTED_INCR_NA_1ST => Sortedness::IncreasingNaFirst,
            SORTED_DECR_NA_1ST => Sortedness::DecreasingNaFirst,
            _ => Sortedness::Unknown,
        }
    }
}
// endregion

// region: Dataptr / serialization / subset helpers

/// Trait for ALTREP types that can expose a data pointer.
///
/// # Writability contract
///
/// When `writable = true`, R **will** write through the returned pointer
/// (e.g., `x[i] <- val`). The implementation must ensure:
///
/// 1. The returned pointer is safe to write to (not read-only memory).
/// 2. Writes are visible to subsequent `Elt`/`Get_region` calls (no stale cache).
///
/// For owned containers (`Vec<T>`, `Box<[T]>`), this is automatic because
/// DATAPTR and Elt both access the same allocation (data1).
///
/// For copy-on-write types (`Cow<'static, [T]>`), `writable = true` should
/// trigger the copy so writes go to owned memory. When `writable = false`,
/// the borrowed pointer can be returned directly.
///
/// For immutable data (`&'static [T]`), `writable = true` should panic or
/// return `None` since the data cannot be modified.
///
/// The `__impl_altvec_dataptr` macro uses `dataptr_or_null` for read-only
/// access and only calls `dataptr(&mut self, true)` when R requests a
/// writable pointer.
pub trait AltrepDataptr<T> {
    /// Get a pointer to the underlying data, possibly triggering materialization.
    ///
    /// When `writable` is true, R will write through the returned pointer.
    /// Implementations for immutable data should panic or return `None`.
    ///
    /// Return `None` if data cannot be accessed as a contiguous buffer.
    fn dataptr(&mut self, writable: bool) -> Option<*mut T>;

    /// Get a read-only pointer without forcing materialization.
    ///
    /// Return `None` if data is not already materialized or cannot provide
    /// a contiguous buffer. R will fall back to element-by-element access
    /// via `Elt` when this returns `None`.
    ///
    /// The `__impl_altvec_dataptr` macro calls this for `Dataptr(x, writable=false)`
    /// to avoid unnecessary mutable borrows and copy-on-write overhead.
    fn dataptr_or_null(&self) -> Option<*const T> {
        None
    }
}

/// Materialize an ALTREP SEXP into a plain R vector in data2.
///
/// Called by `__impl_altvec_dataptr` when the custom `dataptr()` returns `None`.
/// Allocates a destination vector via `alloc_r_vector_unchecked`, fills it from
/// `T::elt()` (which goes through R's ALTREP Elt dispatch), stores in data2,
/// and returns DATAPTR of data2.
///
/// # Safety
/// - `x` must be a valid ALTREP SEXP of element type `T`
/// - Must be called on R's main thread
pub unsafe fn materialize_altrep_data2<T: crate::ffi::RNativeType>(
    x: SEXP,
) -> *mut core::ffi::c_void {
    use crate::altrep_ext::AltrepSexpExt;
    use crate::ffi::{self, SexpExt};

    let n = x.len();
    let (vec, dst) = unsafe { crate::into_r::alloc_r_vector_unchecked::<T>(n) };
    unsafe { ffi::Rf_protect_unchecked(vec) };
    for (i, slot) in dst.iter_mut().enumerate() {
        *slot = T::elt(x, i as isize);
    }

    unsafe {
        AltrepSexpExt::set_altrep_data2(&x, vec);
        ffi::Rf_unprotect_unchecked(1);
        ffi::DATAPTR_RO_unchecked(vec).cast_mut()
    }
}

/// Trait for ALTREP types that support serialization.
pub trait AltrepSerialize: Sized {
    /// Convert the ALTREP data to a serializable R object.
    fn serialized_state(&self) -> SEXP;

    /// Reconstruct the ALTREP data from a serialized state.
    ///
    /// Return `None` if the state is invalid or cannot be deserialized.
    fn unserialize(state: SEXP) -> Option<Self>;
}

/// Trait for ALTREP types that can provide optimized subsetting.
pub trait AltrepExtractSubset {
    /// Extract a subset of this ALTREP.
    ///
    /// `indices` contains 1-based R indices.
    /// Return `None` to fall back to R's default subsetting.
    fn extract_subset(&self, indices: &[i32]) -> Option<SEXP>;
}
// endregion

// region: AltrepExtract - how to get &Self from an ALTREP SEXP

/// How to extract a reference to `Self` from an ALTREP SEXP's data1 slot.
///
/// The default implementation (for types that implement `TypedExternal`) extracts
/// via `ExternalPtr<T>` downcast from data1. Power users who want native SEXP
/// storage can implement this trait manually.
///
/// # Safety
///
/// Implementations must ensure that the returned references are valid for the
/// duration of the ALTREP callback (i.e., the SEXP is protected by R's GC).
pub trait AltrepExtract: Sized {
    /// Extract a shared reference from the ALTREP data1 slot.
    ///
    /// # Safety
    ///
    /// - `x` must be a valid ALTREP SEXP whose data1 holds data of type `Self`
    /// - Must be called from R's main thread
    unsafe fn altrep_extract_ref(x: crate::ffi::SEXP) -> &'static Self;

    /// Extract a mutable reference from the ALTREP data1 slot.
    ///
    /// # Safety
    ///
    /// - `x` must be a valid ALTREP SEXP whose data1 holds data of type `Self`
    /// - Must be called from R's main thread
    /// - The caller must ensure no other references to the data exist
    unsafe fn altrep_extract_mut(x: crate::ffi::SEXP) -> &'static mut Self;
}

/// Blanket implementation for types stored in ExternalPtr (the common case).
///
/// This is the default storage strategy: data1 is an EXTPTRSXP wrapping a
/// `Box<Box<dyn Any>>` that downcasts to `&T`.
impl<T: crate::externalptr::TypedExternal> AltrepExtract for T {
    unsafe fn altrep_extract_ref(x: crate::ffi::SEXP) -> &'static Self {
        // Use the unchecked variant — ALTREP callbacks are always on R's main thread.
        // ExternalPtr wraps the SEXP and provides Deref<Target=T>.
        // We transmute the borrow to 'static since the ALTREP object is GC-protected
        // for the duration of the callback.
        unsafe {
            let ext = crate::altrep_data1_as_unchecked::<T>(x)
                .expect("ALTREP data1 ExternalPtr extraction failed");
            // ExternalPtr's Deref gives &T with the ExternalPtr's lifetime.
            // The ALTREP SEXP keeps the ExternalPtr alive, so 'static is sound
            // within the callback scope.
            &*(ext.as_ref().unwrap() as *const T)
        }
    }

    unsafe fn altrep_extract_mut(x: crate::ffi::SEXP) -> &'static mut Self {
        // SAFETY: caller guarantees x is a valid ALTREP with ExternalPtr<T> in data1
        // and that no other references exist. ALTREP callbacks are on R's main thread.
        unsafe {
            crate::altrep_data1_mut_unchecked::<T>(x)
                .expect("ALTREP data1 mutable ExternalPtr extraction failed")
        }
    }
}
// endregion

// region: InferBase trait - automatic base type inference from data traits

/// Trait for inferring the R base type from a data type's implemented traits.
///
/// This is automatically implemented via blanket impls for types that implement
/// one of the `Alt*Data` traits. It allows the `#[miniextendr]` macro to infer
/// the base type without requiring an explicit `base = \"...\"` attribute.
pub trait InferBase {
    /// The inferred R base type.
    const BASE: crate::altrep::RBase;

    /// Create the ALTREP class handle.
    ///
    /// # Safety
    /// Must be called during R initialization.
    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t;

    /// Install ALTREP methods on the class.
    ///
    /// # Safety
    /// Must be called during R initialization with a valid class handle.
    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t);
}
// endregion
