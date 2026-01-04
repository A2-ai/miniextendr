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

use crate::ffi::{RNativeType, SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, SexpLengthError, SexpTypeError, TryFromSexp};
use crate::gc_protect::OwnedProtect;
use crate::into_r::IntoR;
use nalgebra::Scalar;

// =============================================================================
// DVector conversions
// =============================================================================

/// Convert R vector to `DVector<T>`.
impl<T> TryFromSexp for DVector<T>
where
    T: RNativeType + Scalar + Clone,
{
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
        Ok(DVector::from_column_slice(slice))
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
        Ok(DVector::from_column_slice(slice))
    }
}

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

/// Convert R matrix to `DMatrix<T>`.
///
/// Both R and nalgebra use column-major storage, so this is efficient.
impl<T> TryFromSexp for DMatrix<T>
where
    T: RNativeType + Scalar + Clone,
{
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
        let (nrow, ncol) = get_matrix_dims(sexp)?;

        let slice: &[T] = unsafe { sexp.as_slice() };

        if slice.len() != nrow * ncol {
            return Err(SexpLengthError {
                expected: nrow * ncol,
                actual: slice.len(),
            }
            .into());
        }

        // nalgebra uses column-major by default, same as R
        Ok(DMatrix::from_column_slice(nrow, ncol, slice))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        // Type check is still needed
        Self::try_from_sexp(sexp)
    }
}

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

            guard.into_inner()
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
}
