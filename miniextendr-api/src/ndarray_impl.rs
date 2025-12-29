//! Integration with the `ndarray` crate.
//!
//! This module provides conversions between R vectors/matrices and ndarray types:
//!
//! | R Type | ndarray Type | Notes |
//! |--------|--------------|-------|
//! | INTSXP | `Array1<i32>`, `Array2<i32>` | Integer vectors/matrices |
//! | REALSXP | `Array1<f64>`, `Array2<f64>` | Numeric vectors/matrices |
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
//! R matrices are stored in **column-major** order (Fortran style).
//! ndarray's default is **row-major** (C style).
//!
//! When converting:
//! - **R → ndarray**: Data is copied and the array is created in **column-major**
//!   (Fortran) layout to match R's storage. This preserves the element ordering
//!   but means `array.is_standard_layout()` returns `false`.
//! - **ndarray → R**: Data is copied into column-major R storage. If the array
//!   is already column-major, this is a direct copy. If row-major, elements are
//!   reordered via transpose.
//!
//! For zero-copy access, use `ArrayView` types (see [`from_r_slice`]).
//!
//! # Example
//!
//! ```ignore
//! use ndarray::{Array1, Array2};
//!
//! #[miniextendr]
//! fn sum_array(arr: Array1<f64>) -> f64 {
//!     arr.sum()
//! }
//!
//! #[miniextendr]
//! fn matrix_sum(mat: Array2<f64>) -> f64 {
//!     mat.sum()
//! }
//! ```

pub use ndarray::{Array1, Array2, ArrayView1, ArrayView2, ShapeBuilder};

use crate::ffi::{RNativeType, SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, SexpLengthError, SexpTypeError, TryFromSexp};
use crate::into_r::IntoR;

// =============================================================================
// Array1 conversions
// =============================================================================

/// Convert R vector to `Array1<T>`.
impl<T: RNativeType + Clone> TryFromSexp for Array1<T> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
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

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != T::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: T::SEXP_TYPE,
                actual,
            }
            .into());
        }

        let slice: &[T] = unsafe { sexp.as_slice_unchecked() };
        Ok(Array1::from_vec(slice.to_vec()))
    }
}

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
// Array2 conversions
// =============================================================================

/// Convert R matrix to `Array2<T>`.
///
/// R stores matrices in column-major order. This function reads the data
/// and creates an ndarray with the correct shape.
impl<T: RNativeType + Clone> TryFromSexp for Array2<T> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != T::SEXP_TYPE {
            return Err(SexpTypeError {
                expected: T::SEXP_TYPE,
                actual,
            }
            .into());
        }

        // Get dimensions
        let dims = get_matrix_dims(sexp)?;
        let (nrow, ncol) = dims;

        let slice: &[T] = unsafe { sexp.as_slice() };

        // R stores in column-major order, so we create in Fortran order
        // then can optionally convert to standard order
        let arr = Array2::from_shape_vec((nrow, ncol).f(), slice.to_vec()).map_err(|_| {
            SexpLengthError {
                expected: nrow * ncol,
                actual: slice.len(),
            }
        })?;

        Ok(arr)
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        // Type check is still needed for safety
        Self::try_from_sexp(sexp)
    }
}

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

        // Create R matrix
        unsafe {
            let mat = crate::ffi::Rf_allocMatrix(T::SEXP_TYPE, nrow as i32, ncol as i32);
            crate::ffi::Rf_protect(mat);

            let ptr = crate::ffi::DATAPTR_RO(mat) as *mut T;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());

            crate::ffi::Rf_unprotect(1);
            mat
        }
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// Get matrix dimensions from R object.
fn get_matrix_dims(sexp: SEXP) -> Result<(usize, usize), SexpError> {
    unsafe {
        let dim = crate::ffi::Rf_getAttrib(sexp, crate::ffi::R_DimSymbol);
        if dim.type_of() != SEXPTYPE::INTSXP {
            // Not a matrix, treat as column vector
            return Ok((sexp.len(), 1));
        }

        let dim_slice: &[i32] = dim.as_slice();
        if dim_slice.len() != 2 {
            return Err(SexpLengthError {
                expected: 2,
                actual: dim_slice.len(),
            }
            .into());
        }

        Ok((dim_slice[0] as usize, dim_slice[1] as usize))
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
use crate::ffi::{RLogical, Rcomplex};

// --- Array1 TypedExternal (all R native types) ---

impl TypedExternal for Array1<i32> {
    const TYPE_NAME: &'static str = "ndarray::Array1<i32>";
    const TYPE_NAME_CSTR: &'static [u8] = b"ndarray::Array1<i32>\0";
}

impl TypedExternal for Array1<f64> {
    const TYPE_NAME: &'static str = "ndarray::Array1<f64>";
    const TYPE_NAME_CSTR: &'static [u8] = b"ndarray::Array1<f64>\0";
}

impl TypedExternal for Array1<u8> {
    const TYPE_NAME: &'static str = "ndarray::Array1<u8>";
    const TYPE_NAME_CSTR: &'static [u8] = b"ndarray::Array1<u8>\0";
}

impl TypedExternal for Array1<RLogical> {
    const TYPE_NAME: &'static str = "ndarray::Array1<RLogical>";
    const TYPE_NAME_CSTR: &'static [u8] = b"ndarray::Array1<RLogical>\0";
}

impl TypedExternal for Array1<Rcomplex> {
    const TYPE_NAME: &'static str = "ndarray::Array1<Rcomplex>";
    const TYPE_NAME_CSTR: &'static [u8] = b"ndarray::Array1<Rcomplex>\0";
}

// --- Array2 TypedExternal (all R native types) ---

impl TypedExternal for Array2<i32> {
    const TYPE_NAME: &'static str = "ndarray::Array2<i32>";
    const TYPE_NAME_CSTR: &'static [u8] = b"ndarray::Array2<i32>\0";
}

impl TypedExternal for Array2<f64> {
    const TYPE_NAME: &'static str = "ndarray::Array2<f64>";
    const TYPE_NAME_CSTR: &'static [u8] = b"ndarray::Array2<f64>\0";
}

impl TypedExternal for Array2<u8> {
    const TYPE_NAME: &'static str = "ndarray::Array2<u8>";
    const TYPE_NAME_CSTR: &'static [u8] = b"ndarray::Array2<u8>\0";
}

impl TypedExternal for Array2<RLogical> {
    const TYPE_NAME: &'static str = "ndarray::Array2<RLogical>";
    const TYPE_NAME_CSTR: &'static [u8] = b"ndarray::Array2<RLogical>\0";
}

impl TypedExternal for Array2<Rcomplex> {
    const TYPE_NAME: &'static str = "ndarray::Array2<Rcomplex>";
    const TYPE_NAME_CSTR: &'static [u8] = b"ndarray::Array2<Rcomplex>\0";
}

#[cfg(test)]
mod tests {
    #[test]
    fn array1_can_be_created() {
        let arr = super::Array1::from_vec(vec![1.0, 2.0, 3.0]);
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn array2_can_be_created() {
        let arr = super::Array2::from_shape_vec((2, 3), vec![1, 2, 3, 4, 5, 6]).unwrap();
        assert_eq!(arr.dim(), (2, 3));
    }
}
