//! Integration with the `ndarray` crate.
//!
//! This module provides conversions between R arrays and ndarray types:
//!
//! | R Type | ndarray Type | Notes |
//! |--------|--------------|-------|
//! | vector | `Array1<T>` | 1D array |
//! | matrix | `Array2<T>` | 2D array |
//! | array (3D) | `Array3<T>` | 3D array |
//! | array (N-D) | `ArrayD<T>` | Dynamic dimension array |
//!
//! Supported element types: `i32`, `f64`, `u8` (raw), `RLogical`, `Rcomplex`.
//!
//! # Features
//!
//! Enable this module with the `ndarray` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["ndarray"] }
//! ```
//!
//! # Memory Layout
//!
//! R arrays are stored in **column-major** order (Fortran style).
//! ndarray's default is **row-major** (C style).
//!
//! When converting:
//! - **R → ndarray**: Data is copied and the array is created in **column-major**
//!   (Fortran) layout to match R's storage. This preserves the element ordering
//!   but means `array.is_standard_layout()` returns `false`.
//! - **ndarray → R**: Data is copied into column-major R storage.
//!
//! For zero-copy access, use `ArrayView` types:
//! - [`from_r_slice`] - 1D view
//! - [`from_r_matrix`] - 2D view
//! - [`from_r_array3`] - 3D view
//! - [`from_r_array`] - N-D view (dynamic)
//!
//! # Examples
//!
//! ```ignore
//! use ndarray::{Array1, Array2, Array3, ArrayD};
//!
//! #[miniextendr]
//! fn sum_vector(arr: Array1<f64>) -> f64 {
//!     arr.sum()
//! }
//!
//! #[miniextendr]
//! fn matrix_trace(mat: Array2<f64>) -> f64 {
//!     mat.diag().sum()
//! }
//!
//! #[miniextendr]
//! fn cube_sum(arr: Array3<f64>) -> f64 {
//!     arr.sum()
//! }
//!
//! #[miniextendr]
//! fn nd_array_shape(arr: ArrayD<f64>) -> Vec<i32> {
//!     arr.shape().iter().map(|&d| d as i32).collect()
//! }
//! ```

pub use ndarray::{
    // Shared ownership
    ArcArray1,
    ArcArray2,
    // Owned arrays (all fixed dimensions)
    Array0,
    Array1,
    Array2,
    Array3,
    Array4,
    Array5,
    Array6,
    ArrayD,
    // Read-only views
    ArrayView0,
    ArrayView1,
    ArrayView2,
    ArrayView3,
    ArrayView4,
    ArrayView5,
    ArrayView6,
    ArrayViewD,
    // Mutable views
    ArrayViewMut0,
    ArrayViewMut1,
    ArrayViewMut2,
    ArrayViewMut3,
    ArrayViewMut4,
    ArrayViewMut5,
    ArrayViewMut6,
    ArrayViewMutD,
    // Index types
    Ix0,
    Ix1,
    Ix2,
    Ix3,
    Ix4,
    Ix5,
    Ix6,
    IxDyn,
    // Shape builder
    ShapeBuilder,
};

use crate::coerce::TryCoerce;
use crate::ffi::{RLogical, RNativeType, SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, SexpLengthError, SexpTypeError, TryFromSexp};
use crate::gc_protect::{OwnedProtect, ProtectScope};
use crate::into_r::IntoR;

// =============================================================================
// R-native Array<T> conversions (explicit)
// =============================================================================

fn array0_from_sexp<T: RNativeType + Copy>(sexp: SEXP) -> Result<Array0<T>, SexpError> {
    let actual = sexp.type_of();
    if actual != T::SEXP_TYPE {
        return Err(SexpTypeError {
            expected: T::SEXP_TYPE,
            actual,
        }
        .into());
    }

    let len = sexp.len();
    if len != 1 {
        return Err(SexpLengthError {
            expected: 1,
            actual: len,
        }
        .into());
    }

    let slice: &[T] = unsafe { sexp.as_slice() };
    Ok(Array0::from_elem((), slice[0]))
}

fn array1_from_sexp<T: RNativeType + Copy>(sexp: SEXP) -> Result<Array1<T>, SexpError> {
    let actual = sexp.type_of();
    if actual != T::SEXP_TYPE {
        return Err(SexpTypeError {
            expected: T::SEXP_TYPE,
            actual,
        }
        .into());
    }

    let slice: &[T] = unsafe { sexp.as_slice() };
    Ok(Array1::from_vec(slice.to_vec()))
}

fn array2_from_sexp<T: RNativeType + Copy>(sexp: SEXP) -> Result<Array2<T>, SexpError> {
    let actual = sexp.type_of();
    if actual != T::SEXP_TYPE {
        return Err(SexpTypeError {
            expected: T::SEXP_TYPE,
            actual,
        }
        .into());
    }

    let (nrow, ncol) = get_matrix_dims(sexp)?;
    let slice: &[T] = unsafe { sexp.as_slice() };
    Array2::from_shape_vec((nrow, ncol).f(), slice.to_vec()).map_err(|_| {
        SexpLengthError {
            expected: nrow * ncol,
            actual: slice.len(),
        }
        .into()
    })
}

fn array3_from_sexp<T: RNativeType + Copy>(sexp: SEXP) -> Result<Array3<T>, SexpError> {
    let actual = sexp.type_of();
    if actual != T::SEXP_TYPE {
        return Err(SexpTypeError {
            expected: T::SEXP_TYPE,
            actual,
        }
        .into());
    }

    let (d0, d1, d2) = get_array3_dims(sexp)?;
    let slice: &[T] = unsafe { sexp.as_slice() };
    Array3::from_shape_vec((d0, d1, d2).f(), slice.to_vec()).map_err(|_| {
        SexpLengthError {
            expected: d0 * d1 * d2,
            actual: slice.len(),
        }
        .into()
    })
}

fn array4_from_sexp<T: RNativeType + Copy>(sexp: SEXP) -> Result<Array4<T>, SexpError> {
    let actual = sexp.type_of();
    if actual != T::SEXP_TYPE {
        return Err(SexpTypeError {
            expected: T::SEXP_TYPE,
            actual,
        }
        .into());
    }

    let dims = get_array_dims(sexp).ok_or(SexpLengthError {
        expected: 4,
        actual: 1,
    })?;
    if dims.len() != 4 {
        return Err(SexpLengthError {
            expected: 4,
            actual: dims.len(),
        }
        .into());
    }

    let slice: &[T] = unsafe { sexp.as_slice() };
    Array4::from_shape_vec((dims[0], dims[1], dims[2], dims[3]).f(), slice.to_vec()).map_err(|_| {
        SexpLengthError {
            expected: dims.iter().product(),
            actual: slice.len(),
        }
        .into()
    })
}

fn array5_from_sexp<T: RNativeType + Copy>(sexp: SEXP) -> Result<Array5<T>, SexpError> {
    let actual = sexp.type_of();
    if actual != T::SEXP_TYPE {
        return Err(SexpTypeError {
            expected: T::SEXP_TYPE,
            actual,
        }
        .into());
    }

    let dims = get_array_dims(sexp).ok_or(SexpLengthError {
        expected: 5,
        actual: 1,
    })?;
    if dims.len() != 5 {
        return Err(SexpLengthError {
            expected: 5,
            actual: dims.len(),
        }
        .into());
    }

    let slice: &[T] = unsafe { sexp.as_slice() };
    Array5::from_shape_vec(
        (dims[0], dims[1], dims[2], dims[3], dims[4]).f(),
        slice.to_vec(),
    )
    .map_err(|_| {
        SexpLengthError {
            expected: dims.iter().product(),
            actual: slice.len(),
        }
        .into()
    })
}

fn array6_from_sexp<T: RNativeType + Copy>(sexp: SEXP) -> Result<Array6<T>, SexpError> {
    let actual = sexp.type_of();
    if actual != T::SEXP_TYPE {
        return Err(SexpTypeError {
            expected: T::SEXP_TYPE,
            actual,
        }
        .into());
    }

    let dims = get_array_dims(sexp).ok_or(SexpLengthError {
        expected: 6,
        actual: 1,
    })?;
    if dims.len() != 6 {
        return Err(SexpLengthError {
            expected: 6,
            actual: dims.len(),
        }
        .into());
    }

    let slice: &[T] = unsafe { sexp.as_slice() };
    Array6::from_shape_vec(
        (dims[0], dims[1], dims[2], dims[3], dims[4], dims[5]).f(),
        slice.to_vec(),
    )
    .map_err(|_| {
        SexpLengthError {
            expected: dims.iter().product(),
            actual: slice.len(),
        }
        .into()
    })
}

fn arrayd_from_sexp<T: RNativeType + Copy>(sexp: SEXP) -> Result<ArrayD<T>, SexpError> {
    let actual = sexp.type_of();
    if actual != T::SEXP_TYPE {
        return Err(SexpTypeError {
            expected: T::SEXP_TYPE,
            actual,
        }
        .into());
    }

    let dims = get_array_dims(sexp).unwrap_or_else(|| vec![sexp.len()]);
    let slice: &[T] = unsafe { sexp.as_slice() };
    let shape = IxDyn(&dims);
    ArrayD::from_shape_vec(shape.f(), slice.to_vec()).map_err(|_| {
        SexpLengthError {
            expected: dims.iter().product(),
            actual: slice.len(),
        }
        .into()
    })
}

macro_rules! impl_array_try_from_sexp_native {
    ($t:ty) => {
        impl TryFromSexp for Array0<$t> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                array0_from_sexp::<$t>(sexp)
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                array0_from_sexp::<$t>(sexp)
            }
        }

        impl TryFromSexp for Array1<$t> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                array1_from_sexp::<$t>(sexp)
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                array1_from_sexp::<$t>(sexp)
            }
        }

        impl TryFromSexp for Array2<$t> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                array2_from_sexp::<$t>(sexp)
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                array2_from_sexp::<$t>(sexp)
            }
        }

        impl TryFromSexp for Array3<$t> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                array3_from_sexp::<$t>(sexp)
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                array3_from_sexp::<$t>(sexp)
            }
        }

        impl TryFromSexp for Array4<$t> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                array4_from_sexp::<$t>(sexp)
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                array4_from_sexp::<$t>(sexp)
            }
        }

        impl TryFromSexp for Array5<$t> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                array5_from_sexp::<$t>(sexp)
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                array5_from_sexp::<$t>(sexp)
            }
        }

        impl TryFromSexp for Array6<$t> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                array6_from_sexp::<$t>(sexp)
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                array6_from_sexp::<$t>(sexp)
            }
        }

        impl TryFromSexp for ArrayD<$t> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                arrayd_from_sexp::<$t>(sexp)
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                arrayd_from_sexp::<$t>(sexp)
            }
        }

        impl TryFromSexp for ArcArray1<$t> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let arr: Array1<$t> = TryFromSexp::try_from_sexp(sexp)?;
                Ok(arr.into_shared())
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                let arr: Array1<$t> = TryFromSexp::try_from_sexp(sexp)?;
                Ok(arr.into_shared())
            }
        }

        impl TryFromSexp for ArcArray2<$t> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let arr: Array2<$t> = TryFromSexp::try_from_sexp(sexp)?;
                Ok(arr.into_shared())
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                let arr: Array2<$t> = TryFromSexp::try_from_sexp(sexp)?;
                Ok(arr.into_shared())
            }
        }
    };
}

impl_array_try_from_sexp_native!(i32);
impl_array_try_from_sexp_native!(f64);
impl_array_try_from_sexp_native!(u8);
impl_array_try_from_sexp_native!(RLogical);
impl_array_try_from_sexp_native!(crate::ffi::Rcomplex);

// =============================================================================
// Array0 conversions (0-dimensional scalar arrays)
// =============================================================================

/// Convert `Array0<T>` to R scalar (length-1 vector).
impl<T: RNativeType + Clone> IntoR for Array0<T> {
    fn into_sexp(self) -> SEXP {
        // Extract the single element and convert to R scalar via Vec
        vec![self.into_scalar()].into_sexp()
    }

    unsafe fn into_sexp_unchecked(self) -> SEXP {
        unsafe { vec![self.into_scalar()].into_sexp_unchecked() }
    }
}

// =============================================================================
// Array1 conversions
// =============================================================================

/// Convert `Array1<T>` to R vector.
impl<T: RNativeType> IntoR for Array1<T> {
    fn into_sexp(self) -> SEXP {
        let vec: Vec<T> = self.into_raw_vec_and_offset().0;
        vec.into_sexp()
    }

    unsafe fn into_sexp_unchecked(self) -> SEXP {
        let vec: Vec<T> = self.into_raw_vec_and_offset().0;
        unsafe { vec.into_sexp_unchecked() }
    }
}

