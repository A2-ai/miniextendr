//! Integration with the `nalgebra` crate.
//!
//! This module provides conversions between R vectors/matrices and nalgebra types:
//!
//! | R Type | nalgebra Type | Notes |
//! |--------|--------------|-------|
//! | REALSXP | `DVector<f64>`, `DMatrix<f64>` | Dynamic vectors/matrices |
//! | INTSXP | `DVector<i32>`, `DMatrix<i32>` | Integer vectors/matrices |
//! | REALSXP | `SVector<f64, D>`, `SMatrix<f64, R, C>` | Static (stack-allocated) |
//! | INTSXP | `SVector<i32, D>`, `SMatrix<i32, R, C>` | Static (stack-allocated) |
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

pub use nalgebra::{DMatrix, DVector, SMatrix, SVector};

use crate::ffi::{RNativeType, SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, SexpLengthError, TryFromSexp};
use crate::gc_protect::OwnedProtect;
use crate::into_r::IntoR;
use nalgebra::Scalar;

// region: Blanket implementations for DVector and DMatrix
//
// Now that we have blanket impls for `&[T]` where T: RNativeType, we can write
// blanket impls for nalgebra types instead of using macros.

/// Blanket impl for `DVector<T>` where T: RNativeType
impl<T> TryFromSexp for DVector<T>
where
    T: RNativeType + Scalar + Copy,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = TryFromSexp::try_from_sexp(sexp)?;
        Ok(DVector::from_column_slice(slice))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let slice: &[T] = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        Ok(DVector::from_column_slice(slice))
    }
}

/// Blanket impl for `DMatrix<T>` where T: RNativeType
impl<T> TryFromSexp for DMatrix<T>
where
    T: RNativeType + Scalar + Copy,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let (nrow, ncol) = get_matrix_dims(sexp)?;
        let slice: &[T] = TryFromSexp::try_from_sexp(sexp)?;

        if slice.len() != nrow * ncol {
            return Err(SexpLengthError {
                expected: nrow * ncol,
                actual: slice.len(),
            }
            .into());
        }

        Ok(DMatrix::from_column_slice(nrow, ncol, slice))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let (nrow, ncol) = get_matrix_dims(sexp)?;
        let slice: &[T] = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };

        if slice.len() != nrow * ncol {
            return Err(SexpLengthError {
                expected: nrow * ncol,
                actual: slice.len(),
            }
            .into());
        }

        Ok(DMatrix::from_column_slice(nrow, ncol, slice))
    }
}
// endregion

// region: DVector conversions

/// Convert `DVector<T>` to R vector.
impl<T: RNativeType + Scalar> IntoR for DVector<T> {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        let data: Vec<T> = self.data.into();
        Ok(data.into_sexp())
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        let data: Vec<T> = self.data.into();
        Ok(unsafe { data.into_sexp_unchecked() })
    }
}
// endregion

// region: DMatrix conversions

/// Convert `DMatrix<T>` to R matrix.
///
/// nalgebra stores data in column-major order (same as R), so this is efficient.
impl<T: RNativeType + Scalar> IntoR for DMatrix<T> {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        let nrow = self.nrows();
        let ncol = self.ncols();

        // nalgebra data is already column-major
        let data: Vec<T> = self.data.into();

        // Create R matrix with RAII protection
        Ok(unsafe {
            let mat = crate::ffi::Rf_allocMatrix(T::SEXP_TYPE, nrow as i32, ncol as i32);
            let guard = OwnedProtect::new(mat);

            let ptr = crate::ffi::DATAPTR_RO(guard.get()).cast_mut().cast::<T>();
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());

            // Return the SEXP - guard drops and unprotects
            guard.get()
        })
    }
}
// endregion

// region: Helper functions

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
// endregion

