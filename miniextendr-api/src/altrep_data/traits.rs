//! Per-family ALTREP data traits.
//!
//! Each ALTREP family has a high-level data trait that users implement:
//!
//! | Trait | R Type | Key Method |
//! |-------|--------|-----------|
//! | [`AltIntegerData`] | INTSXP | `elt(i) -> i32` |
//! | [`AltRealData`] | REALSXP | `elt(i) -> f64` |
//! | [`AltLogicalData`] | LGLSXP | `elt(i) -> Logical` |
//! | [`AltRawData`] | RAWSXP | `elt(i) -> u8` |
//! | [`AltComplexData`] | CPLXSXP | `elt(i) -> Rcomplex` |
//! | [`AltStringData`] | STRSXP | `elt(i) -> Option<&str>` |
//! | [`AltListData`] | VECSXP | `elt(i) -> SEXP` |

use super::{AltrepLen, Logical, Sortedness, fill_region};
use crate::{Rcomplex, SEXP};

// region: Integer ALTREP

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
    ///
    /// R calls this when scanning the vector in bulk (`sum()`, `anyNA()`,
    /// `duplicated()`, printing, ...), usually in blocks of up to 512
    /// elements via `ITERATE_BY_REGION`, but occasionally with `len` equal
    /// to the full vector length — never assume a block size. Calls are
    /// synchronous pulls on the R main thread; a class cannot schedule or
    /// prefetch them.
    ///
    /// The default fills element-by-element from [`elt`](Self::elt), which
    /// already costs only one FFI dispatch per block. Override it when the
    /// block has structure a per-element accessor would recompute (hoistable
    /// per-block invariants, contiguous sub-ranges, chunked decoding).
    ///
    /// # Contract
    ///
    /// Fill at most `len.min(buf.len()).min(self.len() - start)` elements
    /// starting at `start`, and return the count actually written. A `start`
    /// at or past the end returns `0`.
    ///
    /// # Examples
    ///
    /// A lazy arithmetic sequence with a bulk fill that hoists the block
    /// base out of the loop:
    ///
    /// ```
    /// use miniextendr_api::altrep_data::{AltIntegerData, AltrepLen};
    ///
    /// /// Lazy arithmetic sequence: element i is `start + i * step`.
    /// struct LazySeq { start: i32, step: i32, len: usize }
    ///
    /// impl AltrepLen for LazySeq {
    ///     fn len(&self) -> usize { self.len }
    /// }
    ///
    /// impl AltIntegerData for LazySeq {
    ///     fn elt(&self, i: usize) -> i32 {
    ///         self.start + (i as i32) * self.step
    ///     }
    ///
    ///     /// Bulk fill: clamp once, hoist the block base out of the loop,
    ///     /// then write `buf` sequentially (cache-friendly, no per-element
    ///     /// bounds or dispatch overhead).
    ///     fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
    ///         let n = len.min(buf.len()).min(self.len.saturating_sub(start));
    ///         let base = self.start + (start as i32) * self.step;
    ///         for (k, slot) in buf[..n].iter_mut().enumerate() {
    ///             *slot = base + (k as i32) * self.step;
    ///         }
    ///         n
    ///     }
    /// }
    ///
    /// let seq = LazySeq { start: 10, step: 2, len: 100 };
    /// let mut buf = [0i32; 8];
    ///
    /// // Interior block: fully filled.
    /// assert_eq!(seq.get_region(5, 8, &mut buf), 8);
    /// assert_eq!(buf, [20, 22, 24, 26, 28, 30, 32, 34]);
    ///
    /// // Tail block: clamped to the vector length.
    /// assert_eq!(seq.get_region(96, 8, &mut buf), 4);
    /// assert_eq!(&buf[..4], &[202, 204, 206, 208]);
    ///
    /// // Past the end: nothing written.
    /// assert_eq!(seq.get_region(100, 8, &mut buf), 0);
    /// ```
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
// endregion

// region: Real ALTREP

/// Trait for types that can back an ALTREAL vector.
pub trait AltRealData: AltrepLen {
    /// Get the real element at index `i`.
    fn elt(&self, i: usize) -> f64;

    /// Optional: return a pointer to contiguous data if available.
    fn as_slice(&self) -> Option<&[f64]> {
        None
    }

    /// Optional: bulk read into buffer (clamped to available data).
    ///
    /// Same contract and override guidance as
    /// [`AltIntegerData::get_region`]: fill at most
    /// `len.min(buf.len()).min(self.len() - start)` elements and return the
    /// count written.
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
// endregion

// region: Logical ALTREP

/// Trait for types that can back an ALTLOGICAL vector.
pub trait AltLogicalData: AltrepLen {
    /// Get the logical element at index `i`.
    fn elt(&self, i: usize) -> Logical;

    /// Optional: return a slice if data is contiguous i32 (R's internal format).
    fn as_r_slice(&self) -> Option<&[i32]> {
        None
    }

    /// Optional: bulk read into buffer (clamped to available data).
    ///
    /// The buffer is R's native logical storage (`i32`: 0, 1, or
    /// `NA_LOGICAL`). Same contract and override guidance as
    /// [`AltIntegerData::get_region`].
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
// endregion

// region: Raw ALTREP

/// Trait for types that can back an ALTRAW vector.
pub trait AltRawData: AltrepLen {
    /// Get the raw byte at index `i`.
    fn elt(&self, i: usize) -> u8;

    /// Optional: return a slice if data is contiguous.
    fn as_slice(&self) -> Option<&[u8]> {
        None
    }

    /// Optional: bulk read into buffer (clamped to available data).
    ///
    /// Same contract and override guidance as
    /// [`AltIntegerData::get_region`].
    fn get_region(&self, start: usize, len: usize, buf: &mut [u8]) -> usize {
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }
}
// endregion

// region: Complex ALTREP

/// Trait for types that can back an ALTCOMPLEX vector.
pub trait AltComplexData: AltrepLen {
    /// Get the complex element at index `i`.
    fn elt(&self, i: usize) -> Rcomplex;

    /// Optional: return a slice if data is contiguous.
    fn as_slice(&self) -> Option<&[Rcomplex]> {
        None
    }

    /// Optional: bulk read into buffer (clamped to available data).
    ///
    /// Same contract and override guidance as
    /// [`AltIntegerData::get_region`].
    fn get_region(&self, start: usize, len: usize, buf: &mut [Rcomplex]) -> usize {
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }
}
// endregion

// region: String ALTREP

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
// endregion

// region: List ALTREP

/// Trait for types that can back an ALTLIST vector.
///
/// List elements are arbitrary SEXPs, so this trait works with raw SEXP.
pub trait AltListData: AltrepLen {
    /// Get the list element at index `i`.
    ///
    /// Returns a SEXP (any R object).
    fn elt(&self, i: usize) -> SEXP;
}
// endregion
