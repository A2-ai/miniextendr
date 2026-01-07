//! Integration with the `nalgebra` crate.
//!
//! This module provides conversions between R vectors/matrices and nalgebra types:
//!
//! | R Type | nalgebra Type | Notes |
//! |--------|--------------|-------|
//! | REALSXP | `DVector<f64>`, `DMatrix<f64>` | Dynamic vectors/matrices |
//! | INTSXP | `DVector<i32>`, `DMatrix<i32>` | Integer vectors/matrices |
//!
//! # Features
//!
//! Enable this module with the `nalgebra` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["nalgebra"] }
//! ```
//!
//! # Dimension Handling
//!
//! When converting R objects to `DMatrix<T>`, the dimension (`dim`) attribute
//! determines the matrix shape:
//!
//! | R Input | `dim` Attribute | Result |
//! |---------|-----------------|--------|
//! | `matrix(1:6, 2, 3)` | `c(2L, 3L)` | `DMatrix` with 2 rows, 3 cols |
//! | `c(1, 2, 3)` | None | **Column vector**: `DMatrix` with 3 rows, 1 col |
//! | `array(1:24, c(2,3,4))` | `c(2L, 3L, 4L)` | **Error**: only 2D supported |
//!
//! **Important**: Plain R vectors (without `dim`) are treated as **column vectors**
//! with shape `(length, 1)`. This matches R's convention where vectors are
//! implicitly single-column matrices in matrix operations.
//!
//! ```r
//! # In R:
//! v <- c(1, 2, 3)         # No dim attribute
//! m <- matrix(v, 3, 1)    # Explicit column vector
//! # Both convert to DMatrix with shape (3, 1)
//! ```
//!
//! # Memory Layout
//!
//! Both R and nalgebra use **column-major** storage by default, which makes
//! conversion efficient. Data is copied but doesn't need reordering.
//!
//! # Example
//!
//! ```ignore
//! use nalgebra::{DVector, DMatrix};
//!
//! #[miniextendr]
//! fn norm(v: DVector<f64>) -> f64 {
//!     v.norm()
//! }
//!
//! #[miniextendr]
//! fn determinant(m: DMatrix<f64>) -> f64 {
//!     m.determinant()
//! }
//! ```

pub use nalgebra::{DMatrix, DVector};

use crate::ffi::{RLogical, RNativeType, SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, SexpLengthError, SexpTypeError, TryFromSexp};
use crate::gc_protect::OwnedProtect;
use crate::into_r::IntoR;
use nalgebra::Scalar;

// =============================================================================
// R-native DVector/DMatrix conversions (explicit)
// =============================================================================

fn dvector_from_sexp<T: RNativeType + Scalar + Copy>(sexp: SEXP) -> Result<DVector<T>, SexpError> {
    let actual = sexp.type_of();
    if actual != T::SEXP_TYPE {
        return Err(SexpTypeError {
            expected: T::SEXP_TYPE,
            actual,
        }
        .into());
    }

    let slice: &[T] = unsafe { sexp.as_slice() };
    Ok(DVector::from_column_slice(slice))
}

fn dmatrix_from_sexp<T: RNativeType + Scalar + Copy>(sexp: SEXP) -> Result<DMatrix<T>, SexpError> {
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

    if slice.len() != nrow * ncol {
        return Err(SexpLengthError {
            expected: nrow * ncol,
            actual: slice.len(),
        }
        .into());
    }

    Ok(DMatrix::from_column_slice(nrow, ncol, slice))
}

macro_rules! impl_nalgebra_try_from_sexp_native {
    ($t:ty) => {
        impl TryFromSexp for DVector<$t> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                dvector_from_sexp::<$t>(sexp)
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                dvector_from_sexp::<$t>(sexp)
            }
        }

        impl TryFromSexp for DMatrix<$t> {
            type Error = SexpError;

            fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
                dmatrix_from_sexp::<$t>(sexp)
            }

            unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
                dmatrix_from_sexp::<$t>(sexp)
            }
        }
    };
}

impl_nalgebra_try_from_sexp_native!(i32);
impl_nalgebra_try_from_sexp_native!(f64);
impl_nalgebra_try_from_sexp_native!(u8);
impl_nalgebra_try_from_sexp_native!(RLogical);
impl_nalgebra_try_from_sexp_native!(crate::ffi::Rcomplex);