// region: Static (stack-allocated) vector and matrix: SVector and SMatrix
//
// SVector<T, D> and SMatrix<T, R, C> are stack-allocated, compile-time sized
// nalgebra types. They're useful when dimensions are known at compile time:
//
// - SVector<f64, 3> - 3D point/vector (24 bytes on stack)
// - SMatrix<f64, 4, 4> - transformation matrix (128 bytes on stack)
//
// These avoid heap allocation and are more cache-friendly for small sizes.
//
// **Important**: `SVector<T, D>` is a type alias for `SMatrix<T, D, 1>`.
// We only implement conversions for SMatrix; SVector works through this alias.
//
// R Conversion Semantics:
// - SMatrix<T, R, C> ↔ R matrix with dim (R, C)
// - SVector<T, D> (= SMatrix<T, D, 1>) ↔ R column vector with dim (D, 1)
//
// If you want a plain R vector (no dim attr), use DVector instead.

/// Blanket impl for `SMatrix<T, R, C>` where T: RNativeType
///
/// Converts an R matrix to a statically-sized nalgebra matrix.
/// Fails if the R matrix dimensions don't match R×C.
impl<T, const R: usize, const C: usize> TryFromSexp for SMatrix<T, R, C>
where
    T: RNativeType + Scalar + Copy,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let (nrow, ncol) = get_matrix_dims(sexp)?;
        if nrow != R || ncol != C {
            return Err(SexpError::InvalidValue(format!(
                "expected {}x{} matrix, got {}x{}",
                R, C, nrow, ncol
            )));
        }

        let slice: &[T] = TryFromSexp::try_from_sexp(sexp)?;
        // SAFETY: dimensions validated above
        Ok(SMatrix::from_column_slice(slice))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let (nrow, ncol) = get_matrix_dims(sexp)?;
        if nrow != R || ncol != C {
            return Err(SexpError::InvalidValue(format!(
                "expected {}x{} matrix, got {}x{}",
                R, C, nrow, ncol
            )));
        }

        let slice: &[T] = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        Ok(SMatrix::from_column_slice(slice))
    }
}
// endregion

// region: SMatrix IntoR (also covers SVector since SVector<T,D> = SMatrix<T,D,1>)

/// Convert `SMatrix<T, R, C>` to R matrix.
///
/// nalgebra stores data in column-major order (same as R), so this is efficient.
impl<T: RNativeType + Scalar, const R: usize, const C: usize> IntoR for SMatrix<T, R, C> {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(unsafe {
            let mat = crate::ffi::Rf_allocMatrix(T::SEXP_TYPE, R as i32, C as i32);
            let guard = OwnedProtect::new(mat);

            let ptr = crate::ffi::DATAPTR_RO(guard.get()).cast_mut().cast::<T>();
            std::ptr::copy_nonoverlapping(self.as_slice().as_ptr(), ptr, R * C);

            guard.get()
        })
    }
}
// endregion

// region: Option<SVector> and Option<SMatrix> conversions

// Note: Option<SVector<T, D>> is handled by Option<SMatrix<T, D, 1>> below
// since SVector<T, D> = SMatrix<T, D, 1>.

impl<T, const R: usize, const C: usize> TryFromSexp for Option<SMatrix<T, R, C>>
where
    T: RNativeType + Scalar + Copy,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        SMatrix::<T, R, C>::try_from_sexp(sexp).map(Some)
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        unsafe { SMatrix::<T, R, C>::try_from_sexp_unchecked(sexp).map(Some) }
    }
}

// Note: Option<SVector<T, D>> IntoR is handled by Option<SMatrix<T, D, 1>> below.

impl<T: RNativeType + Scalar, const R: usize, const C: usize> IntoR for Option<SMatrix<T, R, C>> {
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(match self {
            Some(m) => m.into_sexp(),
            None => unsafe { crate::ffi::R_NilValue },
        })
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        Ok(match self {
            Some(m) => unsafe { m.into_sexp_unchecked() },
            None => unsafe { crate::ffi::R_NilValue },
        })
    }
}
// endregion

// region: TypedExternal implementations for ExternalPtr support
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