// =============================================================================
// Coerced Array conversions via TryCoerce
// =============================================================================
//
// These macros implement TryFromSexp for Array*<T> where T is not an R native
// type but can be coerced from one. This mirrors the pattern in into_r.rs
// (impl_into_r_via_coerce!) for the reverse direction.
//
// Example: Array1<i8> reads from R integer (i32) and coerces each element.

/// Helper to coerce a slice element-wise.
fn coerce_slice<R, T>(slice: &[R]) -> Result<Vec<T>, SexpError>
where
    R: Copy + TryCoerce<T>,
    <R as TryCoerce<T>>::Error: std::fmt::Debug,
{
    slice
        .iter()
        .copied()
        .map(|v| {
            v.try_coerce()
                .map_err(|e| SexpError::InvalidValue(format!("{e:?}")))
        })
        .collect()
}

/// Implement TryFromSexp for Array1<$target> by reading $source and coercing.
macro_rules! impl_array1_try_from_sexp_coerce {
    ($source:ty => $target:ty) => {
        impl TryFromSexp for Array1<$target> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$source as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$source as RNativeType>::SEXP_TYPE,
                        actual,
                    }
                    .into());
                }
                let slice: &[$source] = unsafe { sexp.as_slice() };
                let data: Vec<$target> = coerce_slice(slice)?;
                Ok(Array1::from_vec(data))
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                Self::try_from_sexp(sexp)
            }
        }
    };
}

/// Implement TryFromSexp for Array2<$target> by reading $source and coercing.
macro_rules! impl_array2_try_from_sexp_coerce {
    ($source:ty => $target:ty) => {
        impl TryFromSexp for Array2<$target> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$source as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$source as RNativeType>::SEXP_TYPE,
                        actual,
                    }
                    .into());
                }
                let (nrow, ncol) = get_matrix_dims(sexp)?;
                let slice: &[$source] = unsafe { sexp.as_slice() };
                let data: Vec<$target> = coerce_slice(slice)?;
                Array2::from_shape_vec((nrow, ncol).f(), data).map_err(|_| {
                    SexpLengthError {
                        expected: nrow * ncol,
                        actual: slice.len(),
                    }
                    .into()
                })
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                Self::try_from_sexp(sexp)
            }
        }
    };
}

/// Implement TryFromSexp for Array3<$target> by reading $source and coercing.
macro_rules! impl_array3_try_from_sexp_coerce {
    ($source:ty => $target:ty) => {
        impl TryFromSexp for Array3<$target> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$source as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$source as RNativeType>::SEXP_TYPE,
                        actual,
                    }
                    .into());
                }
                let (d0, d1, d2) = get_array3_dims(sexp)?;
                let slice: &[$source] = unsafe { sexp.as_slice() };
                let data: Vec<$target> = coerce_slice(slice)?;
                Array3::from_shape_vec((d0, d1, d2).f(), data).map_err(|_| {
                    SexpLengthError {
                        expected: d0 * d1 * d2,
                        actual: slice.len(),
                    }
                    .into()
                })
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                Self::try_from_sexp(sexp)
            }
        }
    };
}

/// Implement TryFromSexp for Array4<$target> by reading $source and coercing.
macro_rules! impl_array4_try_from_sexp_coerce {
    ($source:ty => $target:ty) => {
        impl TryFromSexp for Array4<$target> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$source as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$source as RNativeType>::SEXP_TYPE,
                        actual,
                    }
                    .into());
                }
                let dims = get_array_dims(sexp).ok_or_else(|| SexpLengthError {
                    expected: 4,
                    actual: 1,
                })?;
                if dims.len() != 4 {
                    return Err(SexpLengthError {
                        expected: 4,
                        actual: dims.len(),
                    }
                    .into());
                }
                let slice: &[$source] = unsafe { sexp.as_slice() };
                let data: Vec<$target> = coerce_slice(slice)?;
                Array4::from_shape_vec((dims[0], dims[1], dims[2], dims[3]).f(), data).map_err(
                    |_| {
                        SexpLengthError {
                            expected: dims.iter().product(),
                            actual: slice.len(),
                        }
                        .into()
                    },
                )
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                Self::try_from_sexp(sexp)
            }
        }
    };
}

/// Implement TryFromSexp for Array5<$target> by reading $source and coercing.
macro_rules! impl_array5_try_from_sexp_coerce {
    ($source:ty => $target:ty) => {
        impl TryFromSexp for Array5<$target> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$source as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$source as RNativeType>::SEXP_TYPE,
                        actual,
                    }
                    .into());
                }
                let dims = get_array_dims(sexp).ok_or_else(|| SexpLengthError {
                    expected: 5,
                    actual: 1,
                })?;
                if dims.len() != 5 {
                    return Err(SexpLengthError {
                        expected: 5,
                        actual: dims.len(),
                    }
                    .into());
                }
                let slice: &[$source] = unsafe { sexp.as_slice() };
                let data: Vec<$target> = coerce_slice(slice)?;
                Array5::from_shape_vec((dims[0], dims[1], dims[2], dims[3], dims[4]).f(), data)
                    .map_err(|_| {
                        SexpLengthError {
                            expected: dims.iter().product(),
                            actual: slice.len(),
                        }
                        .into()
                    })
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                Self::try_from_sexp(sexp)
            }
        }
    };
}

/// Implement TryFromSexp for Array6<$target> by reading $source and coercing.
macro_rules! impl_array6_try_from_sexp_coerce {
    ($source:ty => $target:ty) => {
        impl TryFromSexp for Array6<$target> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$source as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$source as RNativeType>::SEXP_TYPE,
                        actual,
                    }
                    .into());
                }
                let dims = get_array_dims(sexp).ok_or_else(|| SexpLengthError {
                    expected: 6,
                    actual: 1,
                })?;
                if dims.len() != 6 {
                    return Err(SexpLengthError {
                        expected: 6,
                        actual: dims.len(),
                    }
                    .into());
                }
                let slice: &[$source] = unsafe { sexp.as_slice() };
                let data: Vec<$target> = coerce_slice(slice)?;
                Array6::from_shape_vec(
                    (dims[0], dims[1], dims[2], dims[3], dims[4], dims[5]).f(),
                    data,
                )
                .map_err(|_| {
                    SexpLengthError {
                        expected: dims.iter().product(),
                        actual: slice.len(),
                    }
                    .into()
                })
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                Self::try_from_sexp(sexp)
            }
        }
    };
}

/// Implement TryFromSexp for ArrayD<$target> by reading $source and coercing.
macro_rules! impl_arrayd_try_from_sexp_coerce {
    ($source:ty => $target:ty) => {
        impl TryFromSexp for ArrayD<$target> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                let actual = sexp.type_of();
                if actual != <$source as RNativeType>::SEXP_TYPE {
                    return Err(SexpTypeError {
                        expected: <$source as RNativeType>::SEXP_TYPE,
                        actual,
                    }
                    .into());
                }
                let dims = get_array_dims(sexp).unwrap_or_else(|| vec![sexp.len()]);
                let slice: &[$source] = unsafe { sexp.as_slice() };
                let data: Vec<$target> = coerce_slice(slice)?;
                let shape = IxDyn(&dims);
                ArrayD::from_shape_vec(shape.f(), data).map_err(|_| {
                    SexpLengthError {
                        expected: dims.iter().product(),
                        actual: slice.len(),
                    }
                    .into()
                })
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                Self::try_from_sexp(sexp)
            }
        }
    };
}

/// Implement TryFromSexp for all array types for a given source/target pair.
macro_rules! impl_all_arrays_try_from_sexp_coerce {
    ($source:ty => $target:ty) => {
        impl_array1_try_from_sexp_coerce!($source => $target);
        impl_array2_try_from_sexp_coerce!($source => $target);
        impl_array3_try_from_sexp_coerce!($source => $target);
        impl_array4_try_from_sexp_coerce!($source => $target);
        impl_array5_try_from_sexp_coerce!($source => $target);
        impl_array6_try_from_sexp_coerce!($source => $target);
        impl_arrayd_try_from_sexp_coerce!($source => $target);
    };
}

// -----------------------------------------------------------------------------
// Integer coercions: R integer (i32) -> various Rust integer types
// -----------------------------------------------------------------------------
impl_all_arrays_try_from_sexp_coerce!(i32 => i8);
impl_all_arrays_try_from_sexp_coerce!(i32 => i16);
impl_all_arrays_try_from_sexp_coerce!(i32 => i64);
impl_all_arrays_try_from_sexp_coerce!(i32 => isize);
impl_all_arrays_try_from_sexp_coerce!(i32 => u16);
impl_all_arrays_try_from_sexp_coerce!(i32 => u32);
impl_all_arrays_try_from_sexp_coerce!(i32 => u64);
impl_all_arrays_try_from_sexp_coerce!(i32 => usize);

// -----------------------------------------------------------------------------
// Float coercions: R numeric (f64) -> f32
// -----------------------------------------------------------------------------
impl_all_arrays_try_from_sexp_coerce!(f64 => f32);

// -----------------------------------------------------------------------------
// Logical coercions: R logical (RLogical) -> bool
// -----------------------------------------------------------------------------
impl_all_arrays_try_from_sexp_coerce!(crate::ffi::RLogical => bool);

// =============================================================================
// Array2 conversions
// =============================================================================

/// Convert `Array2<T>` to R matrix.
///
/// Creates a column-major R matrix from the ndarray.
/// Data is always written in column-major order regardless of the array's internal layout.
impl<T: RNativeType + Clone> IntoR for Array2<T> {
    fn into_sexp(self) -> SEXP {
        let (nrow, ncol) = self.dim();

        // Always produce column-major data by iterating column-by-column.
        // This works correctly regardless of whether the array is row-major (standard)
        // or column-major (Fortran order).
        //
        // Note: ndarray's `iter()` always visits in logical order (row-major iteration),
        // NOT memory order. So we explicitly iterate by columns to get R's expected layout.
        let mut data: Vec<T> = Vec::with_capacity(nrow * ncol);
        for j in 0..ncol {
            data.extend(self.column(j).iter().cloned());
        }

        // Create R matrix with RAII protection
        unsafe {
            let mat = crate::ffi::Rf_allocMatrix(T::SEXP_TYPE, nrow as i32, ncol as i32);
            let guard = OwnedProtect::new(mat);

            let ptr = crate::ffi::DATAPTR_RO(guard.get()) as *mut T;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());

            // Return the SEXP - guard drops and unprotects
            // (R manages protection of .Call return values)
            guard.get()
        }
    }
}

// =============================================================================
// Array3 conversions (3D arrays)
// =============================================================================

/// Convert `Array3<T>` to R 3D array.
///
/// Creates a column-major R array from the ndarray.
impl<T: RNativeType + Clone> IntoR for Array3<T> {
    fn into_sexp(self) -> SEXP {
        let (d0, d1, d2) = self.dim();

        // Iterate in Fortran (column-major) order
        let data: Vec<T> = {
            let mut v = Vec::with_capacity(d0 * d1 * d2);
            for k in 0..d2 {
                for j in 0..d1 {
                    for i in 0..d0 {
                        v.push(self[[i, j, k]]);
                    }
                }
            }
            v
        };

        // Create R array with dim attribute using RAII protection
        unsafe {
            let scope = ProtectScope::new();

            let arr = scope.protect_raw(crate::ffi::Rf_allocVector(
                T::SEXP_TYPE,
                data.len() as crate::ffi::R_xlen_t,
            ));

            let ptr = crate::ffi::DATAPTR_RO(arr) as *mut T;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());

            // Set dim attribute
            let dim = scope.protect_raw(crate::ffi::Rf_allocVector(SEXPTYPE::INTSXP, 3));
            let dim_ptr = crate::ffi::INTEGER(dim);
            *dim_ptr = d0 as i32;
            *dim_ptr.add(1) = d1 as i32;
            *dim_ptr.add(2) = d2 as i32;
            crate::ffi::Rf_setAttrib(arr, crate::ffi::R_DimSymbol, dim);

            arr
        } // scope drops here, calling UNPROTECT(2)
    }
}

// =============================================================================
// Array4 conversions (4D arrays)
// =============================================================================

/// Convert `Array4<T>` to R 4D array.
impl<T: RNativeType + Clone> IntoR for Array4<T> {
    fn into_sexp(self) -> SEXP {
        let (d0, d1, d2, d3) = self.dim();
        let shape = vec![d0, d1, d2, d3];
        let total_len = shape.iter().product();

        let data: Vec<T> = {
            let mut v = Vec::with_capacity(total_len);
            fortran_order_iter(&shape, |idx| {
                v.push(self[[idx[0], idx[1], idx[2], idx[3]]]);
            });
            v
        };

        unsafe {
            let scope = ProtectScope::new();
            let arr = scope.protect_raw(crate::ffi::Rf_allocVector(
                T::SEXP_TYPE,
                total_len as crate::ffi::R_xlen_t,
            ));

            let ptr = crate::ffi::DATAPTR_RO(arr) as *mut T;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());

            let dim = scope.protect_raw(crate::ffi::Rf_allocVector(SEXPTYPE::INTSXP, 4));
            let dim_ptr = crate::ffi::INTEGER(dim);
            *dim_ptr = d0 as i32;
            *dim_ptr.add(1) = d1 as i32;
            *dim_ptr.add(2) = d2 as i32;
            *dim_ptr.add(3) = d3 as i32;
            crate::ffi::Rf_setAttrib(arr, crate::ffi::R_DimSymbol, dim);

            arr
        }
    }
}

