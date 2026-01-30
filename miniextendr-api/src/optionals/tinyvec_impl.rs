//! Integration with the `tinyvec` crate.
//!
//! This module provides [`TryFromSexp`] and [`IntoR`] implementations for
//! [`TinyVec`] and [`ArrayVec`], allowing small-vector optimized types to be
//! passed between R and Rust.
//!
//! # Features
//!
//! Enable this module with the `tinyvec` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["tinyvec"] }
//! ```
//!
//! # Supported Types
//!
//! | Type | Description |
//! |------|-------------|
//! | `TinyVec<[T; N]>` | Small, growable vector (inline + heap fallback) |
//! | `ArrayVec<[T; N]>` | Fixed-capacity inline vector |
//!
//! # Element Types
//!
//! Supports native R types that can be efficiently bulk-copied:
//! - `i32` (R integer)
//! - `f64` (R numeric/double)
//! - `u8` (R raw)
//! - `RLogical` (R logical)
//!
//! For String vectors, use `Vec<String>` instead - the overhead of small-vector
//! optimization is negligible compared to string allocation costs.
//!
//! # Conversion Behavior
//!
//! ## From R to Rust (`TryFromSexp`)
//!
//! - `TinyVec<[T; N]>`: Converts via slice copy into TinyVec. Values
//!   will be stored inline if `len <= N`, otherwise on the heap.
//! - `ArrayVec<[T; N]>`: Converts via slice copy, checks `len <= N`. Returns
//!   an error if the R vector exceeds the array capacity.
//!
//! ## From Rust to R (`IntoR`)
//!
//! Uses `as_slice()` for efficient bulk copy to R vectors.
//!
//! # Examples
//!
//! ```ignore
//! use tinyvec::{ArrayVec, TinyVec};
//!
//! #[miniextendr]
//! fn sum_small(x: TinyVec<[i32; 8]>) -> i32 {
//!     x.into_iter().sum()
//! }
//!
//! #[miniextendr]
//! fn top3(x: ArrayVec<[f64; 3]>) -> ArrayVec<[f64; 3]> {
//!     // Returns error if input has more than 3 elements
//!     x
//! }
//! ```

pub use tinyvec::{Array, ArrayVec, TinyVec};

use crate::ffi::{RNativeType, SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, SexpTypeError, TryFromSexp};
use crate::into_r::IntoR;

// =============================================================================
// Blanket implementations for TinyVec and ArrayVec
// =============================================================================
//
// Now that we have blanket impls for `&[T]` where T: RNativeType, we can write
// blanket impls for containers instead of using macros. This provides maximum
// composability - any type implementing RNativeType automatically works with
// TinyVec and ArrayVec.

/// Blanket impl for `TinyVec<[T; N]>` where T: RNativeType
impl<T, const N: usize> TryFromSexp for TinyVec<[T; N]>
where
    T: RNativeType + Copy,
    [T; N]: tinyvec::Array<Item = T>,
{
    type Error = SexpTypeError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = TryFromSexp::try_from_sexp(sexp)?;
        let mut tv = TinyVec::new();
        tv.extend_from_slice(slice);
        Ok(tv)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        let mut tv = TinyVec::new();
        tv.extend_from_slice(slice);
        Ok(tv)
    }
}

/// Blanket impl for `ArrayVec<[T; N]>` where T: RNativeType
impl<T, const N: usize> TryFromSexp for ArrayVec<[T; N]>
where
    T: RNativeType + Copy,
    [T; N]: tinyvec::Array<Item = T>,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = TryFromSexp::try_from_sexp(sexp)?;
        if slice.len() > N {
            return Err(SexpError::InvalidValue(format!(
                "R vector length {} exceeds ArrayVec capacity {}",
                slice.len(),
                N
            )));
        }
        let mut av = ArrayVec::new();
        av.extend_from_slice(slice);
        Ok(av)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        if slice.len() > N {
            return Err(SexpError::InvalidValue(format!(
                "R vector length {} exceeds ArrayVec capacity {}",
                slice.len(),
                N
            )));
        }
        let mut av = ArrayVec::new();
        av.extend_from_slice(slice);
        Ok(av)
    }
}