// =============================================================================
// DVector conversions
// =============================================================================

/// Convert `DVector<T>` to R vector.
impl<T: RNativeType + Scalar> IntoR for DVector<T> {
    fn into_sexp(self) -> SEXP {
        let data: Vec<T> = self.data.into();
        data.into_sexp()
    }

    unsafe fn into_sexp_unchecked(self) -> SEXP {
        let data: Vec<T> = self.data.into();
        unsafe { data.into_sexp_unchecked() }
    }
}

// =============================================================================
// DMatrix conversions
// =============================================================================

/// Convert `DMatrix<T>` to R matrix.
///
/// nalgebra stores data in column-major order (same as R), so this is efficient.
impl<T: RNativeType + Scalar> IntoR for DMatrix<T> {
    fn into_sexp(self) -> SEXP {
        let nrow = self.nrows();
        let ncol = self.ncols();

        // nalgebra data is already column-major
        let data: Vec<T> = self.data.into();

        // Create R matrix with RAII protection
        unsafe {
            let mat = crate::ffi::Rf_allocMatrix(T::SEXP_TYPE, nrow as i32, ncol as i32);
            let guard = OwnedProtect::new(mat);

            let ptr = crate::ffi::DATAPTR_RO(guard.get()) as *mut T;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());

            // Return the SEXP - guard drops and unprotects
            guard.get()
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

        // Validate: reject negative or NA dimensions (NA_INTEGER is i32::MIN)
        let nrow = dim_slice[0];
        let ncol = dim_slice[1];
        if nrow < 0 || ncol < 0 {
            return Err(SexpError::InvalidValue(
                "matrix dimensions must be non-negative".into(),
            ));
        }

        Ok((nrow as usize, ncol as usize))
    }
}

// =============================================================================
// TypedExternal implementations for ExternalPtr support
// =============================================================================
//
// These allow storing nalgebra types as R external pointers (zero-copy).
// Use `ExternalPtr<DVector<f64>>` when you want to pass vectors without copying.
//
// Note: We implement TypedExternal but NOT IntoExternalPtr, because:
// - TypedExternal: Enables explicit `ExternalPtr<DVector<f64>>` usage
// - IntoExternalPtr: Would conflict with our IntoR impl (which copies to R native)
//
// Default behavior: `DVector<f64>` → copies to R vector (via IntoR)
// Explicit ExternalPtr: `ExternalPtr::new(vec)` → wraps without copying
//
// nalgebra requires `T: Scalar` which is `'static + Clone + PartialEq + Debug`.
// We implement for types where both RNativeType and Scalar are satisfied.

use crate::externalptr::TypedExternal;

// Helper macro for implementing TypedExternal for nalgebra types.
macro_rules! impl_te_nalgebra {
    ($ty:ty, $name:expr) => {
        impl TypedExternal for $ty {
            const TYPE_NAME: &'static str = $name;
            const TYPE_NAME_CSTR: &'static [u8] = concat!($name, "\0").as_bytes();
            const TYPE_ID_CSTR: &'static [u8] = concat!($name, "\0").as_bytes();
        }
    };
}

// --- DVector TypedExternal ---
impl_te_nalgebra!(DVector<i32>, "nalgebra::DVector<i32>");
impl_te_nalgebra!(DVector<f64>, "nalgebra::DVector<f64>");
impl_te_nalgebra!(DVector<u8>, "nalgebra::DVector<u8>");

// --- DMatrix TypedExternal ---
impl_te_nalgebra!(DMatrix<i32>, "nalgebra::DMatrix<i32>");
impl_te_nalgebra!(DMatrix<f64>, "nalgebra::DMatrix<f64>");
impl_te_nalgebra!(DMatrix<u8>, "nalgebra::DMatrix<u8>");

// =============================================================================
// RVectorOps adapter trait
// =============================================================================

/// Adapter trait for [`nalgebra::DVector`] operations.
///
/// Provides common vector operations accessible from R.
/// Automatically implemented for `DVector<f64>`.
///
/// # Example
///
/// ```rust,ignore
/// use nalgebra::DVector;
/// use miniextendr_api::nalgebra_impl::RVectorOps;
///
/// #[derive(ExternalPtr)]
/// struct MyVector(DVector<f64>);
///
/// #[miniextendr]
/// impl RVectorOps for MyVector {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RVectorOps for MyVector;
/// }
/// ```
///
/// In R:
/// ```r
/// v <- MyVector$new(c(3, 4))
/// v$norm()      # 5.0 (Euclidean norm)
/// v$len()       # 2L
/// ```
pub trait RVectorOps {
    /// Get the number of elements in the vector.
    fn len(&self) -> i32;