// --- SVector TypedExternal (common sizes) ---
// 2D, 3D, 4D vectors (graphics, physics)
impl_te_nalgebra!(SVector<f64, 2>, "nalgebra::SVector<f64,2>");
impl_te_nalgebra!(SVector<f64, 3>, "nalgebra::SVector<f64,3>");
impl_te_nalgebra!(SVector<f64, 4>, "nalgebra::SVector<f64,4>");
impl_te_nalgebra!(SVector<i32, 2>, "nalgebra::SVector<i32,2>");
impl_te_nalgebra!(SVector<i32, 3>, "nalgebra::SVector<i32,3>");
impl_te_nalgebra!(SVector<i32, 4>, "nalgebra::SVector<i32,4>");

// --- SMatrix TypedExternal (common sizes) ---
// 2x2, 3x3, 4x4 transformation matrices (graphics, physics)
impl_te_nalgebra!(SMatrix<f64, 2, 2>, "nalgebra::SMatrix<f64,2,2>");
impl_te_nalgebra!(SMatrix<f64, 3, 3>, "nalgebra::SMatrix<f64,3,3>");
impl_te_nalgebra!(SMatrix<f64, 4, 4>, "nalgebra::SMatrix<f64,4,4>");
impl_te_nalgebra!(SMatrix<i32, 2, 2>, "nalgebra::SMatrix<i32,2,2>");
impl_te_nalgebra!(SMatrix<i32, 3, 3>, "nalgebra::SMatrix<i32,3,3>");
impl_te_nalgebra!(SMatrix<i32, 4, 4>, "nalgebra::SMatrix<i32,4,4>");
// endregion

// region: RVectorOps adapter trait

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
// endregion

// region: RMatrixOps adapter trait

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
// endregion

// region: Coerced element support
//
// Support for `DVector<Coerced<T, R>>` and `DMatrix<Coerced<T, R>>`.
// This allows reading R native types (i32, f64) and coercing to non-native types
// (i8, f32, etc.) element-wise.
//
// Example: `DVector<Coerced<f32, f64>>` reads from R numeric and coerces each
// element to f32.

use crate::coerce::{Coerced, TryCoerce};

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

/// TryFromSexp for `DVector<Coerced<T, R>>` - reads R native type and coerces.
impl<T, R> TryFromSexp for DVector<Coerced<T, R>>
where
    R: RNativeType + Scalar + Copy + TryCoerce<T>,
    T: Scalar + Copy,
    Coerced<T, R>: Scalar,
    <R as TryCoerce<T>>::Error: std::fmt::Debug,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != R::SEXP_TYPE {
            return Err(crate::from_r::SexpTypeError {
                expected: R::SEXP_TYPE,
                actual,
            }
            .into());
        }
        let slice: &[R] = unsafe { sexp.as_slice() };
        let data: Vec<T> = coerce_slice(slice)?;
        let coerced_data: Vec<Coerced<T, R>> = data.into_iter().map(Coerced::new).collect();
        Ok(DVector::from_vec(coerced_data))
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

/// TryFromSexp for `DMatrix<Coerced<T, R>>` - reads R native type and coerces.
impl<T, R> TryFromSexp for DMatrix<Coerced<T, R>>
where
    R: RNativeType + Scalar + Copy + TryCoerce<T>,
    T: Scalar + Copy,
    Coerced<T, R>: Scalar,
    <R as TryCoerce<T>>::Error: std::fmt::Debug,
{
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != R::SEXP_TYPE {
            return Err(crate::from_r::SexpTypeError {
                expected: R::SEXP_TYPE,
                actual,
            }
            .into());
        }
        let (nrow, ncol) = get_matrix_dims(sexp)?;
        let slice: &[R] = unsafe { sexp.as_slice() };
        if slice.len() != nrow * ncol {
            return Err(SexpLengthError {
                expected: nrow * ncol,
                actual: slice.len(),
            }
            .into());
        }
        let data: Vec<T> = coerce_slice(slice)?;
        let coerced_data: Vec<Coerced<T, R>> = data.into_iter().map(Coerced::new).collect();
        Ok(DMatrix::from_vec(nrow, ncol, coerced_data))
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        Self::try_from_sexp(sexp)
    }
}