/// Blanket impl for `IntoR` on `TinyVec<[T; N]>` where T: RNativeType
impl<T, const N: usize> IntoR for TinyVec<[T; N]>
where
    T: RNativeType + Copy,
    [T; N]: tinyvec::Array<Item = T>,
{
    #[inline]
    fn into_sexp(self) -> SEXP {
        self.as_slice().into_sexp()
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> SEXP {
        unsafe { self.as_slice().into_sexp_unchecked() }
    }
}

/// Blanket impl for `IntoR` on `ArrayVec<[T; N]>` where T: RNativeType
impl<T, const N: usize> IntoR for ArrayVec<[T; N]>
where
    T: RNativeType + Copy,
    [T; N]: tinyvec::Array<Item = T>,
{
    #[inline]
    fn into_sexp(self) -> SEXP {
        self.as_slice().into_sexp()
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> SEXP {
        unsafe { self.as_slice().into_sexp_unchecked() }
    }
}

/// Blanket impl for `Option<TinyVec<[T; N]>>` where T: RNativeType
impl<T, const N: usize> TryFromSexp for Option<TinyVec<[T; N]>>
where
    T: RNativeType + Copy,
    [T; N]: tinyvec::Array<Item = T>,
{
    type Error = SexpTypeError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        TinyVec::try_from_sexp(sexp).map(Some)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        unsafe { TinyVec::try_from_sexp_unchecked(sexp) }.map(Some)
    }
}

/// Blanket impl for `IntoR` on `Option<TinyVec<[T; N]>>` where T: RNativeType
impl<T, const N: usize> IntoR for Option<TinyVec<[T; N]>>
where
    T: RNativeType + Copy,
    [T; N]: tinyvec::Array<Item = T>,
{
    #[inline]
    fn into_sexp(self) -> SEXP {
        match self {
            Some(tv) => tv.into_sexp(),
            None => unsafe { crate::ffi::R_NilValue },
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> SEXP {
        match self {
            Some(tv) => unsafe { tv.into_sexp_unchecked() },
            None => unsafe { crate::ffi::R_NilValue },
        }
    }
}

/// Blanket impl for `Option<ArrayVec<[T; N]>>` where T: RNativeType
impl<T, const N: usize> TryFromSexp for Option<ArrayVec<[T; N]>>
where
    T: RNativeType + Copy,
    [T; N]: tinyvec::Array<Item = T>,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        ArrayVec::try_from_sexp(sexp).map(Some)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        unsafe { ArrayVec::try_from_sexp_unchecked(sexp) }.map(Some)
    }
}

/// Blanket impl for `IntoR` on `Option<ArrayVec<[T; N]>>` where T: RNativeType
impl<T, const N: usize> IntoR for Option<ArrayVec<[T; N]>>
where
    T: RNativeType + Copy,
    [T; N]: tinyvec::Array<Item = T>,
{
    #[inline]
    fn into_sexp(self) -> SEXP {
        match self {
            Some(av) => av.into_sexp(),
            None => unsafe { crate::ffi::R_NilValue },
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> SEXP {
        match self {
            Some(av) => unsafe { av.into_sexp_unchecked() },
            None => unsafe { crate::ffi::R_NilValue },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tinyvec_can_be_created() {
        let _tv: TinyVec<[i32; 4]> = TinyVec::new();
        let _av: ArrayVec<[f64; 8]> = ArrayVec::new();
    }

    #[test]
    fn tinyvec_from_slice() {
        let data = [1i32, 2, 3];
        let mut tv: TinyVec<[i32; 4]> = TinyVec::new();
        tv.extend_from_slice(&data);
        assert_eq!(tv.len(), 3);
        assert!(tv.is_inline()); // Should be stored inline since len <= 4
    }

    #[test]
    fn arrayvec_capacity_check() {
        let mut av: ArrayVec<[i32; 3]> = ArrayVec::new();
        av.push(1);
        av.push(2);
        av.push(3);
        assert_eq!(av.len(), 3);
        // Cannot push more - would panic
    }
}