    /// Check if the vector is empty.
    fn is_empty(&self) -> bool;

    /// Compute the Euclidean (L2) norm.
    fn norm(&self) -> f64;

    /// Compute the squared Euclidean norm (avoids sqrt).
    fn norm_squared(&self) -> f64;

    /// Compute the L1 norm (sum of absolute values).
    fn norm_l1(&self) -> f64;

    /// Compute the L-infinity norm (maximum absolute value).
    fn norm_linf(&self) -> f64;

    /// Compute the sum of all elements.
    fn sum(&self) -> f64;

    /// Compute the mean of all elements.
    fn mean(&self) -> f64;

    /// Get the minimum element.
    fn min(&self) -> f64;

    /// Get the maximum element.
    fn max(&self) -> f64;

    /// Get the index of the minimum element (0-based).
    fn argmin(&self) -> i32;

    /// Get the index of the maximum element (0-based).
    fn argmax(&self) -> i32;

    /// Compute the dot product with another vector.
    fn dot(&self, other: &Self) -> f64;

    /// Return a normalized (unit) vector.
    fn normalize(&self) -> DVector<f64>;

    /// Scale the vector by a scalar.
    fn scale(&self, s: f64) -> DVector<f64>;

    /// Add another vector.
    fn add(&self, other: &Self) -> DVector<f64>;

    /// Subtract another vector.
    fn sub(&self, other: &Self) -> DVector<f64>;

    /// Element-wise multiplication.
    fn component_mul(&self, other: &Self) -> DVector<f64>;
}

impl RVectorOps for DVector<f64> {
    fn len(&self) -> i32 {
        DVector::len(self) as i32
    }

    fn is_empty(&self) -> bool {
        DVector::is_empty(self)
    }

    fn norm(&self) -> f64 {
        nalgebra::base::Normed::norm(self)
    }

    fn norm_squared(&self) -> f64 {
        nalgebra::base::Normed::norm_squared(self)
    }

    fn norm_l1(&self) -> f64 {
        self.iter().map(|x| x.abs()).sum()
    }

    fn norm_linf(&self) -> f64 {
        self.iter().fold(0.0f64, |a, &b| a.max(b.abs()))
    }

    fn sum(&self) -> f64 {
        self.iter().sum()
    }

    fn mean(&self) -> f64 {
        if DVector::is_empty(self) {
            f64::NAN
        } else {
            self.iter().sum::<f64>() / DVector::len(self) as f64
        }
    }

    fn min(&self) -> f64 {
        self.iter().copied().fold(f64::INFINITY, f64::min)
    }

    fn max(&self) -> f64 {
        self.iter().copied().fold(f64::NEG_INFINITY, f64::max)
    }

    fn argmin(&self) -> i32 {
        self.iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i as i32)
            .unwrap_or(-1)
    }

    fn argmax(&self) -> i32 {
        self.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i as i32)
            .unwrap_or(-1)
    }

    fn dot(&self, other: &Self) -> f64 {
        DVector::dot(self, other)
    }

    fn normalize(&self) -> DVector<f64> {
        self.clone().normalize()
    }

    fn scale(&self, s: f64) -> DVector<f64> {
        self * s
    }

    fn add(&self, other: &Self) -> DVector<f64> {
        self + other
    }

    fn sub(&self, other: &Self) -> DVector<f64> {
        self - other
    }

    fn component_mul(&self, other: &Self) -> DVector<f64> {
        DVector::component_mul(self, other)
    }
}

// =============================================================================
// RMatrixOps adapter trait
// =============================================================================

/// Adapter trait for [`nalgebra::DMatrix`] operations.
///
/// Provides common matrix operations accessible from R.
/// Automatically implemented for `DMatrix<f64>`.
///
/// # Example
///
/// ```rust,ignore
/// use nalgebra::DMatrix;
/// use miniextendr_api::nalgebra_impl::RMatrixOps;
///
/// #[derive(ExternalPtr)]
/// struct MyMatrix(DMatrix<f64>);
///
/// #[miniextendr]
/// impl RMatrixOps for MyMatrix {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RMatrixOps for MyMatrix;
/// }
/// ```
///
/// In R:
/// ```r
/// m <- MyMatrix$new(matrix(1:4, 2, 2))
/// m$nrows()        # 2L
/// m$ncols()        # 2L
/// m$determinant()  # -2.0
/// m$transpose()    # Returns transposed matrix
/// ```
pub trait RMatrixOps {
    /// Get the number of rows.
    fn nrows(&self) -> i32;