// =============================================================================
// Array5 conversions (5D arrays)
// =============================================================================

/// Convert `Array5<T>` to R 5D array.
impl<T: RNativeType + Clone> IntoR for Array5<T> {
    fn into_sexp(self) -> SEXP {
        let (d0, d1, d2, d3, d4) = self.dim();
        let shape = vec![d0, d1, d2, d3, d4];
        let total_len = shape.iter().product();

        let data: Vec<T> = {
            let mut v = Vec::with_capacity(total_len);
            fortran_order_iter(&shape, |idx| {
                v.push(self[[idx[0], idx[1], idx[2], idx[3], idx[4]]]);
            });
            v
        };

        unsafe {
            let scope = ProtectScope::new();
            let arr = scope.protect_raw(crate::ffi::Rf_allocVector(
                T::SEXP_TYPE,
                total_len as crate::ffi::R_xlen_t,
            ));

            let ptr = crate::ffi::DATAPTR_RO(arr) as *mut T;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());

            let dim = scope.protect_raw(crate::ffi::Rf_allocVector(SEXPTYPE::INTSXP, 5));
            let dim_ptr = crate::ffi::INTEGER(dim);
            for (i, &d) in [d0, d1, d2, d3, d4].iter().enumerate() {
                *dim_ptr.add(i) = d as i32;
            }
            crate::ffi::Rf_setAttrib(arr, crate::ffi::R_DimSymbol, dim);

            arr
        }
    }
}

// =============================================================================
// Array6 conversions (6D arrays)
// =============================================================================

/// Convert `Array6<T>` to R 6D array.
impl<T: RNativeType + Clone> IntoR for Array6<T> {
    fn into_sexp(self) -> SEXP {
        let (d0, d1, d2, d3, d4, d5) = self.dim();
        let shape = vec![d0, d1, d2, d3, d4, d5];
        let total_len = shape.iter().product();

        let data: Vec<T> = {
            let mut v = Vec::with_capacity(total_len);
            fortran_order_iter(&shape, |idx| {
                v.push(self[[idx[0], idx[1], idx[2], idx[3], idx[4], idx[5]]]);
            });
            v
        };

        unsafe {
            let scope = ProtectScope::new();
            let arr = scope.protect_raw(crate::ffi::Rf_allocVector(
                T::SEXP_TYPE,
                total_len as crate::ffi::R_xlen_t,
            ));

            let ptr = crate::ffi::DATAPTR_RO(arr) as *mut T;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());

            let dim = scope.protect_raw(crate::ffi::Rf_allocVector(SEXPTYPE::INTSXP, 6));
            let dim_ptr = crate::ffi::INTEGER(dim);
            for (i, &d) in [d0, d1, d2, d3, d4, d5].iter().enumerate() {
                *dim_ptr.add(i) = d as i32;
            }
            crate::ffi::Rf_setAttrib(arr, crate::ffi::R_DimSymbol, dim);

            arr
        }
    }
}

// =============================================================================
// ArrayD conversions (N-dimensional arrays)
// =============================================================================

/// Convert `ArrayD<T>` to R N-dimensional array.
///
/// Creates a column-major R array from the ndarray.
impl<T: RNativeType + Clone> IntoR for ArrayD<T> {
    fn into_sexp(self) -> SEXP {
        let shape: Vec<usize> = self.shape().to_vec();
        let ndim = shape.len();
        let total_len: usize = shape.iter().product();

        // Iterate in Fortran (column-major) order
        let data: Vec<T> = {
            let mut v = Vec::with_capacity(total_len);
            // Use ndarray's built-in Fortran-order iteration
            // by creating indices in column-major order
            fortran_order_iter(&shape, |idx| {
                v.push(self[IxDyn(&idx)]);
            });
            v
        };

        // Create R array with RAII protection
        unsafe {
            let scope = ProtectScope::new();

            let arr = scope.protect_raw(crate::ffi::Rf_allocVector(
                T::SEXP_TYPE,
                total_len as crate::ffi::R_xlen_t,
            ));

            let ptr = crate::ffi::DATAPTR_RO(arr) as *mut T;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());

            // Set dim attribute if ndim > 1
            if ndim > 1 {
                let dim = scope.protect_raw(crate::ffi::Rf_allocVector(
                    SEXPTYPE::INTSXP,
                    ndim as crate::ffi::R_xlen_t,
                ));
                let dim_ptr = crate::ffi::INTEGER(dim);
                for (i, &d) in shape.iter().enumerate() {
                    *dim_ptr.add(i) = d as i32;
                }
                crate::ffi::Rf_setAttrib(arr, crate::ffi::R_DimSymbol, dim);
            }

            arr
        } // scope drops here, calling UNPROTECT(n) automatically
    }
}

/// Iterate over indices in Fortran (column-major) order.
///
/// For 0-D arrays (empty shape), calls `f` once with an empty index vector,
/// since a 0-D array contains exactly one scalar element.
fn fortran_order_iter<F: FnMut(Vec<usize>)>(shape: &[usize], mut f: F) {
    if shape.is_empty() {
        // 0-D array: one scalar element, index is empty vec
        f(Vec::new());
        return;
    }

    let ndim = shape.len();
    let mut idx = vec![0usize; ndim];

    loop {
        f(idx.clone());

        // Increment index in Fortran order (first dimension varies fastest)
        let mut dim = 0;
        loop {
            idx[dim] += 1;
            if idx[dim] < shape[dim] {
                break;
            }
            idx[dim] = 0;
            dim += 1;
            if dim >= ndim {
                return; // Done
            }
        }
    }
}

// =============================================================================
// ArcArray conversions (shared ownership)
// =============================================================================

/// Convert `ArcArray1<T>` to R vector.
impl<T: RNativeType + Clone> IntoR for ArcArray1<T> {
    fn into_sexp(self) -> SEXP {
        // Convert to owned array, then to SEXP
        // If we have unique ownership, this is efficient; otherwise it clones
        let arr: Array1<T> = self.into_owned();
        arr.into_sexp()
    }

    unsafe fn into_sexp_unchecked(self) -> SEXP {
        let arr: Array1<T> = self.into_owned();
        unsafe { arr.into_sexp_unchecked() }
    }
}

/// Convert `ArcArray2<T>` to R matrix.
impl<T: RNativeType + Clone> IntoR for ArcArray2<T> {
    fn into_sexp(self) -> SEXP {
        let arr: Array2<T> = self.into_owned();
        arr.into_sexp()
    }

    unsafe fn into_sexp_unchecked(self) -> SEXP {
        let arr: Array2<T> = self.into_owned();
        unsafe { arr.into_sexp_unchecked() }
    }
}

// =============================================================================
// ArrayView conversions (read-only views)
// =============================================================================

/// Convert `ArrayView1<T>` to R vector by copying.
///
/// This copies the view's data into a new R vector.
/// Useful for returning a view of data to R.
impl<'a, T: RNativeType + Clone> IntoR for ArrayView1<'a, T> {
    fn into_sexp(self) -> SEXP {
        let vec: Vec<T> = self.iter().cloned().collect();
        vec.into_sexp()
    }

    unsafe fn into_sexp_unchecked(self) -> SEXP {
        let vec: Vec<T> = self.iter().cloned().collect();
        unsafe { vec.into_sexp_unchecked() }
    }
}

/// Convert `ArrayView2<T>` to R matrix by copying.
///
/// Creates a column-major R matrix from the view.
/// Data is always written in column-major order regardless of the view's layout.
impl<'a, T: RNativeType + Clone> IntoR for ArrayView2<'a, T> {
    fn into_sexp(self) -> SEXP {
        let (nrow, ncol) = self.dim();

        // Iterate column-by-column for R's column-major storage
        let mut data: Vec<T> = Vec::with_capacity(nrow * ncol);
        for j in 0..ncol {
            data.extend(self.column(j).iter().cloned());
        }

        unsafe {
            let mat = crate::ffi::Rf_allocMatrix(T::SEXP_TYPE, nrow as i32, ncol as i32);
            let guard = OwnedProtect::new(mat);

            let ptr = crate::ffi::DATAPTR_RO(guard.get()) as *mut T;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());

            // Return the SEXP - guard drops and unprotects
            // (R manages protection of .Call return values)
            guard.get()
        }
    }
}

/// Convert `ArrayView3<T>` to R 3D array by copying.
impl<'a, T: RNativeType + Clone> IntoR for ArrayView3<'a, T> {
    fn into_sexp(self) -> SEXP {
        let (d0, d1, d2) = self.dim();

        // Iterate in Fortran (column-major) order
        let data: Vec<T> = {
            let mut v = Vec::with_capacity(d0 * d1 * d2);
            for k in 0..d2 {
                for j in 0..d1 {
                    for i in 0..d0 {
                        v.push(self[[i, j, k]]);
                    }
                }
            }
            v
        };

        unsafe {
            let scope = ProtectScope::new();

            let arr = scope.protect_raw(crate::ffi::Rf_allocVector(
                T::SEXP_TYPE,
                data.len() as crate::ffi::R_xlen_t,
            ));

            let ptr = crate::ffi::DATAPTR_RO(arr) as *mut T;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());

            // Set dim attribute
            let dim = scope.protect_raw(crate::ffi::Rf_allocVector(SEXPTYPE::INTSXP, 3));
            let dim_ptr = crate::ffi::INTEGER(dim);
            *dim_ptr = d0 as i32;
            *dim_ptr.add(1) = d1 as i32;
            *dim_ptr.add(2) = d2 as i32;
            crate::ffi::Rf_setAttrib(arr, crate::ffi::R_DimSymbol, dim);

            arr
        }
    }
}

/// Convert `ArrayViewD<T>` to R N-dimensional array by copying.
impl<'a, T: RNativeType + Clone> IntoR for ArrayViewD<'a, T> {
    fn into_sexp(self) -> SEXP {
        let shape: Vec<usize> = self.shape().to_vec();
        let ndim = shape.len();
        let total_len: usize = shape.iter().product();

        // Iterate in Fortran (column-major) order
        let data: Vec<T> = {
            let mut v = Vec::with_capacity(total_len);
            fortran_order_iter(&shape, |idx| {
                v.push(self[IxDyn(&idx)]);
            });
            v
        };

        unsafe {
            let scope = ProtectScope::new();

            let arr = scope.protect_raw(crate::ffi::Rf_allocVector(
                T::SEXP_TYPE,
                total_len as crate::ffi::R_xlen_t,
            ));

            let ptr = crate::ffi::DATAPTR_RO(arr) as *mut T;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());

            // Set dim attribute if ndim > 1
            if ndim > 1 {
                let dim = scope.protect_raw(crate::ffi::Rf_allocVector(
                    SEXPTYPE::INTSXP,
                    ndim as crate::ffi::R_xlen_t,
                ));
                let dim_ptr = crate::ffi::INTEGER(dim);
                for (i, &d) in shape.iter().enumerate() {
                    *dim_ptr.add(i) = d as i32;
                }
                crate::ffi::Rf_setAttrib(arr, crate::ffi::R_DimSymbol, dim);
            }

            arr
        }
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// Get array dimensions from R object as a Vec.
///
/// Returns `None` if no dim attribute (plain vector).
/// Returns `None` if any dimension is negative or NA (NA_INTEGER = i32::MIN).
fn get_array_dims(sexp: SEXP) -> Option<Vec<usize>> {
    unsafe {
        let dim = crate::ffi::Rf_getAttrib(sexp, crate::ffi::R_DimSymbol);
        if dim.type_of() != SEXPTYPE::INTSXP {
            return None;
        }

        let dim_slice: &[i32] = dim.as_slice();
        // Validate: reject negative or NA dimensions (NA_INTEGER is i32::MIN)
        let mut dims = Vec::with_capacity(dim_slice.len());
        for &d in dim_slice {
            if d < 0 {
                return None;
            }
            dims.push(d as usize);
        }
        Some(dims)
    }
}

/// Get matrix dimensions from R object.
fn get_matrix_dims(sexp: SEXP) -> Result<(usize, usize), SexpError> {
    match get_array_dims(sexp) {
        None => Ok((sexp.len(), 1)), // Plain vector → column vector
        Some(dims) if dims.len() == 2 => Ok((dims[0], dims[1])),
        Some(dims) => Err(SexpLengthError {
            expected: 2,
            actual: dims.len(),
        }
        .into()),
    }
}

/// Get 3D array dimensions from R object.
fn get_array3_dims(sexp: SEXP) -> Result<(usize, usize, usize), SexpError> {
    match get_array_dims(sexp) {
        None => Err(SexpLengthError {
            expected: 3,
            actual: 1,
        }
        .into()),
        Some(dims) if dims.len() == 3 => Ok((dims[0], dims[1], dims[2])),
        Some(dims) => Err(SexpLengthError {
            expected: 3,
            actual: dims.len(),
        }
        .into()),
    }
}

// =============================================================================
// Zero-copy slice access
// =============================================================================

/// Create an `ArrayView1` from an R vector without copying.
///
/// # Safety
///
/// The returned view is only valid as long as the R object is protected.
/// The SEXP must be of the correct type for `T`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::ndarray_impl::from_r_slice;
///
/// #[miniextendr]
/// fn sum_view(x: SEXP) -> f64 {
///     let view = unsafe { from_r_slice::<f64>(x).unwrap() };
///     view.sum()
/// }
/// ```
pub unsafe fn from_r_slice<T: RNativeType>(
    sexp: SEXP,
) -> Result<ArrayView1<'static, T>, SexpError> {
    let actual = sexp.type_of();
    if actual != T::SEXP_TYPE {
        return Err(SexpTypeError {
            expected: T::SEXP_TYPE,
            actual,
        }
        .into());
    }

