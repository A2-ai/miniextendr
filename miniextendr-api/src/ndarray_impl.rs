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
    Array1, Array2, Array3, ArrayD, ArrayView1, ArrayView2, ArrayView3, ArrayViewD, IxDyn,
    ShapeBuilder,
};

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
// Array3 conversions (3D arrays)
// =============================================================================

/// Convert R 3D array to `Array3<T>`.
///
/// R stores arrays in column-major order. This function reads the data
/// and creates an ndarray with Fortran layout matching R.
impl<T: RNativeType + Clone> TryFromSexp for Array3<T> {
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

        let (d0, d1, d2) = get_array3_dims(sexp)?;
        let slice: &[T] = unsafe { sexp.as_slice() };

        // R stores in column-major order, so we create in Fortran order
        let arr =
            Array3::from_shape_vec((d0, d1, d2).f(), slice.to_vec()).map_err(|_| SexpLengthError {
                expected: d0 * d1 * d2,
                actual: slice.len(),
            })?;

        Ok(arr)
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

/// Convert `Array3<T>` to R 3D array.
///
/// Creates a column-major R array from the ndarray.
impl<T: RNativeType + Clone> IntoR for Array3<T> {
    fn into_sexp(self) -> SEXP {
        let (d0, d1, d2) = self.dim();

        // For Fortran-order arrays, we can copy directly if contiguous.
        // Otherwise, iterate in Fortran order.
        let data: Vec<T> = if self.is_standard_layout() {
            // Row-major: need to reorder to column-major
            let mut v = Vec::with_capacity(d0 * d1 * d2);
            for k in 0..d2 {
                for j in 0..d1 {
                    for i in 0..d0 {
                        v.push(self[[i, j, k]].clone());
                    }
                }
            }
            v
        } else {
            // Already Fortran order or some other layout - iterate in Fortran order
            let mut v = Vec::with_capacity(d0 * d1 * d2);
            for k in 0..d2 {
                for j in 0..d1 {
                    for i in 0..d0 {
                        v.push(self[[i, j, k]].clone());
                    }
                }
            }
            v
        };

        // Create R array with dim attribute
        unsafe {
            let arr = crate::ffi::Rf_allocVector(T::SEXP_TYPE, data.len() as crate::ffi::R_xlen_t);
            crate::ffi::Rf_protect(arr);

            let ptr = crate::ffi::DATAPTR_RO(arr) as *mut T;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());

            // Set dim attribute
            let dim = crate::ffi::Rf_allocVector(SEXPTYPE::INTSXP, 3);
            crate::ffi::Rf_protect(dim);
            let dim_ptr = crate::ffi::INTEGER(dim);
            *dim_ptr = d0 as i32;
            *dim_ptr.add(1) = d1 as i32;
            *dim_ptr.add(2) = d2 as i32;
            crate::ffi::Rf_setAttrib(arr, crate::ffi::R_DimSymbol, dim);

            crate::ffi::Rf_unprotect(2);
            arr
        }
    }
}

// =============================================================================
// ArrayD conversions (N-dimensional arrays)
// =============================================================================

/// Convert R N-dimensional array to `ArrayD<T>`.
///
/// This is the most flexible conversion, accepting arrays of any dimension.
/// Plain vectors (no dim attribute) are treated as 1D arrays.
impl<T: RNativeType + Clone> TryFromSexp for ArrayD<T> {
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

        let dims = get_array_dims(sexp).unwrap_or_else(|| vec![sexp.len()]);
        let slice: &[T] = unsafe { sexp.as_slice() };

        // R stores in column-major order, so we create in Fortran order
        let shape = IxDyn(&dims);
        let arr = ArrayD::from_shape_vec(shape.f(), slice.to_vec()).map_err(|_| SexpLengthError {
            expected: dims.iter().product(),
            actual: slice.len(),
        })?;

        Ok(arr)
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

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
                v.push(self[IxDyn(&idx)].clone());
            });
            v
        };

        unsafe {
            let arr = crate::ffi::Rf_allocVector(T::SEXP_TYPE, total_len as crate::ffi::R_xlen_t);
            crate::ffi::Rf_protect(arr);

            let ptr = crate::ffi::DATAPTR_RO(arr) as *mut T;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());

            // Set dim attribute if ndim > 1
            if ndim > 1 {
                let dim =
                    crate::ffi::Rf_allocVector(SEXPTYPE::INTSXP, ndim as crate::ffi::R_xlen_t);
                crate::ffi::Rf_protect(dim);
                let dim_ptr = crate::ffi::INTEGER(dim);
                for (i, &d) in shape.iter().enumerate() {
                    *dim_ptr.add(i) = d as i32;
                }
                crate::ffi::Rf_setAttrib(arr, crate::ffi::R_DimSymbol, dim);
                crate::ffi::Rf_unprotect(2);
            } else {
                crate::ffi::Rf_unprotect(1);
            }

            arr
        }
    }
}

/// Iterate over indices in Fortran (column-major) order.
fn fortran_order_iter<F: FnMut(Vec<usize>)>(shape: &[usize], mut f: F) {
    if shape.is_empty() {
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
// Helper functions
// =============================================================================

/// Get array dimensions from R object as a Vec.
///
/// Returns `None` if no dim attribute (plain vector).
fn get_array_dims(sexp: SEXP) -> Option<Vec<usize>> {
    unsafe {
        let dim = crate::ffi::Rf_getAttrib(sexp, crate::ffi::R_DimSymbol);
        if dim.type_of() != SEXPTYPE::INTSXP {
            return None;
        }

        let dim_slice: &[i32] = dim.as_slice();
        Some(dim_slice.iter().map(|&d| d as usize).collect())
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
/// - The SEXP must be a matrix (have a `dim` attribute of length 2).
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
    let view =
        ArrayView3::from_shape((d0, d1, d2).f(), slice).map_err(|_| SexpLengthError {
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

// --- ArrayD TypedExternal (all R native types) ---
impl_te_ndarray!(ArrayD<i32>, "ndarray::ArrayD<i32>");
impl_te_ndarray!(ArrayD<f64>, "ndarray::ArrayD<f64>");
impl_te_ndarray!(ArrayD<u8>, "ndarray::ArrayD<u8>");
impl_te_ndarray!(ArrayD<RLogical>, "ndarray::ArrayD<RLogical>");
impl_te_ndarray!(ArrayD<Rcomplex>, "ndarray::ArrayD<Rcomplex>");

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
}