/// IntoR for `DVector<Coerced<T, R>>` - coerces back and writes to R.
impl<T, R> IntoR for DVector<Coerced<T, R>>
where
    T: Copy + Into<R> + Scalar,
    R: RNativeType + Scalar + Copy,
    Coerced<T, R>: Scalar,
{
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        let r_values: Vec<R> = self.iter().map(|c| (*c.as_inner()).into()).collect();
        Ok(DVector::from_vec(r_values).into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        let r_values: Vec<R> = self.iter().map(|c| (*c.as_inner()).into()).collect();
        Ok(unsafe { DVector::from_vec(r_values).into_sexp_unchecked() })
    }
}

/// IntoR for `DMatrix<Coerced<T, R>>` - coerces back and writes to R.
impl<T, R> IntoR for DMatrix<Coerced<T, R>>
where
    T: Copy + Into<R> + Scalar,
    R: RNativeType + Scalar + Copy,
    Coerced<T, R>: Scalar,
{
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        let nrow = self.nrows();
        let ncol = self.ncols();
        let r_values: Vec<R> = self.iter().map(|c| (*c.as_inner()).into()).collect();
        Ok(DMatrix::from_vec(nrow, ncol, r_values).into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        let nrow = self.nrows();
        let ncol = self.ncols();
        let r_values: Vec<R> = self.iter().map(|c| (*c.as_inner()).into()).collect();
        Ok(unsafe { DMatrix::from_vec(nrow, ncol, r_values).into_sexp_unchecked() })
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

    // region: SVector and SMatrix tests

    #[test]
    fn svector_can_be_created() {
        let v: SVector<f64, 3> = SVector::from_column_slice(&[1.0, 2.0, 3.0]);
        assert_eq!(v.len(), 3);
        assert!((v[0] - 1.0).abs() < 1e-10);
        assert!((v[1] - 2.0).abs() < 1e-10);
        assert!((v[2] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn smatrix_can_be_created() {
        // 2x3 matrix in column-major order
        let m: SMatrix<f64, 2, 3> = SMatrix::from_column_slice(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(m.nrows(), 2);
        assert_eq!(m.ncols(), 3);
        // Column-major: m = [[1, 3, 5], [2, 4, 6]]
        assert!((m[(0, 0)] - 1.0).abs() < 1e-10);
        assert!((m[(1, 0)] - 2.0).abs() < 1e-10);
        assert!((m[(0, 1)] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn svector_type_alias() {
        // SVector<T, D> is SMatrix<T, D, 1>
        let v: SVector<f64, 3> = SVector::from_column_slice(&[1.0, 2.0, 3.0]);
        let m: SMatrix<f64, 3, 1> = SMatrix::from_column_slice(&[1.0, 2.0, 3.0]);

        // They're the same type
        assert_eq!(v.nrows(), m.nrows());
        assert_eq!(v.ncols(), m.ncols());
        assert_eq!(v.as_slice(), m.as_slice());
    }

    #[test]
    fn smatrix_operations() {
        let m: SMatrix<f64, 2, 2> = SMatrix::from_column_slice(&[1.0, 2.0, 3.0, 4.0]);
        // Column-major: [[1, 3], [2, 4]]
        assert!((m.determinant() - -2.0).abs() < 1e-10);
        assert!((m.trace() - 5.0).abs() < 1e-10);

        let t = m.transpose();
        assert_eq!(t.nrows(), 2);
        assert_eq!(t.ncols(), 2);
        // Transposed: [[1, 2], [3, 4]]
        assert!((t[(0, 1)] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn svector_norm() {
        let v: SVector<f64, 2> = SVector::from_column_slice(&[3.0, 4.0]);
        assert!((v.norm() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn smatrix_i32() {
        let m: SMatrix<i32, 2, 2> = SMatrix::from_column_slice(&[1, 2, 3, 4]);
        assert_eq!(m.nrows(), 2);
        assert_eq!(m.ncols(), 2);
        assert_eq!(m[(0, 0)], 1);
        assert_eq!(m[(1, 1)], 4);
    }
    // endregion
}
// endregion
