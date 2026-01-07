use super::{AltrepLen, Logical, Sortedness, fill_region};
use crate::ffi::{Rcomplex, SEXP};

// =============================================================================
// Integer ALTREP
// =============================================================================

/// Trait for types that can back an ALTINTEGER vector.
///
/// Implement this to create custom integer ALTREP classes.
pub trait AltIntegerData: AltrepLen {
    /// Get the integer element at index `i`.
    fn elt(&self, i: usize) -> i32;

    /// Optional: return a pointer to contiguous data if available.
    /// Default returns None (no contiguous backing).
    fn as_slice(&self) -> Option<&[i32]> {
        None
    }

    /// Optional: bulk read into buffer. Returns number of elements read.
    ///
    /// Bounds are clamped to the vector length; see `fill_region` for the
    /// shared safety contract.
    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }

    /// Optional: sortedness hint. Default is unknown.
    fn is_sorted(&self) -> Option<Sortedness> {
        None
    }

    /// Optional: does this vector contain any NA values?
    fn no_na(&self) -> Option<bool> {
        None
    }

    /// Optional: optimized sum. Default returns None (use R's default).
    fn sum(&self, _na_rm: bool) -> Option<i64> {
        None
    }

    /// Optional: optimized min. Default returns None (use R's default).
    fn min(&self, _na_rm: bool) -> Option<i32> {
        None
    }

    /// Optional: optimized max. Default returns None (use R's default).
    fn max(&self, _na_rm: bool) -> Option<i32> {
        None
    }
}

// =============================================================================
// Real ALTREP
// =============================================================================

/// Trait for types that can back an ALTREAL vector.
pub trait AltRealData: AltrepLen {
    /// Get the real element at index `i`.
    fn elt(&self, i: usize) -> f64;

    /// Optional: return a pointer to contiguous data if available.
    fn as_slice(&self) -> Option<&[f64]> {
        None
    }

    /// Optional: bulk read into buffer (clamped to available data).
    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }

    /// Optional: sortedness hint.
    fn is_sorted(&self) -> Option<Sortedness> {
        None
    }

    /// Optional: does this vector contain any NA values?
    fn no_na(&self) -> Option<bool> {
        None
    }

    /// Optional: optimized sum.
    fn sum(&self, _na_rm: bool) -> Option<f64> {
        None
    }

    /// Optional: optimized min.
    fn min(&self, _na_rm: bool) -> Option<f64> {
        None
    }

    /// Optional: optimized max.
    fn max(&self, _na_rm: bool) -> Option<f64> {
        None
    }
}

// =============================================================================
// Logical ALTREP
// =============================================================================

/// Trait for types that can back an ALTLOGICAL vector.
pub trait AltLogicalData: AltrepLen {
    /// Get the logical element at index `i`.
    fn elt(&self, i: usize) -> Logical;

    /// Optional: return a slice if data is contiguous i32 (R's internal format).
    fn as_r_slice(&self) -> Option<&[i32]> {
        None
    }

    /// Optional: bulk read into buffer (clamped to available data).
    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx).to_r_int())
    }

    /// Optional: sortedness hint.
    fn is_sorted(&self) -> Option<Sortedness> {
        None
    }

    /// Optional: does this vector contain any NA values?
    fn no_na(&self) -> Option<bool> {
        None
    }

    /// Optional: optimized sum (count of TRUE values).
    fn sum(&self, _na_rm: bool) -> Option<i64> {
        None
    }
    // Note: R's ALTREP API does not expose min/max for logical vectors
}

// =============================================================================
// Raw ALTREP
// =============================================================================

/// Trait for types that can back an ALTRAW vector.
pub trait AltRawData: AltrepLen {
    /// Get the raw byte at index `i`.
    fn elt(&self, i: usize) -> u8;

    /// Optional: return a slice if data is contiguous.
    fn as_slice(&self) -> Option<&[u8]> {
        None
    }

    /// Optional: bulk read into buffer (clamped to available data).
    fn get_region(&self, start: usize, len: usize, buf: &mut [u8]) -> usize {
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }
}

// =============================================================================
// Complex ALTREP
// =============================================================================

/// Trait for types that can back an ALTCOMPLEX vector.
pub trait AltComplexData: AltrepLen {
    /// Get the complex element at index `i`.
    fn elt(&self, i: usize) -> Rcomplex;

    /// Optional: return a slice if data is contiguous.
    fn as_slice(&self) -> Option<&[Rcomplex]> {
        None
    }

    /// Optional: bulk read into buffer (clamped to available data).
    fn get_region(&self, start: usize, len: usize, buf: &mut [Rcomplex]) -> usize {
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }
}

// =============================================================================
// String ALTREP
// =============================================================================

/// Trait for types that can back an ALTSTRING vector.
///
/// Note: `elt` returns a `&str` which will be converted to CHARSXP.
pub trait AltStringData: AltrepLen {
    /// Get the string element at index `i`.
    ///
    /// Return `None` for NA values.
    fn elt(&self, i: usize) -> Option<&str>;

    /// Optional: sortedness hint.
    fn is_sorted(&self) -> Option<Sortedness> {
        None
    }

    /// Optional: does this vector contain any NA values?
    fn no_na(&self) -> Option<bool> {
        None
    }
}

// =============================================================================
// List ALTREP
// =============================================================================

/// Trait for types that can back an ALTLIST vector.
///
/// List elements are arbitrary SEXPs, so this trait works with raw SEXP.
pub trait AltListData: AltrepLen {
    /// Get the list element at index `i`.
    ///
    /// Returns a SEXP (any R object).
    fn elt(&self, i: usize) -> SEXP;
}