    /// Get the number of columns.
    fn ncols(&self) -> i32;

    /// Get the shape as (rows, cols).
    fn shape(&self) -> (i32, i32);

    /// Check if the matrix is square.
    fn is_square(&self) -> bool;

    /// Check if the matrix is empty.
    fn is_empty(&self) -> bool;

    /// Compute the transpose.
    fn transpose(&self) -> DMatrix<f64>;

    /// Compute the determinant (for square matrices).
    fn determinant(&self) -> f64;

    /// Compute the trace (sum of diagonal elements).
    fn trace(&self) -> f64;

    /// Get the diagonal as a vector.
    fn diagonal(&self) -> DVector<f64>;

    /// Compute the Frobenius norm.
    fn norm(&self) -> f64;

    /// Compute the matrix inverse (returns None if singular).
    fn try_inverse(&self) -> Option<DMatrix<f64>>;

    /// Compute the sum of all elements.
    fn sum(&self) -> f64;

    /// Compute the mean of all elements.
    fn mean(&self) -> f64;

    /// Get the minimum element.
    fn min(&self) -> f64;

    /// Get the maximum element.
    fn max(&self) -> f64;

    /// Scale the matrix by a scalar.
    fn scale(&self, s: f64) -> DMatrix<f64>;

    /// Add another matrix.
    fn add(&self, other: &Self) -> DMatrix<f64>;

    /// Subtract another matrix.
    fn sub(&self, other: &Self) -> DMatrix<f64>;

    /// Matrix multiplication.
    fn mul(&self, other: &Self) -> DMatrix<f64>;

    /// Element-wise multiplication.
    fn component_mul(&self, other: &Self) -> DMatrix<f64>;

    /// Sum of each row, returned as a column vector.
    fn row_sum(&self) -> DVector<f64>;

    /// Sum of each column, returned as a row vector.
    fn column_sum(&self) -> DVector<f64>;

    /// Mean of each row.
    fn row_mean(&self) -> DVector<f64>;

    /// Mean of each column.
    fn column_mean(&self) -> DVector<f64>;
}

impl RMatrixOps for DMatrix<f64> {
    fn nrows(&self) -> i32 {
        DMatrix::nrows(self) as i32
    }

    fn ncols(&self) -> i32 {
        DMatrix::ncols(self) as i32
    }

    fn shape(&self) -> (i32, i32) {
        (DMatrix::nrows(self) as i32, DMatrix::ncols(self) as i32)
    }

    fn is_square(&self) -> bool {
        DMatrix::is_square(self)
    }

    fn is_empty(&self) -> bool {
        DMatrix::is_empty(self)
    }

    fn transpose(&self) -> DMatrix<f64> {
        DMatrix::transpose(self)
    }

    fn determinant(&self) -> f64 {
        DMatrix::determinant(self)
    }

    fn trace(&self) -> f64 {
        DMatrix::trace(self)
    }

    fn diagonal(&self) -> DVector<f64> {
        DMatrix::diagonal(self).into_owned()
    }

    fn norm(&self) -> f64 {
        nalgebra::base::Normed::norm(self)
    }

    fn try_inverse(&self) -> Option<DMatrix<f64>> {
        self.clone().try_inverse()
    }

    fn sum(&self) -> f64 {
        self.iter().sum()
    }

    fn mean(&self) -> f64 {
        let n = DMatrix::nrows(self) * DMatrix::ncols(self);
        if n == 0 {
            f64::NAN
        } else {
            self.iter().sum::<f64>() / n as f64
        }
    }

    fn min(&self) -> f64 {
        self.iter().copied().fold(f64::INFINITY, f64::min)
    }

    fn max(&self) -> f64 {
        self.iter().copied().fold(f64::NEG_INFINITY, f64::max)
    }

    fn scale(&self, s: f64) -> DMatrix<f64> {
        self * s
    }

    fn add(&self, other: &Self) -> DMatrix<f64> {
        self + other
    }

