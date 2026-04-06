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

// region: Blanket implementations for TinyVec and ArrayVec
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
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.as_slice().into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        Ok(unsafe { self.as_slice().into_sexp_unchecked() })
    }
}

/// Blanket impl for `IntoR` on `ArrayVec<[T; N]>` where T: RNativeType
impl<T, const N: usize> IntoR for ArrayVec<[T; N]>
where
    T: RNativeType + Copy,
    [T; N]: tinyvec::Array<Item = T>,
{
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.as_slice().into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        Ok(unsafe { self.as_slice().into_sexp_unchecked() })
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
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(match self {
            Some(tv) => tv.into_sexp(),
            None => crate::ffi::SEXP::nil(),
        })
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        Ok(match self {
            Some(tv) => unsafe { tv.into_sexp_unchecked() },
            None => crate::ffi::SEXP::nil(),
        })
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
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(match self {
            Some(av) => av.into_sexp(),
            None => crate::ffi::SEXP::nil(),
        })
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        Ok(match self {
            Some(av) => unsafe { av.into_sexp_unchecked() },
            None => crate::ffi::SEXP::nil(),
        })
    }
}
// endregion

// region: Coerced element support
//
// Support for `TinyVec<[Coerced<T, R>; N]>` and `ArrayVec<[Coerced<T, R>; N]>`.
// This allows reading R native types (i32, f64) and coercing to non-native types
// (i8, f32, etc.) element-wise.
//
// Example: `TinyVec<[Coerced<i8, i32>; 8]>` reads from R integer and coerces each
// element to i8.

use crate::coerce::{Coerced, TryCoerce};

/// Helper to coerce a slice element-wise into a container.
fn coerce_slice_to_vec<R, T>(slice: &[R]) -> Result<Vec<Coerced<T, R>>, SexpError>
where
    R: Copy + TryCoerce<T>,
    <R as TryCoerce<T>>::Error: std::fmt::Debug,
{
    slice
        .iter()
        .copied()
        .map(|v| {
            v.try_coerce()
                .map(Coerced::new)
                .map_err(|e| SexpError::InvalidValue(format!("{e:?}")))
        })
        .collect()
}

/// TryFromSexp for `TinyVec<[Coerced<T, R>; N]>` - reads R native type and coerces.
impl<T, R, const N: usize> TryFromSexp for TinyVec<[Coerced<T, R>; N]>
where
    R: RNativeType + Copy + TryCoerce<T>,
    T: Copy,
    <R as TryCoerce<T>>::Error: std::fmt::Debug,
    [Coerced<T, R>; N]: tinyvec::Array<Item = Coerced<T, R>>,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != R::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: R::SEXP_TYPE,
                actual,
            }
            .into());
        }
        let slice: &[R] = unsafe { sexp.as_slice() };
        let data: Vec<Coerced<T, R>> = coerce_slice_to_vec(slice)?;
        let mut tv = TinyVec::new();
        for item in data {
            tv.push(item);
        }
        Ok(tv)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

/// TryFromSexp for `ArrayVec<[Coerced<T, R>; N]>` - reads R native type and coerces.
impl<T, R, const N: usize> TryFromSexp for ArrayVec<[Coerced<T, R>; N]>
where
    R: RNativeType + Copy + TryCoerce<T>,
    T: Copy,
    <R as TryCoerce<T>>::Error: std::fmt::Debug,
    [Coerced<T, R>; N]: tinyvec::Array<Item = Coerced<T, R>>,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != R::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: R::SEXP_TYPE,
                actual,
            }
            .into());
        }
        let slice: &[R] = unsafe { sexp.as_slice() };
        if slice.len() > N {
            return Err(SexpError::InvalidValue(format!(
                "R vector length {} exceeds ArrayVec capacity {}",
                slice.len(),
                N
            )));
        }
        let data: Vec<Coerced<T, R>> = coerce_slice_to_vec(slice)?;
        let mut av = ArrayVec::new();
        for item in data {
            av.push(item);
        }
        Ok(av)
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

/// IntoR for `TinyVec<[Coerced<T, R>; N]>` - coerces back and writes to R.
impl<T, R, const N: usize> IntoR for TinyVec<[Coerced<T, R>; N]>
where
    T: Copy + Into<R>,
    R: RNativeType + Copy,
    [Coerced<T, R>; N]: tinyvec::Array<Item = Coerced<T, R>>,
{
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        let r_values: Vec<R> = self.into_iter().map(|c| (*c).into()).collect();
        Ok(r_values.into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        let r_values: Vec<R> = self.into_iter().map(|c| (*c).into()).collect();
        Ok(unsafe { r_values.into_sexp_unchecked() })
    }
}

/// IntoR for `ArrayVec<[Coerced<T, R>; N]>` - coerces back and writes to R.
impl<T, R, const N: usize> IntoR for ArrayVec<[Coerced<T, R>; N]>
where
    T: Copy + Into<R>,
    R: RNativeType + Copy,
    [Coerced<T, R>; N]: tinyvec::Array<Item = Coerced<T, R>>,
{
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        let r_values: Vec<R> = self.into_iter().map(|c| (*c).into()).collect();
        Ok(r_values.into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        let r_values: Vec<R> = self.into_iter().map(|c| (*c).into()).collect();
        Ok(unsafe { r_values.into_sexp_unchecked() })
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
// endregion