    let slice: &[T] = unsafe { sexp.as_slice() };
    Ok(ArrayView1::from(slice))
}

/// Create an `ArrayView2` from an R matrix without copying.
///
/// Returns a Fortran-order (column-major) view that directly references R's
/// matrix storage. This is true zero-copy access.
///
/// # Safety
///
/// - The returned view is only valid as long as the R object is protected.
/// - The SEXP must be of the correct type for `T`.
/// - The SEXP must be a matrix (dim attribute of length 2) or a plain vector
///   (which is treated as an n×1 column matrix).
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::ndarray_impl::from_r_matrix;
///
/// #[miniextendr]
/// fn matrix_sum(x: SEXP) -> f64 {
///     let view = unsafe { from_r_matrix::<f64>(x).unwrap() };
///     view.sum()
/// }
/// ```
pub unsafe fn from_r_matrix<T: RNativeType>(
    sexp: SEXP,
) -> Result<ArrayView2<'static, T>, SexpError> {
    let actual = sexp.type_of();
    if actual != T::SEXP_TYPE {
        return Err(SexpTypeError {
            expected: T::SEXP_TYPE,
            actual,
        }
        .into());
    }

    let (nrow, ncol) = get_matrix_dims(sexp)?;
    let slice: &[T] = unsafe { sexp.as_slice() };

    // R stores in column-major (Fortran) order, so create a Fortran-order view
    let view = ArrayView2::from_shape((nrow, ncol).f(), slice).map_err(|_| SexpLengthError {
        expected: nrow * ncol,
        actual: slice.len(),
    })?;

    Ok(view)
}

/// Create an `ArrayView3` from an R 3D array without copying.
///
/// Returns a Fortran-order (column-major) view that directly references R's
/// array storage. This is true zero-copy access.
///
/// # Safety
///
/// - The returned view is only valid as long as the R object is protected.
/// - The SEXP must be of the correct type for `T`.
/// - The SEXP must be a 3D array (have a `dim` attribute of length 3).
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::ndarray_impl::from_r_array3;
///
/// #[miniextendr]
/// fn cube_sum(x: SEXP) -> f64 {
///     let view = unsafe { from_r_array3::<f64>(x).unwrap() };
///     view.sum()
/// }
/// ```
pub unsafe fn from_r_array3<T: RNativeType>(
    sexp: SEXP,
) -> Result<ArrayView3<'static, T>, SexpError> {
    let actual = sexp.type_of();
    if actual != T::SEXP_TYPE {
        return Err(SexpTypeError {
            expected: T::SEXP_TYPE,
            actual,
        }
        .into());
    }

    let (d0, d1, d2) = get_array3_dims(sexp)?;
    let slice: &[T] = unsafe { sexp.as_slice() };

    // R stores in column-major (Fortran) order, so create a Fortran-order view
    let view = ArrayView3::from_shape((d0, d1, d2).f(), slice).map_err(|_| SexpLengthError {
        expected: d0 * d1 * d2,
        actual: slice.len(),
    })?;

    Ok(view)
}

/// Create an `ArrayViewD` from an R N-dimensional array without copying.
///
/// Returns a Fortran-order (column-major) view that directly references R's
/// array storage. This is true zero-copy access for arrays of any dimension.
///
/// Plain vectors (no dim attribute) are treated as 1D arrays.
///
/// # Safety
///
/// - The returned view is only valid as long as the R object is protected.
/// - The SEXP must be of the correct type for `T`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::ndarray_impl::from_r_array;
///
/// #[miniextendr]
/// fn nd_sum(x: SEXP) -> f64 {
///     let view = unsafe { from_r_array::<f64>(x).unwrap() };
///     view.sum()
/// }
/// ```
pub unsafe fn from_r_array<T: RNativeType>(
    sexp: SEXP,
) -> Result<ArrayViewD<'static, T>, SexpError> {
    let actual = sexp.type_of();
    if actual != T::SEXP_TYPE {
        return Err(SexpTypeError {
            expected: T::SEXP_TYPE,
            actual,
        }
        .into());
    }

    let dims = get_array_dims(sexp).unwrap_or_else(|| vec![sexp.len()]);
    let slice: &[T] = unsafe { sexp.as_slice() };

    let expected_len: usize = dims.iter().product();

    // R stores in column-major (Fortran) order, so create a Fortran-order view
    let shape = IxDyn(&dims);
    let view = ArrayViewD::from_shape(shape.f(), slice).map_err(|_| SexpLengthError {
        expected: expected_len,
        actual: slice.len(),
    })?;

    Ok(view)
}

// =============================================================================
// RArray <-> ndarray conversions
// =============================================================================
//
// These implementations allow converting between RArray (a view over R data)
// and ndarray types using standard From/Into traits.
//
// Two conversion modes are available:
//
// 1. **Zero-copy views** (`From<&RArray>` for `ArrayView`):
//    - `ArrayView1::from(&rvector)` or `(&rvector).into()`
//    - `ArrayView2::from(&rmatrix)` or `(&rmatrix).into()`
//    - The view borrows from the RArray and has the same lifetime
//
// 2. **Copy to owned** (`From<&RArray>` for `Array`):
//    - `Array1::from(&rvector)` or `(&rvector).into()`
//    - `Array2::from(&rmatrix)` or `(&rmatrix).into()`
//    - The resulting array owns its data and can be used freely
//
// For the reverse direction (ndarray -> R):
// - Use the existing `IntoR` impl: `array.into_sexp()`
// - Then wrap as RArray: `RArray::from_sexp(sexp)`
//
// # Safety
//
// These conversions internally use unsafe code to access RArray's slice data.
// This is safe because:
// 1. RArray is always constructed from a valid, type-checked SEXP
// 2. RArray is !Send + !Sync, ensuring it stays on the R main thread
// 3. The lifetime of views is bounded by the RArray reference

use crate::rarray::{RArray, RArray3D, RMatrix, RVector};

// =============================================================================
// Zero-copy view conversions
// =============================================================================

/// Convert `&RVector<T>` to `ArrayView1<T>` without copying.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rarray::RVector;
/// use ndarray::ArrayView1;
///
/// #[miniextendr(unsafe(main_thread))]
/// fn vector_sum(v: RVector<f64>) -> f64 {
///     let view: ArrayView1<f64> = (&v).into();
///     view.sum()
/// }
/// ```
impl<'a, T: RNativeType> From<&'a RVector<T>> for ArrayView1<'a, T> {
    #[inline]
    fn from(arr: &'a RVector<T>) -> Self {
        // SAFETY: RArray is constructed from a valid SEXP and is !Send+!Sync.
        // For 1D, ArrayView1::from(slice) is already optimal.
        let slice = unsafe { arr.as_slice() };
        ArrayView1::from(slice)
    }
}

/// Convert `&RMatrix<T>` to `ArrayView2<T>` without copying.
///
/// The view is in Fortran (column-major) order to match R's storage.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rarray::RMatrix;
/// use ndarray::ArrayView2;
///
/// #[miniextendr(unsafe(main_thread))]
/// fn matrix_trace(m: RMatrix<f64>) -> f64 {
///     let view: ArrayView2<f64> = (&m).into();
///     view.diag().sum()
/// }
/// ```
impl<'a, T: RNativeType> From<&'a RMatrix<T>> for ArrayView2<'a, T> {
    #[inline]
    fn from(arr: &'a RMatrix<T>) -> Self {
        // SAFETY: RArray is constructed from a valid, type-checked SEXP.
        // The dims and slice are consistent by R's invariants.
        let slice = unsafe { arr.as_slice() };
        let dims = unsafe { arr.dims() };
        unsafe { ArrayView2::from_shape_ptr((dims[0], dims[1]).f(), slice.as_ptr()) }
    }
}

/// Convert `&RArray3D<T>` to `ArrayView3<T>` without copying.
impl<'a, T: RNativeType> From<&'a RArray3D<T>> for ArrayView3<'a, T> {
    #[inline]
    fn from(arr: &'a RArray3D<T>) -> Self {
        let slice = unsafe { arr.as_slice() };
        let dims = unsafe { arr.dims() };
        unsafe { ArrayView3::from_shape_ptr((dims[0], dims[1], dims[2]).f(), slice.as_ptr()) }
    }
}

/// Convert `&RArray<T, NDIM>` to `ArrayViewD<T>` without copying.
///
/// Works with any dimension count. Useful for generic code.
impl<'a, T: RNativeType, const NDIM: usize> From<&'a RArray<T, NDIM>> for ArrayViewD<'a, T> {
    #[inline]
    fn from(arr: &'a RArray<T, NDIM>) -> Self {
        let slice = unsafe { arr.as_slice() };
        let dims = unsafe { arr.dims() };
        let shape = IxDyn(&dims);
        unsafe { ArrayViewD::from_shape_ptr(shape.f(), slice.as_ptr()) }
    }
}

// =============================================================================
// Copy-to-owned conversions
// =============================================================================

/// Copy `&RVector<T>` into an owned `Array1<T>`.
///
/// The resulting array owns its data and can outlive the RArray.
/// Useful for passing data to worker threads.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::rarray::RVector;
/// use ndarray::Array1;
///
/// #[miniextendr(unsafe(main_thread))]
/// fn double_vector(v: RVector<f64>) -> Array1<f64> {
///     let arr: Array1<f64> = (&v).into();
///     arr * 2.0
/// }
/// ```
impl<T: RNativeType + Clone> From<&RVector<T>> for Array1<T> {
    #[inline]
    fn from(arr: &RVector<T>) -> Self {
        let slice = unsafe { arr.as_slice() };
        Array1::from_vec(slice.to_vec())
    }
}

/// Copy `&RMatrix<T>` into an owned `Array2<T>`.
///
/// The result is in Fortran (column-major) order to match R.
impl<T: RNativeType + Clone> From<&RMatrix<T>> for Array2<T> {
    #[inline]
    fn from(arr: &RMatrix<T>) -> Self {
        let slice = unsafe { arr.as_slice() };
        let dims = unsafe { arr.dims() };
        // SAFETY: RArray dimensions are validated at construction time,
        // so dims and slice length are guaranteed to be consistent.
        unsafe { Array2::from_shape_vec_unchecked((dims[0], dims[1]).f(), slice.to_vec()) }
    }
}

/// Copy `&RArray3D<T>` into an owned `Array3<T>`.
impl<T: RNativeType + Clone> From<&RArray3D<T>> for Array3<T> {
    #[inline]
    fn from(arr: &RArray3D<T>) -> Self {
        let slice = unsafe { arr.as_slice() };
        let dims = unsafe { arr.dims() };
        // SAFETY: RArray dimensions are validated at construction time.
        unsafe { Array3::from_shape_vec_unchecked((dims[0], dims[1], dims[2]).f(), slice.to_vec()) }
    }
}

/// Copy `&RArray<T, NDIM>` into an owned `ArrayD<T>`.
///
/// Works with any dimension count.
impl<T: RNativeType + Clone, const NDIM: usize> From<&RArray<T, NDIM>> for ArrayD<T> {
    #[inline]
    fn from(arr: &RArray<T, NDIM>) -> Self {
        let slice = unsafe { arr.as_slice() };
        let dims = unsafe { arr.dims() };
        let shape = IxDyn(&dims);
        // SAFETY: RArray dimensions are validated at construction time.
        unsafe { ArrayD::from_shape_vec_unchecked(shape.f(), slice.to_vec()) }
    }
}

