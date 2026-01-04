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
use crate::gc_protect::{OwnedProtect, ProtectScope};
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

        // Create R matrix with RAII protection
        unsafe {
            let mat = crate::ffi::Rf_allocMatrix(T::SEXP_TYPE, nrow as i32, ncol as i32);
            let guard = OwnedProtect::new(mat);

            let ptr = crate::ffi::DATAPTR_RO(guard.get()) as *mut T;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());

            guard.into_inner()
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
        let arr = Array3::from_shape_vec((d0, d1, d2).f(), slice.to_vec()).map_err(|_| {
            SexpLengthError {
                expected: d0 * d1 * d2,
                actual: slice.len(),
            }
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
        let arr =
            ArrayD::from_shape_vec(shape.f(), slice.to_vec()).map_err(|_| SexpLengthError {
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
/// arr$r_get(2L)              # Get element at index 2 (0-indexed): 3.0
/// arr$r_slice_1d(1L, 4L)     # Slice [1:4): c(2, 3, 4)
/// arr$r_first()              # First element: 1.0
/// arr$r_last()               # Last element: 5.0
/// ```
pub trait RNdSlice {
    /// Element type of the array.
    type Elem: Clone;

    /// Get the element at the given flat index (0-indexed).
    ///
    /// Returns None if index is out of bounds.
    fn r_get(&self, index: i32) -> Option<Self::Elem>;

    /// Get the first element, or None if empty.
    fn r_first(&self) -> Option<Self::Elem>;

    /// Get the last element, or None if empty.
    fn r_last(&self) -> Option<Self::Elem>;

    /// Extract a 1D slice as a new Vec (0-indexed, exclusive end).
    ///
    /// Returns elements in the range [start, end).
    fn r_slice_1d(&self, start: i32, end: i32) -> Vec<Self::Elem>;

    /// Get elements at the given indices.
    fn r_get_many(&self, indices: Vec<i32>) -> Vec<Option<Self::Elem>> {
        indices.into_iter().map(|i| self.r_get(i)).collect()
    }

    /// Check if the given index is valid.
    fn r_is_valid_index(&self, index: i32) -> bool {
        self.r_get(index).is_some()
    }
}

impl RNdSlice for Array1<f64> {
    type Elem = f64;

    fn r_get(&self, index: i32) -> Option<f64> {
        if index < 0 {
            return None;
        }
        self.get(index as usize).copied()
    }

    fn r_first(&self) -> Option<f64> {
        self.first().copied()
    }

    fn r_last(&self) -> Option<f64> {
        self.last().copied()
    }

    fn r_slice_1d(&self, start: i32, end: i32) -> Vec<f64> {
        let start = start.max(0) as usize;
        let end = (end as usize).min(self.len());
        if start >= end {
            return Vec::new();
        }
        self.slice(ndarray::s![start..end]).to_vec()
    }
}

impl RNdSlice for Array1<i32> {
    type Elem = i32;

    fn r_get(&self, index: i32) -> Option<i32> {
        if index < 0 {
            return None;
        }
        self.get(index as usize).copied()
    }

    fn r_first(&self) -> Option<i32> {
        self.first().copied()
    }

    fn r_last(&self) -> Option<i32> {
        self.last().copied()
    }

    fn r_slice_1d(&self, start: i32, end: i32) -> Vec<i32> {
        let start = start.max(0) as usize;
        let end = (end as usize).min(self.len());
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
/// mat$r_row(0L)     # First row: c(1, 3, 5)
/// mat$r_col(1L)     # Second column: c(3, 4)
/// mat$r_get_2d(0L, 1L)  # Element at [0,1]: 3
/// ```
pub trait RNdSlice2D {
    /// Element type of the array.
    type Elem: Clone;

    /// Get the element at [row, col] (0-indexed).
    fn r_get_2d(&self, row: i32, col: i32) -> Option<Self::Elem>;

    /// Get a row as a vector.
    fn r_row(&self, row: i32) -> Vec<Self::Elem>;

    /// Get a column as a vector.
    fn r_col(&self, col: i32) -> Vec<Self::Elem>;

    /// Get the diagonal elements.
    fn r_diag(&self) -> Vec<Self::Elem>;

    /// Get the number of rows.
    fn r_nrows(&self) -> i32;

    /// Get the number of columns.
    fn r_ncols(&self) -> i32;
}

impl RNdSlice2D for Array2<f64> {
    type Elem = f64;

    fn r_get_2d(&self, row: i32, col: i32) -> Option<f64> {
        if row < 0 || col < 0 {
            return None;
        }
        self.get((row as usize, col as usize)).copied()
    }

    fn r_row(&self, row: i32) -> Vec<f64> {
        if row < 0 || row as usize >= self.nrows() {
            return Vec::new();
        }
        self.row(row as usize).to_vec()
    }

    fn r_col(&self, col: i32) -> Vec<f64> {
        if col < 0 || col as usize >= self.ncols() {
            return Vec::new();
        }
        self.column(col as usize).to_vec()
    }

    fn r_diag(&self) -> Vec<f64> {
        self.diag().to_vec()
    }

    fn r_nrows(&self) -> i32 {
        self.nrows() as i32
    }

    fn r_ncols(&self) -> i32 {
        self.ncols() as i32
    }
}

impl RNdSlice2D for Array2<i32> {
    type Elem = i32;

    fn r_get_2d(&self, row: i32, col: i32) -> Option<i32> {
        if row < 0 || col < 0 {
            return None;
        }
        self.get((row as usize, col as usize)).copied()
    }

    fn r_row(&self, row: i32) -> Vec<i32> {
        if row < 0 || row as usize >= self.nrows() {
            return Vec::new();
        }
        self.row(row as usize).to_vec()
    }

    fn r_col(&self, col: i32) -> Vec<i32> {
        if col < 0 || col as usize >= self.ncols() {
            return Vec::new();
        }
        self.column(col as usize).to_vec()
    }

    fn r_diag(&self) -> Vec<i32> {
        self.diag().to_vec()
    }

    fn r_nrows(&self) -> i32 {
        self.nrows() as i32
    }

    fn r_ncols(&self) -> i32 {
        self.ncols() as i32
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
        assert_eq!(RNdSlice::r_get(&arr, 0), Some(1.0));
        assert_eq!(RNdSlice::r_get(&arr, 2), Some(3.0));
        assert_eq!(RNdSlice::r_get(&arr, 4), Some(5.0));
        assert_eq!(RNdSlice::r_get(&arr, 5), None);
        assert_eq!(RNdSlice::r_get(&arr, -1), None);
    }

    #[test]
    fn rndslice_first_last() {
        let arr = Array1::from_vec(vec![10.0, 20.0, 30.0]);
        assert_eq!(RNdSlice::r_first(&arr), Some(10.0));
        assert_eq!(RNdSlice::r_last(&arr), Some(30.0));

        let empty: Array1<f64> = Array1::from_vec(vec![]);
        assert_eq!(RNdSlice::r_first(&empty), None);
        assert_eq!(RNdSlice::r_last(&empty), None);
    }

    #[test]
    fn rndslice_slice_1d() {
        let arr = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(RNdSlice::r_slice_1d(&arr, 1, 4), vec![2.0, 3.0, 4.0]);
        assert_eq!(RNdSlice::r_slice_1d(&arr, 0, 2), vec![1.0, 2.0]);
        assert_eq!(RNdSlice::r_slice_1d(&arr, 3, 10), vec![4.0, 5.0]); // Clamped
        assert_eq!(RNdSlice::r_slice_1d(&arr, -5, 2), vec![1.0, 2.0]); // Negative start clamped
        assert_eq!(RNdSlice::r_slice_1d(&arr, 3, 2), Vec::<f64>::new()); // Empty range
    }

    #[test]
    fn rndslice_get_many() {
        let arr = Array1::from_vec(vec![10, 20, 30, 40, 50]);
        let results = RNdSlice::r_get_many(&arr, vec![0, 2, 4, 10]);
        assert_eq!(results, vec![Some(10), Some(30), Some(50), None]);
    }

    #[test]
    fn rndslice_is_valid_index() {
        let arr = Array1::from_vec(vec![1.0, 2.0, 3.0]);
        assert!(RNdSlice::r_is_valid_index(&arr, 0));
        assert!(RNdSlice::r_is_valid_index(&arr, 2));
        assert!(!RNdSlice::r_is_valid_index(&arr, 3));
        assert!(!RNdSlice::r_is_valid_index(&arr, -1));
    }

    // RNdSlice2D tests
    #[test]
    fn rndslice2d_get_2d() {
        let arr = Array2::from_shape_vec((2, 3), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
        assert_eq!(RNdSlice2D::r_get_2d(&arr, 0, 0), Some(1.0));
        assert_eq!(RNdSlice2D::r_get_2d(&arr, 0, 2), Some(3.0));
        assert_eq!(RNdSlice2D::r_get_2d(&arr, 1, 1), Some(5.0));
        assert_eq!(RNdSlice2D::r_get_2d(&arr, 2, 0), None);
        assert_eq!(RNdSlice2D::r_get_2d(&arr, -1, 0), None);
    }

    #[test]
    fn rndslice2d_row_col() {
        let arr = Array2::from_shape_vec((2, 3), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
        assert_eq!(RNdSlice2D::r_row(&arr, 0), vec![1.0, 2.0, 3.0]);
        assert_eq!(RNdSlice2D::r_row(&arr, 1), vec![4.0, 5.0, 6.0]);
        assert_eq!(RNdSlice2D::r_row(&arr, 2), Vec::<f64>::new()); // Out of bounds
        assert_eq!(RNdSlice2D::r_col(&arr, 0), vec![1.0, 4.0]);
        assert_eq!(RNdSlice2D::r_col(&arr, 1), vec![2.0, 5.0]);
        assert_eq!(RNdSlice2D::r_col(&arr, 3), Vec::<f64>::new()); // Out of bounds
    }

    #[test]
    fn rndslice2d_diag() {
        let arr = Array2::from_shape_vec((3, 3), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0])
            .unwrap();
        assert_eq!(RNdSlice2D::r_diag(&arr), vec![1.0, 5.0, 9.0]);
    }

    #[test]
    fn rndslice2d_nrows_ncols() {
        let arr = Array2::from_shape_vec((2, 4), vec![1.0; 8]).unwrap();
        assert_eq!(RNdSlice2D::r_nrows(&arr), 2);
        assert_eq!(RNdSlice2D::r_ncols(&arr), 4);
    }
}