    fn sub(&self, other: &Self) -> DMatrix<f64> {
        self - other
    }

    fn mul(&self, other: &Self) -> DMatrix<f64> {
        self * other
    }

    fn component_mul(&self, other: &Self) -> DMatrix<f64> {
        DMatrix::component_mul(self, other)
    }

    fn row_sum(&self) -> DVector<f64> {
        let nrows = DMatrix::nrows(self);
        let ncols = DMatrix::ncols(self);
        let mut result = DVector::zeros(nrows);
        for i in 0..nrows {
            let mut sum = 0.0;
            for j in 0..ncols {
                sum += self[(i, j)];
            }
            result[i] = sum;
        }
        result
    }

    fn column_sum(&self) -> DVector<f64> {
        let nrows = DMatrix::nrows(self);
        let ncols = DMatrix::ncols(self);
        let mut result = DVector::zeros(ncols);
        for j in 0..ncols {
            let mut sum = 0.0;
            for i in 0..nrows {
                sum += self[(i, j)];
            }
            result[j] = sum;
        }
        result
    }

    fn row_mean(&self) -> DVector<f64> {
        let ncols = DMatrix::ncols(self) as f64;
        if ncols == 0.0 {
            DVector::from_element(DMatrix::nrows(self), f64::NAN)
        } else {
            let sums = self.row_sum();
            DVector::from_iterator(sums.len(), sums.iter().map(|x| x / ncols))
        }
    }