// =============================================================================
// TypedExternal implementations for ExternalPtr support
// =============================================================================
//
// These allow storing ndarray arrays as R external pointers (zero-copy).
// Use `ExternalPtr<Array1<f64>>` when you want to pass arrays without copying.
//
// Note: We implement TypedExternal but NOT IntoExternalPtr, because:
// - TypedExternal: Enables explicit `ExternalPtr<Array1<f64>>` usage
// - IntoExternalPtr: Would conflict with our IntoR impl (which copies to R native)
//
// Default behavior: `Array1<f64>` → copies to R vector (via IntoR)
// Explicit ExternalPtr: `ExternalPtr::new(arr)` → wraps without copying

use crate::externalptr::TypedExternal;
use crate::ffi::Rcomplex;

// Helper macro for implementing TypedExternal for ndarray types.
// For these well-known library types, we use the descriptive name as both
// the display name and the type ID (they're already unique as "ndarray::Array1<i32>").
macro_rules! impl_te_ndarray {
    ($ty:ty, $name:expr) => {
        impl TypedExternal for $ty {
            const TYPE_NAME: &'static str = $name;
            const TYPE_NAME_CSTR: &'static [u8] = concat!($name, "\0").as_bytes();
            const TYPE_ID_CSTR: &'static [u8] = concat!($name, "\0").as_bytes();
        }
    };
}

// --- Array1 TypedExternal (all R native types) ---
impl_te_ndarray!(Array1<i32>, "ndarray::Array1<i32>");
impl_te_ndarray!(Array1<f64>, "ndarray::Array1<f64>");
impl_te_ndarray!(Array1<u8>, "ndarray::Array1<u8>");
impl_te_ndarray!(Array1<RLogical>, "ndarray::Array1<RLogical>");
impl_te_ndarray!(Array1<Rcomplex>, "ndarray::Array1<Rcomplex>");

// --- Array2 TypedExternal (all R native types) ---
impl_te_ndarray!(Array2<i32>, "ndarray::Array2<i32>");
impl_te_ndarray!(Array2<f64>, "ndarray::Array2<f64>");
impl_te_ndarray!(Array2<u8>, "ndarray::Array2<u8>");
impl_te_ndarray!(Array2<RLogical>, "ndarray::Array2<RLogical>");
impl_te_ndarray!(Array2<Rcomplex>, "ndarray::Array2<Rcomplex>");

// --- Array3 TypedExternal (all R native types) ---
impl_te_ndarray!(Array3<i32>, "ndarray::Array3<i32>");
impl_te_ndarray!(Array3<f64>, "ndarray::Array3<f64>");
impl_te_ndarray!(Array3<u8>, "ndarray::Array3<u8>");
impl_te_ndarray!(Array3<RLogical>, "ndarray::Array3<RLogical>");
impl_te_ndarray!(Array3<Rcomplex>, "ndarray::Array3<Rcomplex>");

// --- Array4 TypedExternal (all R native types) ---
impl_te_ndarray!(Array4<i32>, "ndarray::Array4<i32>");
impl_te_ndarray!(Array4<f64>, "ndarray::Array4<f64>");
impl_te_ndarray!(Array4<u8>, "ndarray::Array4<u8>");
impl_te_ndarray!(Array4<RLogical>, "ndarray::Array4<RLogical>");
impl_te_ndarray!(Array4<Rcomplex>, "ndarray::Array4<Rcomplex>");

// --- Array5 TypedExternal (all R native types) ---
impl_te_ndarray!(Array5<i32>, "ndarray::Array5<i32>");
impl_te_ndarray!(Array5<f64>, "ndarray::Array5<f64>");
impl_te_ndarray!(Array5<u8>, "ndarray::Array5<u8>");
impl_te_ndarray!(Array5<RLogical>, "ndarray::Array5<RLogical>");
impl_te_ndarray!(Array5<Rcomplex>, "ndarray::Array5<Rcomplex>");

// --- Array6 TypedExternal (all R native types) ---
impl_te_ndarray!(Array6<i32>, "ndarray::Array6<i32>");
impl_te_ndarray!(Array6<f64>, "ndarray::Array6<f64>");
impl_te_ndarray!(Array6<u8>, "ndarray::Array6<u8>");
impl_te_ndarray!(Array6<RLogical>, "ndarray::Array6<RLogical>");
impl_te_ndarray!(Array6<Rcomplex>, "ndarray::Array6<Rcomplex>");

// --- ArrayD TypedExternal (all R native types) ---
impl_te_ndarray!(ArrayD<i32>, "ndarray::ArrayD<i32>");
impl_te_ndarray!(ArrayD<f64>, "ndarray::ArrayD<f64>");
impl_te_ndarray!(ArrayD<u8>, "ndarray::ArrayD<u8>");
impl_te_ndarray!(ArrayD<RLogical>, "ndarray::ArrayD<RLogical>");
impl_te_ndarray!(ArrayD<Rcomplex>, "ndarray::ArrayD<Rcomplex>");

// --- ArcArray TypedExternal (common types) ---
impl_te_ndarray!(ArcArray1<f64>, "ndarray::ArcArray1<f64>");
impl_te_ndarray!(ArcArray1<i32>, "ndarray::ArcArray1<i32>");
impl_te_ndarray!(ArcArray2<f64>, "ndarray::ArcArray2<f64>");
impl_te_ndarray!(ArcArray2<i32>, "ndarray::ArcArray2<i32>");

// =============================================================================
// RNdArrayOps adapter trait
// =============================================================================

/// Adapter trait for ndarray array operations.
///
/// Provides common array operations accessible from R.
/// Implemented for `Array1<f64>`, `Array2<f64>`, and `ArrayD<f64>`.
///
/// # Example
///
/// ```rust,ignore
/// use ndarray::Array1;
/// use miniextendr_api::ndarray_impl::RNdArrayOps;
///
/// #[derive(ExternalPtr)]
/// struct MyArray(Array1<f64>);
///
/// #[miniextendr]
/// impl RNdArrayOps for MyArray {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RNdArrayOps for MyArray;
/// }
/// ```
///
/// In R:
/// ```r
/// arr <- MyArray$new(c(1, 2, 3, 4, 5))
/// arr$len()          # 5L
/// arr$sum()          # 15.0
/// arr$mean()         # 3.0
/// arr$shape()        # c(5L)
/// ```
pub trait RNdArrayOps {
    /// Get the total number of elements.
    fn len(&self) -> i32;

    /// Check if the array is empty.
    fn is_empty(&self) -> bool;

    /// Get the number of dimensions (ndim).
    fn ndim(&self) -> i32;

    /// Get the shape as a vector of dimensions.
    fn shape(&self) -> Vec<i32>;

    /// Compute the sum of all elements.
    fn sum(&self) -> f64;

    /// Compute the mean of all elements.
    fn mean(&self) -> f64;

    /// Get the minimum element.
    fn min(&self) -> f64;

    /// Get the maximum element.
    fn max(&self) -> f64;

    /// Compute the product of all elements.
    fn product(&self) -> f64;

    /// Compute the variance (population variance, ddof=0).
    fn var(&self) -> f64;

    /// Compute the standard deviation (population, ddof=0).
    fn std(&self) -> f64;
}

impl RNdArrayOps for Array1<f64> {
    fn len(&self) -> i32 {
        Array1::len(self) as i32
    }

    fn is_empty(&self) -> bool {
        Array1::is_empty(self)
    }

    fn ndim(&self) -> i32 {
        Array1::ndim(self) as i32
    }

    fn shape(&self) -> Vec<i32> {
        Array1::shape(self).iter().map(|&d| d as i32).collect()
    }

    fn sum(&self) -> f64 {
        self.iter().sum()
    }

    fn mean(&self) -> f64 {
        if Array1::is_empty(self) {
            f64::NAN
        } else {
            self.iter().sum::<f64>() / Array1::len(self) as f64
        }
    }

    fn min(&self) -> f64 {
        self.iter().copied().fold(f64::INFINITY, f64::min)
    }

    fn max(&self) -> f64 {
        self.iter().copied().fold(f64::NEG_INFINITY, f64::max)
    }

    fn product(&self) -> f64 {
        self.iter().product()
    }

    fn var(&self) -> f64 {
        let n = Array1::len(self) as f64;
        if n == 0.0 {
            return f64::NAN;
        }
        let mean = self.iter().sum::<f64>() / n;
        self.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n
    }

    fn std(&self) -> f64 {
        self.var().sqrt()
    }
}

impl RNdArrayOps for Array2<f64> {
    fn len(&self) -> i32 {
        Array2::len(self) as i32
    }

    fn is_empty(&self) -> bool {
        Array2::is_empty(self)
    }

    fn ndim(&self) -> i32 {
        Array2::ndim(self) as i32
    }

    fn shape(&self) -> Vec<i32> {
        Array2::shape(self).iter().map(|&d| d as i32).collect()
    }

    fn sum(&self) -> f64 {
        self.iter().sum()
    }

    fn mean(&self) -> f64 {
        if Array2::is_empty(self) {
            f64::NAN
        } else {
            self.iter().sum::<f64>() / Array2::len(self) as f64
        }
    }

    fn min(&self) -> f64 {
        self.iter().copied().fold(f64::INFINITY, f64::min)
    }

    fn max(&self) -> f64 {
        self.iter().copied().fold(f64::NEG_INFINITY, f64::max)
    }

    fn product(&self) -> f64 {
        self.iter().product()
    }

    fn var(&self) -> f64 {
        let n = Array2::len(self) as f64;
        if n == 0.0 {
            return f64::NAN;
        }
        let mean = self.iter().sum::<f64>() / n;
        self.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n
    }

    fn std(&self) -> f64 {
        self.var().sqrt()
    }
}

impl RNdArrayOps for ArrayD<f64> {
    fn len(&self) -> i32 {
        ArrayD::len(self) as i32
    }

    fn is_empty(&self) -> bool {
        ArrayD::is_empty(self)
    }

    fn ndim(&self) -> i32 {
        ArrayD::ndim(self) as i32
    }

    fn shape(&self) -> Vec<i32> {
        ArrayD::shape(self).iter().map(|&d| d as i32).collect()
    }

    fn sum(&self) -> f64 {
        self.iter().sum()
    }

    fn mean(&self) -> f64 {
        if ArrayD::is_empty(self) {
            f64::NAN
        } else {
            self.iter().sum::<f64>() / ArrayD::len(self) as f64
        }
    }

    fn min(&self) -> f64 {
        self.iter().copied().fold(f64::INFINITY, f64::min)
    }

    fn max(&self) -> f64 {
        self.iter().copied().fold(f64::NEG_INFINITY, f64::max)
    }

    fn product(&self) -> f64 {
        self.iter().product()
    }

    fn var(&self) -> f64 {
        let n = ArrayD::len(self) as f64;
        if n == 0.0 {
            return f64::NAN;
        }
        let mean = self.iter().sum::<f64>() / n;
        self.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n
    }

    fn std(&self) -> f64 {
        self.var().sqrt()
    }
}

// --- RNdArrayOps for i32 arrays (convert to f64 for stats) ---

impl RNdArrayOps for Array1<i32> {
    fn len(&self) -> i32 {
        Array1::len(self) as i32
    }

    fn is_empty(&self) -> bool {
        Array1::is_empty(self)
    }

    fn ndim(&self) -> i32 {
        Array1::ndim(self) as i32
    }

    fn shape(&self) -> Vec<i32> {
        Array1::shape(self).iter().map(|&d| d as i32).collect()
    }

    fn sum(&self) -> f64 {
        self.iter().map(|&x| x as f64).sum()
    }

    fn mean(&self) -> f64 {
        if Array1::is_empty(self) {
            f64::NAN
        } else {
            self.iter().map(|&x| x as f64).sum::<f64>() / Array1::len(self) as f64
        }
    }

    fn min(&self) -> f64 {
        self.iter().map(|&x| x as f64).fold(f64::INFINITY, f64::min)
    }

    fn max(&self) -> f64 {
        self.iter()
            .map(|&x| x as f64)
            .fold(f64::NEG_INFINITY, f64::max)
    }

    fn product(&self) -> f64 {
        self.iter().map(|&x| x as f64).product()
    }

    fn var(&self) -> f64 {
        let n = Array1::len(self) as f64;
        if n == 0.0 {
            return f64::NAN;
        }
        let mean = self.iter().map(|&x| x as f64).sum::<f64>() / n;
        self.iter().map(|&x| (x as f64 - mean).powi(2)).sum::<f64>() / n
    }

    fn std(&self) -> f64 {
        self.var().sqrt()
    }
}

