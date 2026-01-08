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

// =============================================================================
// Logical value type
// =============================================================================

/// Logical value: TRUE, FALSE, or NA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Logical {
    False,
    True,
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

// =============================================================================
// Sortedness hint
// =============================================================================

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

// =============================================================================
// Dataptr / serialization / subset helpers
// =============================================================================

/// Trait for ALTREP types that can expose a data pointer.
pub trait AltrepDataptr<T> {
    /// Get a mutable pointer to the underlying data.
    ///
    /// If `writable` is true, R may modify the data.
    /// Return `None` if data cannot be accessed as a contiguous buffer.
    ///
    /// This method may trigger materialization of lazy data.
    fn dataptr(&mut self, writable: bool) -> Option<*mut T>;

    /// Get a read-only pointer without forcing materialization.
    ///
    /// Return `None` if data is not already materialized or cannot provide
    /// a contiguous buffer. R will fall back to element-by-element access
    /// via `Elt` when this returns `None`.
    fn dataptr_or_null(&self) -> Option<*const T> {
        None
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

// =============================================================================
// InferBase trait - automatic base type inference from data traits
// =============================================================================

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