    fn column_mean(&self) -> DVector<f64> {
        let nrows = DMatrix::nrows(self) as f64;
        if nrows == 0.0 {
            DVector::from_element(DMatrix::ncols(self), f64::NAN)
        } else {
            let sums = self.column_sum();
            DVector::from_iterator(sums.len(), sums.iter().map(|x| x / nrows))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dvector_can_be_created() {
        let v = DVector::from_column_slice(&[1.0, 2.0, 3.0]);
        assert_eq!(v.len(), 3);
    }

    #[test]
    fn dmatrix_can_be_created() {
        let m = DMatrix::from_column_slice(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(m.nrows(), 2);
        assert_eq!(m.ncols(), 3);
    }

    // RVectorOps tests
    #[test]
    fn rvectorops_len_and_empty() {
        let v = DVector::from_column_slice(&[1.0, 2.0, 3.0]);
        assert_eq!(RVectorOps::len(&v), 3);
        assert!(!RVectorOps::is_empty(&v));

        let empty: DVector<f64> = DVector::zeros(0);
        assert_eq!(RVectorOps::len(&empty), 0);
        assert!(RVectorOps::is_empty(&empty));
    }

    #[test]
    fn rvectorops_norms() {
        let v = DVector::from_column_slice(&[3.0, 4.0]);
        assert!((RVectorOps::norm(&v) - 5.0).abs() < 1e-10);
        assert!((RVectorOps::norm_squared(&v) - 25.0).abs() < 1e-10);
        assert!((RVectorOps::norm_l1(&v) - 7.0).abs() < 1e-10);
        assert!((RVectorOps::norm_linf(&v) - 4.0).abs() < 1e-10);
    }

    #[test]
    fn rvectorops_stats() {
        let v = DVector::from_column_slice(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!((RVectorOps::sum(&v) - 15.0).abs() < 1e-10);
        assert!((RVectorOps::mean(&v) - 3.0).abs() < 1e-10);
        assert!((RVectorOps::min(&v) - 1.0).abs() < 1e-10);
        assert!((RVectorOps::max(&v) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn rvectorops_argmin_argmax() {
        let v = DVector::from_column_slice(&[3.0, 1.0, 4.0, 1.0, 5.0]);
        assert_eq!(RVectorOps::argmin(&v), 1); // First occurrence of min
        assert_eq!(RVectorOps::argmax(&v), 4);
    }

    #[test]
    fn rvectorops_dot() {
        let a = DVector::from_column_slice(&[1.0, 2.0, 3.0]);
        let b = DVector::from_column_slice(&[4.0, 5.0, 6.0]);
        assert!((RVectorOps::dot(&a, &b) - 32.0).abs() < 1e-10);
    }

    #[test]
    fn rvectorops_normalize() {
        let v = DVector::from_column_slice(&[3.0, 4.0]);
        let n = RVectorOps::normalize(&v);
        assert!((RVectorOps::norm(&n) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rvectorops_arithmetic() {
        let a = DVector::from_column_slice(&[1.0, 2.0]);
        let b = DVector::from_column_slice(&[3.0, 4.0]);

        let sum = RVectorOps::add(&a, &b);
        assert!((sum[0] - 4.0).abs() < 1e-10);
        assert!((sum[1] - 6.0).abs() < 1e-10);

        let diff = RVectorOps::sub(&a, &b);
        assert!((diff[0] - -2.0).abs() < 1e-10);
        assert!((diff[1] - -2.0).abs() < 1e-10);

        let scaled = RVectorOps::scale(&a, 2.0);
        assert!((scaled[0] - 2.0).abs() < 1e-10);
        assert!((scaled[1] - 4.0).abs() < 1e-10);
    }

    // RMatrixOps tests
    #[test]
    fn rmatrixops_shape() {
        let m = DMatrix::from_column_slice(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(RMatrixOps::nrows(&m), 2);
        assert_eq!(RMatrixOps::ncols(&m), 3);
        assert_eq!(RMatrixOps::shape(&m), (2, 3));
        assert!(!RMatrixOps::is_square(&m));
    }

    #[test]
    fn rmatrixops_square() {
        let m = DMatrix::from_column_slice(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        assert!(RMatrixOps::is_square(&m));
        // Column-major: [[1, 3], [2, 4]]
        // Determinant: 1*4 - 3*2 = -2
        assert!((RMatrixOps::determinant(&m) - -2.0).abs() < 1e-10);
        assert!((RMatrixOps::trace(&m) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn rmatrixops_transpose() {
        let m = DMatrix::from_column_slice(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let t = RMatrixOps::transpose(&m);
        assert_eq!(t.nrows(), 3);
        assert_eq!(t.ncols(), 2);
    }

    #[test]
    fn rmatrixops_inverse() {
        let m = DMatrix::from_column_slice(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        let inv = RMatrixOps::try_inverse(&m).unwrap();
        let product = &m * &inv;
        // Should be close to identity
        assert!((product[(0, 0)] - 1.0).abs() < 1e-10);
        assert!((product[(1, 1)] - 1.0).abs() < 1e-10);
        assert!(product[(0, 1)].abs() < 1e-10);
        assert!(product[(1, 0)].abs() < 1e-10);
    }

    #[test]
    fn rmatrixops_stats() {
        let m = DMatrix::from_column_slice(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        assert!((RMatrixOps::sum(&m) - 10.0).abs() < 1e-10);
        assert!((RMatrixOps::mean(&m) - 2.5).abs() < 1e-10);
        assert!((RMatrixOps::min(&m) - 1.0).abs() < 1e-10);
        assert!((RMatrixOps::max(&m) - 4.0).abs() < 1e-10);
    }

    #[test]
    fn rmatrixops_row_column_sums() {
        // Column-major: [[1, 3], [2, 4]]
        let m = DMatrix::from_column_slice(2, 2, &[1.0, 2.0, 3.0, 4.0]);

        let rs = RMatrixOps::row_sum(&m);
        assert!((rs[0] - 4.0).abs() < 1e-10); // 1 + 3
        assert!((rs[1] - 6.0).abs() < 1e-10); // 2 + 4

        let cs = RMatrixOps::column_sum(&m);
        assert!((cs[0] - 3.0).abs() < 1e-10); // 1 + 2
        assert!((cs[1] - 7.0).abs() < 1e-10); // 3 + 4
    }

    #[test]
    fn rmatrixops_mul() {
        let a = DMatrix::from_column_slice(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        let b = DMatrix::from_column_slice(2, 2, &[5.0, 6.0, 7.0, 8.0]);
        let c = RMatrixOps::mul(&a, &b);
        // Column-major: a = [[1, 3], [2, 4]], b = [[5, 7], [6, 8]]
        // c = a * b = [[1*5+3*6, 1*7+3*8], [2*5+4*6, 2*7+4*8]]
        //           = [[23, 31], [34, 46]]
        assert!((c[(0, 0)] - 23.0).abs() < 1e-10);
        assert!((c[(0, 1)] - 31.0).abs() < 1e-10);
        assert!((c[(1, 0)] - 34.0).abs() < 1e-10);
        assert!((c[(1, 1)] - 46.0).abs() < 1e-10);
    }
}