impl RNdArrayOps for Array2<i32> {
    fn len(&self) -> i32 {
        Array2::len(self) as i32
    }

    fn is_empty(&self) -> bool {
        Array2::is_empty(self)
    }

    fn ndim(&self) -> i32 {
        Array2::ndim(self) as i32
    }

    fn shape(&self) -> Vec<i32> {
        Array2::shape(self).iter().map(|&d| d as i32).collect()
    }

    fn sum(&self) -> f64 {
        self.iter().map(|&x| x as f64).sum()
    }

    fn mean(&self) -> f64 {
        if Array2::is_empty(self) {
            f64::NAN
        } else {
            self.iter().map(|&x| x as f64).sum::<f64>() / Array2::len(self) as f64
        }
    }

    fn min(&self) -> f64 {
        self.iter().map(|&x| x as f64).fold(f64::INFINITY, f64::min)
    }

    fn max(&self) -> f64 {
        self.iter()
            .map(|&x| x as f64)
            .fold(f64::NEG_INFINITY, f64::max)
    }

    fn product(&self) -> f64 {
        self.iter().map(|&x| x as f64).product()
    }

    fn var(&self) -> f64 {
        let n = Array2::len(self) as f64;
        if n == 0.0 {
            return f64::NAN;
        }
        let mean = self.iter().map(|&x| x as f64).sum::<f64>() / n;
        self.iter().map(|&x| (x as f64 - mean).powi(2)).sum::<f64>() / n
    }

    fn std(&self) -> f64 {
        self.var().sqrt()
    }
}

impl RNdArrayOps for ArrayD<i32> {
    fn len(&self) -> i32 {
        ArrayD::len(self) as i32
    }

    fn is_empty(&self) -> bool {
        ArrayD::is_empty(self)
    }

    fn ndim(&self) -> i32 {
        ArrayD::ndim(self) as i32
    }

    fn shape(&self) -> Vec<i32> {
        ArrayD::shape(self).iter().map(|&d| d as i32).collect()
    }

    fn sum(&self) -> f64 {
        self.iter().map(|&x| x as f64).sum()
    }

    fn mean(&self) -> f64 {
        if ArrayD::is_empty(self) {
            f64::NAN
        } else {
            self.iter().map(|&x| x as f64).sum::<f64>() / ArrayD::len(self) as f64
        }
    }

    fn min(&self) -> f64 {
        self.iter().map(|&x| x as f64).fold(f64::INFINITY, f64::min)
    }

    fn max(&self) -> f64 {
        self.iter()
            .map(|&x| x as f64)
            .fold(f64::NEG_INFINITY, f64::max)
    }

    fn product(&self) -> f64 {
        self.iter().map(|&x| x as f64).product()
    }

    fn var(&self) -> f64 {
        let n = ArrayD::len(self) as f64;
        if n == 0.0 {
            return f64::NAN;
        }
        let mean = self.iter().map(|&x| x as f64).sum::<f64>() / n;
        self.iter().map(|&x| (x as f64 - mean).powi(2)).sum::<f64>() / n
    }

    fn std(&self) -> f64 {
        self.var().sqrt()
    }
}

// =============================================================================
// RNdSlice adapter trait for array indexing/slicing
// =============================================================================

/// Adapter trait for ndarray slicing and indexing operations.
///
/// Provides element access and subarray extraction accessible from R.
/// Unlike `RNdArrayOps` which provides aggregate operations, `RNdSlice`
/// focuses on accessing individual elements and extracting subarrays.
///
/// # Example
///
/// ```rust,ignore
/// use ndarray::Array1;
/// use miniextendr_api::ndarray_impl::RNdSlice;
///
/// #[derive(ExternalPtr)]
/// struct MyArray(Array1<f64>);
///
/// #[miniextendr]
/// impl RNdSlice for MyArray {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RNdSlice for MyArray;
/// }
/// ```
///
/// In R:
/// ```r
/// arr <- MyArray$new(c(1, 2, 3, 4, 5))
/// arr$get(2L)              # Get element at index 2 (0-indexed): 3.0
/// arr$slice_1d(1L, 4L)     # Slice [1:4): c(2, 3, 4)
/// arr$first()              # First element: 1.0
/// arr$last()               # Last element: 5.0
/// ```
pub trait RNdSlice {
    /// Element type of the array.
    type Elem: Clone;

    /// Get the element at the given flat index (0-indexed).
    ///
    /// Returns None if index is out of bounds.
    fn get(&self, index: i32) -> Option<Self::Elem>;

    /// Get the first element, or None if empty.
    fn first(&self) -> Option<Self::Elem>;

    /// Get the last element, or None if empty.
    fn last(&self) -> Option<Self::Elem>;

    /// Extract a 1D slice as a new Vec (0-indexed, exclusive end).
    ///
    /// Returns elements in the range [start, end).
    fn slice_1d(&self, start: i32, end: i32) -> Vec<Self::Elem>;

    /// Get elements at the given indices.
    fn get_many(&self, indices: Vec<i32>) -> Vec<Option<Self::Elem>> {
        indices.into_iter().map(|i| self.get(i)).collect()
    }

    /// Check if the given index is valid.
    fn is_valid_index(&self, index: i32) -> bool {
        self.get(index).is_some()
    }
}

impl RNdSlice for Array1<f64> {
    type Elem = f64;

    fn get(&self, index: i32) -> Option<f64> {
        if index < 0 {
            return None;
        }
        self.view().get(index as usize).copied()
    }

    fn first(&self) -> Option<f64> {
        self.view().first().copied()
    }

    fn last(&self) -> Option<f64> {
        self.view().last().copied()
    }

    fn slice_1d(&self, start: i32, end: i32) -> Vec<f64> {
        let start = start.max(0) as usize;
        let end = (end.max(0) as usize).min(self.len());
        if start >= end {
            return Vec::new();
        }
        self.slice(ndarray::s![start..end]).to_vec()
    }
}

impl RNdSlice for Array1<i32> {
    type Elem = i32;

    fn get(&self, index: i32) -> Option<i32> {
        if index < 0 {
            return None;
        }
        self.view().get(index as usize).copied()
    }

    fn first(&self) -> Option<i32> {
        self.view().first().copied()
    }

    fn last(&self) -> Option<i32> {
        self.view().last().copied()
    }

    fn slice_1d(&self, start: i32, end: i32) -> Vec<i32> {
        let start = start.max(0) as usize;
        let end = (end.max(0) as usize).min(self.len());
        if start >= end {
            return Vec::new();
        }
        self.slice(ndarray::s![start..end]).to_vec()
    }
}

/// Adapter trait for 2D array row/column access.
///
/// Provides row and column extraction for matrices.
///
/// # Example
///
/// ```rust,ignore
/// use ndarray::Array2;
/// use miniextendr_api::ndarray_impl::RNdSlice2D;
///
/// #[derive(ExternalPtr)]
/// struct MyMatrix(Array2<f64>);
///
/// #[miniextendr]
/// impl RNdSlice2D for MyMatrix {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RNdSlice2D for MyMatrix;
/// }
/// ```
///
/// In R:
/// ```r
/// mat <- MyMatrix$new(matrix(1:6, nrow=2, ncol=3))
/// mat$row(0L)     # First row: c(1, 3, 5)
/// mat$col(1L)     # Second column: c(3, 4)
/// mat$get_2d(0L, 1L)  # Element at [0,1]: 3
/// ```
pub trait RNdSlice2D {
    /// Element type of the array.
    type Elem: Clone;

    /// Get the element at [row, col] (0-indexed).
    fn get_2d(&self, row: i32, col: i32) -> Option<Self::Elem>;

    /// Get a row as a vector.
    fn row(&self, row: i32) -> Vec<Self::Elem>;

    /// Get a column as a vector.
    fn col(&self, col: i32) -> Vec<Self::Elem>;

    /// Get the diagonal elements.
    fn diag(&self) -> Vec<Self::Elem>;

    /// Get the number of rows.
    fn nrows(&self) -> i32;

    /// Get the number of columns.
    fn ncols(&self) -> i32;
}

impl RNdSlice2D for Array2<f64> {
    type Elem = f64;

    fn get_2d(&self, row: i32, col: i32) -> Option<f64> {
        if row < 0 || col < 0 {
            return None;
        }
        self.get((row as usize, col as usize)).copied()
    }

    fn row(&self, row: i32) -> Vec<f64> {
        let (nrows, _) = self.dim();
        if row < 0 || row as usize >= nrows {
            return Vec::new();
        }
        // Use view() to get around name collision with trait method
        self.view().row(row as usize).to_vec()
    }

    fn col(&self, col: i32) -> Vec<f64> {
        let (_, ncols) = self.dim();
        if col < 0 || col as usize >= ncols {
            return Vec::new();
        }
        self.view().column(col as usize).to_vec()
    }

    fn diag(&self) -> Vec<f64> {
        // Use diag() from ndarray - need to avoid name collision
        let view = self.view();
        view.diag().to_vec()
    }

    fn nrows(&self) -> i32 {
        self.view().nrows() as i32
    }

    fn ncols(&self) -> i32 {
        self.view().ncols() as i32
    }
}

impl RNdSlice2D for Array2<i32> {
    type Elem = i32;

    fn get_2d(&self, row: i32, col: i32) -> Option<i32> {
        if row < 0 || col < 0 {
            return None;
        }
        self.get((row as usize, col as usize)).copied()
    }

    fn row(&self, row: i32) -> Vec<i32> {
        let (nrows, _) = self.dim();
        if row < 0 || row as usize >= nrows {
            return Vec::new();
        }
        // Use view() to get around name collision with trait method
        self.view().row(row as usize).to_vec()
    }

    fn col(&self, col: i32) -> Vec<i32> {
        let (_, ncols) = self.dim();
        if col < 0 || col as usize >= ncols {
            return Vec::new();
        }
        self.view().column(col as usize).to_vec()
    }

    fn diag(&self) -> Vec<i32> {
        // Use diag() from ndarray - need to avoid name collision
        let view = self.view();
        view.diag().to_vec()
    }

    fn nrows(&self) -> i32 {
        self.view().nrows() as i32
    }

    fn ncols(&self) -> i32 {
        self.view().ncols() as i32
    }
}

// =============================================================================
// RNdIndex adapter trait for n-dimensional indexing
// =============================================================================

/// Adapter trait for n-dimensional array indexing and slicing.
///
/// Provides R-style array subsetting for `ArrayD` (dynamic dimension arrays).
/// Unlike `RNdSlice` (1D) and `RNdSlice2D` (2D), this trait handles arrays
/// of arbitrary dimension.
///
/// # Example
///
/// ```rust,ignore
/// use ndarray::ArrayD;
/// use miniextendr_api::ndarray_impl::RNdIndex;
///
/// #[derive(ExternalPtr)]
/// struct MyNdArray(ArrayD<f64>);
///
/// #[miniextendr]
/// impl RNdIndex for MyNdArray {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RNdIndex for MyNdArray;
/// }
/// ```
///
/// In R:
/// ```r
/// # Create a 3D array (2x3x4)
/// arr <- MyNdArray$new(array(1:24, dim = c(2, 3, 4)))
/// arr$get_nd(c(0L, 1L, 2L))     # Element at [0,1,2]
/// arr$slice_nd(c(0L, 0L, 0L), c(2L, 2L, 2L))  # Subarray [0:2, 0:2, 0:2]
/// arr$flatten()                 # Flatten to 1D vector
/// ```
pub trait RNdIndex {
    /// Element type of the array.
    type Elem: Clone;

    /// Get the element at the given n-dimensional index (0-indexed).
    ///
    /// Returns None if the index is out of bounds or has wrong dimensionality.
    fn get_nd(&self, indices: Vec<i32>) -> Option<Self::Elem>;

    /// Extract a subarray from start (inclusive) to end (exclusive).
    ///
    /// Both `start` and `end` must have the same length as the array's ndim.
    /// Returns None if bounds are invalid.
    fn slice_nd(&self, start: Vec<i32>, end: Vec<i32>) -> Option<Vec<Self::Elem>>;

    /// Get the shape of the array.
    fn shape_nd(&self) -> Vec<i32>;

    /// Get the number of dimensions.
    fn ndim(&self) -> i32;

    /// Get the total number of elements.
    fn len_nd(&self) -> i32;

    /// Flatten the array to a 1D vector in Fortran (column-major) order.
    ///
    /// This matches R's default array storage order.
    fn flatten(&self) -> Vec<Self::Elem>;

    /// Flatten the array to a 1D vector in C (row-major) order.
    fn flatten_c(&self) -> Vec<Self::Elem>;

    /// Check if the given index is valid.
    fn is_valid_nd(&self, indices: Vec<i32>) -> bool {
        self.get_nd(indices).is_some()
    }

    /// Get elements along a specific axis at the given index.
    ///
    /// Returns elements where the specified axis is fixed at `index`.
    fn axis_slice(&self, axis: i32, index: i32) -> Vec<Self::Elem>;

    /// Reshape the array to new dimensions (data must fit exactly).
    ///
    /// Returns None if the total element count doesn't match.
    fn reshape(&self, new_shape: Vec<i32>) -> Option<Vec<Self::Elem>>;
}

impl RNdIndex for ArrayD<f64> {
    type Elem = f64;

    fn get_nd(&self, indices: Vec<i32>) -> Option<f64> {
        if indices.len() != self.ndim() {
            return None;
        }
        // Check for negative indices
        if indices.iter().any(|&i| i < 0) {
            return None;
        }
        let idx: Vec<usize> = indices.iter().map(|&i| i as usize).collect();
        self.get(IxDyn(&idx)).copied()
    }

    fn slice_nd(&self, start: Vec<i32>, end: Vec<i32>) -> Option<Vec<f64>> {
        let ndim = self.ndim();
        if start.len() != ndim || end.len() != ndim {
            return None;
        }

        let shape = self.shape();
        let mut result = Vec::new();

        // Convert to usize and clamp
        let start_usize: Vec<usize> = start
            .iter()
            .enumerate()
            .map(|(i, &s)| (s.max(0) as usize).min(shape[i]))
            .collect();
        let end_usize: Vec<usize> = end
            .iter()
            .enumerate()
            .map(|(i, &e)| (e.max(0) as usize).min(shape[i]))
            .collect();

        // Check if range is valid
        for i in 0..ndim {
            if start_usize[i] >= end_usize[i] {
                return Some(Vec::new());
            }
        }

        // Calculate subarray shape
        let sub_shape: Vec<usize> = (0..ndim).map(|i| end_usize[i] - start_usize[i]).collect();

        // Iterate in Fortran order (column-major)
        fortran_order_iter(&sub_shape, |sub_idx| {
            let full_idx: Vec<usize> = sub_idx
                .iter()
                .enumerate()
                .map(|(i, &s)| s + start_usize[i])
                .collect();
            if let Some(&val) = self.get(IxDyn(&full_idx)) {
                result.push(val);
            }
        });

        Some(result)
    }

    fn shape_nd(&self) -> Vec<i32> {
        self.shape().iter().map(|&d| d as i32).collect()
    }

    fn ndim(&self) -> i32 {
        self.ndim() as i32
    }

    fn len_nd(&self) -> i32 {
        self.len() as i32
    }

    fn flatten(&self) -> Vec<f64> {
        // Iterate in Fortran order to match R
        let shape: Vec<usize> = self.shape().to_vec();
        let mut result = Vec::with_capacity(self.len());
        fortran_order_iter(&shape, |idx| {
            if let Some(&val) = self.get(IxDyn(&idx)) {
                result.push(val);
            }
        });
        result
    }

    fn flatten_c(&self) -> Vec<f64> {
        // Standard iteration (C order)
        self.iter().copied().collect()
    }

    fn axis_slice(&self, axis: i32, index: i32) -> Vec<f64> {
        let ndim = self.ndim();
        if axis < 0 || axis as usize >= ndim {
            return Vec::new();
        }
        let shape = self.shape();
        if index < 0 || index as usize >= shape[axis as usize] {
            return Vec::new();
        }

        let axis_usize = axis as usize;
        let index_usize = index as usize;

        // Build shape for the result (all dims except the sliced axis)
        let result_shape: Vec<usize> = shape
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != axis_usize)
            .map(|(_, &d)| d)
            .collect();

        let mut result = Vec::new();

        // Iterate over the result shape
        fortran_order_iter(&result_shape, |sub_idx| {
            // Build full index by inserting the fixed axis value
            let mut full_idx = Vec::with_capacity(ndim);
            let mut sub_i = 0;
            for i in 0..ndim {
                if i == axis_usize {
                    full_idx.push(index_usize);
                } else {
                    full_idx.push(sub_idx[sub_i]);
                    sub_i += 1;
                }
            }
            if let Some(&val) = self.get(IxDyn(&full_idx)) {
                result.push(val);
            }
        });

        result
    }

    fn reshape(&self, new_shape: Vec<i32>) -> Option<Vec<f64>> {
        let new_len: usize = new_shape.iter().map(|&d| d.max(0) as usize).product();
        if new_len != self.len() {
            return None;
        }
        // Return flattened data - reshaping is just reinterpretation
        Some(RNdIndex::flatten(self))
    }
}

impl RNdIndex for ArrayD<i32> {
    type Elem = i32;

    fn get_nd(&self, indices: Vec<i32>) -> Option<i32> {
        if indices.len() != self.ndim() {
            return None;
        }
        if indices.iter().any(|&i| i < 0) {
            return None;
        }
        let idx: Vec<usize> = indices.iter().map(|&i| i as usize).collect();
        self.get(IxDyn(&idx)).copied()
    }

    fn slice_nd(&self, start: Vec<i32>, end: Vec<i32>) -> Option<Vec<i32>> {
        let ndim = self.ndim();
        if start.len() != ndim || end.len() != ndim {
            return None;
        }

        let shape = self.shape();
        let mut result = Vec::new();

        let start_usize: Vec<usize> = start
            .iter()
            .enumerate()
            .map(|(i, &s)| (s.max(0) as usize).min(shape[i]))
            .collect();
        let end_usize: Vec<usize> = end
            .iter()
            .enumerate()
            .map(|(i, &e)| (e.max(0) as usize).min(shape[i]))
            .collect();

        for i in 0..ndim {
            if start_usize[i] >= end_usize[i] {
                return Some(Vec::new());
            }
        }

        let sub_shape: Vec<usize> = (0..ndim).map(|i| end_usize[i] - start_usize[i]).collect();

        fortran_order_iter(&sub_shape, |sub_idx| {
            let full_idx: Vec<usize> = sub_idx
                .iter()
                .enumerate()
                .map(|(i, &s)| s + start_usize[i])
                .collect();
            if let Some(&val) = self.get(IxDyn(&full_idx)) {
                result.push(val);
            }
        });

        Some(result)
    }

    fn shape_nd(&self) -> Vec<i32> {
        self.shape().iter().map(|&d| d as i32).collect()
    }

    fn ndim(&self) -> i32 {
        self.ndim() as i32
    }

    fn len_nd(&self) -> i32 {
        self.len() as i32
    }

    fn flatten(&self) -> Vec<i32> {
        let shape: Vec<usize> = self.shape().to_vec();
        let mut result = Vec::with_capacity(self.len());
        fortran_order_iter(&shape, |idx| {
            if let Some(&val) = self.get(IxDyn(&idx)) {
                result.push(val);
            }
        });
        result
    }

    fn flatten_c(&self) -> Vec<i32> {
        self.iter().copied().collect()
    }

    fn axis_slice(&self, axis: i32, index: i32) -> Vec<i32> {
        let ndim = self.ndim();
        if axis < 0 || axis as usize >= ndim {
            return Vec::new();
        }
        let shape = self.shape();
        if index < 0 || index as usize >= shape[axis as usize] {
            return Vec::new();
        }

        let axis_usize = axis as usize;
        let index_usize = index as usize;

        let result_shape: Vec<usize> = shape
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != axis_usize)
            .map(|(_, &d)| d)
            .collect();

        let mut result = Vec::new();

        fortran_order_iter(&result_shape, |sub_idx| {
            let mut full_idx = Vec::with_capacity(ndim);
            let mut sub_i = 0;
            for i in 0..ndim {
                if i == axis_usize {
                    full_idx.push(index_usize);
                } else {
                    full_idx.push(sub_idx[sub_i]);
                    sub_i += 1;
                }
            }
            if let Some(&val) = self.get(IxDyn(&full_idx)) {
                result.push(val);
            }
        });

        result
    }

    fn reshape(&self, new_shape: Vec<i32>) -> Option<Vec<i32>> {
        let new_len: usize = new_shape.iter().map(|&d| d.max(0) as usize).product();
        if new_len != self.len() {
            return None;
        }
        Some(RNdIndex::flatten(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn array1_can_be_created() {
        let arr = Array1::from_vec(vec![1.0, 2.0, 3.0]);
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn array2_can_be_created() {
        let arr = Array2::from_shape_vec((2, 3), vec![1, 2, 3, 4, 5, 6]).unwrap();
        assert_eq!(arr.dim(), (2, 3));
    }

    #[test]
    fn array3_can_be_created() {
        // 2x3x4 = 24 elements
        let data: Vec<f64> = (0..24).map(|x| x as f64).collect();
        let arr = Array3::from_shape_vec((2, 3, 4), data).unwrap();
        assert_eq!(arr.dim(), (2, 3, 4));
        assert_eq!(arr[[0, 0, 0]], 0.0);
        assert_eq!(arr[[1, 2, 3]], 23.0);
    }

    #[test]
    fn array3_fortran_order() {
        // Test that Fortran order works correctly
        let data: Vec<i32> = (0..24).collect();
        let arr = Array3::from_shape_vec((2, 3, 4).f(), data).unwrap();
        assert_eq!(arr.dim(), (2, 3, 4));
        // In Fortran order, first dimension varies fastest
        // So arr[0,0,0]=0, arr[1,0,0]=1, arr[0,1,0]=2, ...
        assert_eq!(arr[[0, 0, 0]], 0);
        assert_eq!(arr[[1, 0, 0]], 1);
        assert_eq!(arr[[0, 1, 0]], 2);
        assert_eq!(arr[[1, 1, 0]], 3);
    }

    #[test]
    fn arrayd_can_be_created() {
        // Create a 4D array: 2x3x4x5 = 120 elements
        let data: Vec<f64> = (0..120).map(|x| x as f64).collect();
        let arr = ArrayD::from_shape_vec(IxDyn(&[2, 3, 4, 5]), data).unwrap();
        assert_eq!(arr.shape(), &[2, 3, 4, 5]);
        assert_eq!(arr.ndim(), 4);
    }

    #[test]
    fn arrayd_fortran_order() {
        // Test dynamic dimension array in Fortran order
        let data: Vec<i32> = (0..12).collect();
        let arr = ArrayD::from_shape_vec(IxDyn(&[2, 2, 3]).f(), data).unwrap();
        assert_eq!(arr.shape(), &[2, 2, 3]);
        // In Fortran order, first dimension varies fastest
        assert_eq!(arr[IxDyn(&[0, 0, 0])], 0);
        assert_eq!(arr[IxDyn(&[1, 0, 0])], 1);
        assert_eq!(arr[IxDyn(&[0, 1, 0])], 2);
    }

    #[test]
    fn fortran_order_iter_generates_correct_indices() {
        let mut indices = Vec::new();
        fortran_order_iter(&[2, 3], |idx| {
            indices.push(idx);
        });
        // Fortran order: first index varies fastest
        assert_eq!(
            indices,
            vec![
                vec![0, 0],
                vec![1, 0],
                vec![0, 1],
                vec![1, 1],
                vec![0, 2],
                vec![1, 2],
            ]
        );
    }

    #[test]
    fn fortran_order_iter_3d() {
        let mut indices = Vec::new();
        fortran_order_iter(&[2, 2, 2], |idx| {
            indices.push(idx);
        });
        // Should be: [0,0,0], [1,0,0], [0,1,0], [1,1,0], [0,0,1], [1,0,1], [0,1,1], [1,1,1]
        assert_eq!(indices.len(), 8);
        assert_eq!(indices[0], vec![0, 0, 0]);
        assert_eq!(indices[1], vec![1, 0, 0]);
        assert_eq!(indices[4], vec![0, 0, 1]);
        assert_eq!(indices[7], vec![1, 1, 1]);
    }

    // RNdArrayOps tests
    #[test]
    fn rndarrayops_array1_basic() {
        let arr = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(RNdArrayOps::len(&arr), 5);
        assert!(!RNdArrayOps::is_empty(&arr));
        assert_eq!(RNdArrayOps::ndim(&arr), 1);
        assert_eq!(RNdArrayOps::shape(&arr), vec![5]);
    }

    #[test]
    fn rndarrayops_array1_stats() {
        let arr = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!((RNdArrayOps::sum(&arr) - 15.0).abs() < 1e-10);
        assert!((RNdArrayOps::mean(&arr) - 3.0).abs() < 1e-10);
        assert!((RNdArrayOps::min(&arr) - 1.0).abs() < 1e-10);
        assert!((RNdArrayOps::max(&arr) - 5.0).abs() < 1e-10);
        assert!((RNdArrayOps::product(&arr) - 120.0).abs() < 1e-10);
    }

    #[test]
    fn rndarrayops_array1_var_std() {
        let arr = Array1::from_vec(vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
        // Mean = 5.0, variance = 4.0, std = 2.0
        assert!((RNdArrayOps::mean(&arr) - 5.0).abs() < 1e-10);
        assert!((RNdArrayOps::var(&arr) - 4.0).abs() < 1e-10);
        assert!((RNdArrayOps::std(&arr) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn rndarrayops_array2_basic() {
        let arr = Array2::from_shape_vec((2, 3), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
        assert_eq!(RNdArrayOps::len(&arr), 6);
        assert_eq!(RNdArrayOps::ndim(&arr), 2);
        assert_eq!(RNdArrayOps::shape(&arr), vec![2, 3]);
        assert!((RNdArrayOps::sum(&arr) - 21.0).abs() < 1e-10);
        assert!((RNdArrayOps::mean(&arr) - 3.5).abs() < 1e-10);
    }

    #[test]
    fn rndarrayops_arrayd_basic() {
        let arr = ArrayD::from_shape_vec(IxDyn(&[2, 2, 2]), vec![1.0; 8]).unwrap();
        assert_eq!(RNdArrayOps::len(&arr), 8);
        assert_eq!(RNdArrayOps::ndim(&arr), 3);
        assert_eq!(RNdArrayOps::shape(&arr), vec![2, 2, 2]);
        assert!((RNdArrayOps::sum(&arr) - 8.0).abs() < 1e-10);
        assert!((RNdArrayOps::mean(&arr) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rndarrayops_empty() {
        let arr: Array1<f64> = Array1::from_vec(vec![]);
        assert_eq!(RNdArrayOps::len(&arr), 0);
        assert!(RNdArrayOps::is_empty(&arr));
        assert!(RNdArrayOps::mean(&arr).is_nan());
        assert!(RNdArrayOps::var(&arr).is_nan());
    }

    // RNdSlice tests
    #[test]
    fn rndslice_get() {
        let arr = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(RNdSlice::get(&arr, 0), Some(1.0));
        assert_eq!(RNdSlice::get(&arr, 2), Some(3.0));
        assert_eq!(RNdSlice::get(&arr, 4), Some(5.0));
        assert_eq!(RNdSlice::get(&arr, 5), None);
        assert_eq!(RNdSlice::get(&arr, -1), None);
    }

    #[test]
    fn rndslice_first_last() {
        let arr = Array1::from_vec(vec![10.0, 20.0, 30.0]);
        assert_eq!(RNdSlice::first(&arr), Some(10.0));
        assert_eq!(RNdSlice::last(&arr), Some(30.0));

        let empty: Array1<f64> = Array1::from_vec(vec![]);
        assert_eq!(RNdSlice::first(&empty), None);
        assert_eq!(RNdSlice::last(&empty), None);
    }

    #[test]
    fn rndslice_slice_1d() {
        let arr = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(RNdSlice::slice_1d(&arr, 1, 4), vec![2.0, 3.0, 4.0]);
        assert_eq!(RNdSlice::slice_1d(&arr, 0, 2), vec![1.0, 2.0]);
        assert_eq!(RNdSlice::slice_1d(&arr, 3, 10), vec![4.0, 5.0]); // Clamped
        assert_eq!(RNdSlice::slice_1d(&arr, -5, 2), vec![1.0, 2.0]); // Negative start clamped
        assert_eq!(RNdSlice::slice_1d(&arr, 3, 2), Vec::<f64>::new()); // Empty range
    }

    #[test]
    fn rndslice_get_many() {
        let arr = Array1::from_vec(vec![10, 20, 30, 40, 50]);
        let results = RNdSlice::get_many(&arr, vec![0, 2, 4, 10]);
        assert_eq!(results, vec![Some(10), Some(30), Some(50), None]);
    }

    #[test]
    fn rndslice_is_valid_index() {
        let arr = Array1::from_vec(vec![1.0, 2.0, 3.0]);
        assert!(RNdSlice::is_valid_index(&arr, 0));
        assert!(RNdSlice::is_valid_index(&arr, 2));
        assert!(!RNdSlice::is_valid_index(&arr, 3));
        assert!(!RNdSlice::is_valid_index(&arr, -1));
    }

    // RNdSlice2D tests
    #[test]
    fn rndslice2d_get_2d() {
        let arr = Array2::from_shape_vec((2, 3), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
        assert_eq!(RNdSlice2D::get_2d(&arr, 0, 0), Some(1.0));
        assert_eq!(RNdSlice2D::get_2d(&arr, 0, 2), Some(3.0));
        assert_eq!(RNdSlice2D::get_2d(&arr, 1, 1), Some(5.0));
        assert_eq!(RNdSlice2D::get_2d(&arr, 2, 0), None);
        assert_eq!(RNdSlice2D::get_2d(&arr, -1, 0), None);
    }

    #[test]
    fn rndslice2d_row_col() {
        let arr = Array2::from_shape_vec((2, 3), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
        assert_eq!(RNdSlice2D::row(&arr, 0), vec![1.0, 2.0, 3.0]);
        assert_eq!(RNdSlice2D::row(&arr, 1), vec![4.0, 5.0, 6.0]);
        assert_eq!(RNdSlice2D::row(&arr, 2), Vec::<f64>::new()); // Out of bounds
        assert_eq!(RNdSlice2D::col(&arr, 0), vec![1.0, 4.0]);
        assert_eq!(RNdSlice2D::col(&arr, 1), vec![2.0, 5.0]);
        assert_eq!(RNdSlice2D::col(&arr, 3), Vec::<f64>::new()); // Out of bounds
    }

    #[test]
    fn rndslice2d_diag() {
        let arr = Array2::from_shape_vec((3, 3), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0])
            .unwrap();
        assert_eq!(RNdSlice2D::diag(&arr), vec![1.0, 5.0, 9.0]);
    }

    #[test]
    fn rndslice2d_nrows_ncols() {
        let arr = Array2::from_shape_vec((2, 4), vec![1.0; 8]).unwrap();
        assert_eq!(RNdSlice2D::nrows(&arr), 2);
        assert_eq!(RNdSlice2D::ncols(&arr), 4);
    }

    // RNdIndex tests
    #[test]
    fn rndindex_get_nd() {
        // 2x3x2 array with values 1..12
        let data: Vec<f64> = (1..=12).map(|x| x as f64).collect();
        let arr = ArrayD::from_shape_vec(IxDyn(&[2, 3, 2]), data).unwrap();

        // Test valid indices
        assert_eq!(RNdIndex::get_nd(&arr, vec![0, 0, 0]), Some(1.0));
        assert_eq!(RNdIndex::get_nd(&arr, vec![0, 0, 1]), Some(2.0));
        assert_eq!(RNdIndex::get_nd(&arr, vec![1, 2, 1]), Some(12.0));

        // Test out of bounds
        assert_eq!(RNdIndex::get_nd(&arr, vec![2, 0, 0]), None);
        assert_eq!(RNdIndex::get_nd(&arr, vec![0, 3, 0]), None);

        // Test wrong number of indices
        assert_eq!(RNdIndex::get_nd(&arr, vec![0, 0]), None);
        assert_eq!(RNdIndex::get_nd(&arr, vec![0, 0, 0, 0]), None);

        // Test negative indices
        assert_eq!(RNdIndex::get_nd(&arr, vec![-1, 0, 0]), None);
    }

    #[test]
    fn rndindex_shape_ndim_len() {
        let arr = ArrayD::from_shape_vec(IxDyn(&[2, 3, 4]), vec![0.0; 24]).unwrap();

        assert_eq!(RNdIndex::shape_nd(&arr), vec![2, 3, 4]);
        assert_eq!(RNdIndex::ndim(&arr), 3);
        assert_eq!(RNdIndex::len_nd(&arr), 24);
    }

    #[test]
    fn rndindex_slice_nd() {
        // 3x3 array
        let data: Vec<f64> = (1..=9).map(|x| x as f64).collect();
        let arr = ArrayD::from_shape_vec(IxDyn(&[3, 3]), data).unwrap();

        // Slice [0:2, 0:2] - 2x2 subarray
        let slice = RNdIndex::slice_nd(&arr, vec![0, 0], vec![2, 2]);
        assert!(slice.is_some());
        let slice = slice.unwrap();
        // In Fortran order (column-major): [0,0]=1, [1,0]=4, [0,1]=2, [1,1]=5
        assert_eq!(slice, vec![1.0, 4.0, 2.0, 5.0]);

        // Invalid slice (wrong dimensions)
        assert_eq!(RNdIndex::slice_nd(&arr, vec![0], vec![2, 2]), None);

        // Empty slice (start >= end)
        let empty = RNdIndex::slice_nd(&arr, vec![2, 2], vec![1, 1]);
        assert!(empty.is_some());
        assert!(empty.unwrap().is_empty());
    }

    #[test]
    fn rndindex_flatten() {
        // 2x3 array: [[1,2,3], [4,5,6]]
        let data: Vec<f64> = (1..=6).map(|x| x as f64).collect();
        let arr = ArrayD::from_shape_vec(IxDyn(&[2, 3]), data).unwrap();

        // Fortran order (column-major, R-compatible)
        let f_order = RNdIndex::flatten(&arr);
        assert_eq!(f_order, vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);

        // C order (row-major)
        let c_order = RNdIndex::flatten_c(&arr);
        assert_eq!(c_order, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn rndindex_is_valid_nd() {
        let arr = ArrayD::from_shape_vec(IxDyn(&[2, 3]), vec![0.0; 6]).unwrap();

        assert!(RNdIndex::is_valid_nd(&arr, vec![0, 0]));
        assert!(RNdIndex::is_valid_nd(&arr, vec![1, 2]));
        assert!(!RNdIndex::is_valid_nd(&arr, vec![2, 0]));
        assert!(!RNdIndex::is_valid_nd(&arr, vec![0, 3]));
        assert!(!RNdIndex::is_valid_nd(&arr, vec![0])); // Wrong dim count
        assert!(!RNdIndex::is_valid_nd(&arr, vec![-1, 0])); // Negative
    }

    #[test]
    fn rndindex_axis_slice() {
        // 2x3 array: [[1,2,3], [4,5,6]]
        let data: Vec<f64> = (1..=6).map(|x| x as f64).collect();
        let arr = ArrayD::from_shape_vec(IxDyn(&[2, 3]), data).unwrap();

        // Get row 0 (axis 0, index 0)
        let row0 = RNdIndex::axis_slice(&arr, 0, 0);
        assert_eq!(row0, vec![1.0, 2.0, 3.0]);

        // Get row 1 (axis 0, index 1)
        let row1 = RNdIndex::axis_slice(&arr, 0, 1);
        assert_eq!(row1, vec![4.0, 5.0, 6.0]);

        // Get col 1 (axis 1, index 1)
        let col1 = RNdIndex::axis_slice(&arr, 1, 1);
        assert_eq!(col1, vec![2.0, 5.0]);

        // Invalid axis
        let invalid = RNdIndex::axis_slice(&arr, 2, 0);
        assert!(invalid.is_empty());

        // Invalid index
        let invalid = RNdIndex::axis_slice(&arr, 0, 5);
        assert!(invalid.is_empty());
    }

    #[test]
    fn rndindex_reshape() {
        // 2x3 array flattened and reshaped to 3x2
        let data: Vec<f64> = (1..=6).map(|x| x as f64).collect();
        let arr = ArrayD::from_shape_vec(IxDyn(&[2, 3]), data).unwrap();

        // Valid reshape
        let reshaped = RNdIndex::reshape(&arr, vec![3, 2]);
        assert!(reshaped.is_some());
        assert_eq!(reshaped.unwrap().len(), 6);

        // Reshape to 1D
        let flat = RNdIndex::reshape(&arr, vec![6]);
        assert!(flat.is_some());
        assert_eq!(flat.unwrap().len(), 6);

        // Invalid reshape (wrong total size)
        let invalid = RNdIndex::reshape(&arr, vec![2, 2]);
        assert!(invalid.is_none());

        // Invalid reshape (zero dimension)
        let invalid = RNdIndex::reshape(&arr, vec![0, 6]);
        assert!(invalid.is_none());

        // Invalid reshape (negative dimension)
        let invalid = RNdIndex::reshape(&arr, vec![-1, 6]);
        assert!(invalid.is_none());
    }

    #[test]
    fn rndindex_i32_impl() {
        // Test i32 implementation
        let arr = ArrayD::from_shape_vec(IxDyn(&[2, 2]), vec![1i32, 2, 3, 4]).unwrap();

        assert_eq!(RNdIndex::get_nd(&arr, vec![0, 0]), Some(1));
        assert_eq!(RNdIndex::get_nd(&arr, vec![1, 1]), Some(4));
        assert_eq!(RNdIndex::shape_nd(&arr), vec![2, 2]);
        assert_eq!(RNdIndex::ndim(&arr), 2);
        assert_eq!(RNdIndex::len_nd(&arr), 4);
    }
}
