//! nalgebra adapter tests
use miniextendr_api::miniextendr;
use miniextendr_api::nalgebra_impl::{DMatrix, DVector, RMatrixOps, RVectorOps};

/// Test DVector<f64> roundtrip through R.
/// @param v Numeric vector from R.
#[miniextendr]
pub fn nalgebra_dvector_roundtrip(v: DVector<f64>) -> DVector<f64> {
    v
}

/// Test getting the length of a DVector.
/// @param v Numeric vector from R.
#[miniextendr]
pub fn nalgebra_dvector_len(v: DVector<f64>) -> i32 {
    RVectorOps::len(&v)
}

/// Test computing the sum of DVector elements.
/// @param v Numeric vector from R.
#[miniextendr]
pub fn nalgebra_dvector_sum(v: DVector<f64>) -> f64 {
    RVectorOps::sum(&v)
}

/// Test computing the Euclidean norm of a DVector.
/// @param v Numeric vector from R.
#[miniextendr]
pub fn nalgebra_dvector_norm(v: DVector<f64>) -> f64 {
    v.norm()
}

/// Test computing the dot product of two DVectors.
/// @param a First numeric vector from R.
/// @param b Second numeric vector from R.
#[miniextendr]
pub fn nalgebra_dvector_dot(a: DVector<f64>, b: DVector<f64>) -> f64 {
    RVectorOps::dot(&a, &b)
}

/// Test DMatrix<f64> roundtrip through R.
/// @param m Numeric matrix from R.
#[miniextendr]
pub fn nalgebra_dmatrix_roundtrip(m: DMatrix<f64>) -> DMatrix<f64> {
    m
}

/// Test getting the number of rows in a DMatrix.
/// @param m Numeric matrix from R.
#[miniextendr]
pub fn nalgebra_dmatrix_nrows(m: DMatrix<f64>) -> i32 {
    RMatrixOps::nrows(&m)
}

/// Test getting the number of columns in a DMatrix.
/// @param m Numeric matrix from R.
#[miniextendr]
pub fn nalgebra_dmatrix_ncols(m: DMatrix<f64>) -> i32 {
    RMatrixOps::ncols(&m)
}

/// Test transposing a DMatrix.
/// @param m Numeric matrix from R.
#[miniextendr]
pub fn nalgebra_dmatrix_transpose(m: DMatrix<f64>) -> DMatrix<f64> {
    RMatrixOps::transpose(&m)
}

/// Test computing the trace of a DMatrix.
/// @param m Numeric square matrix from R.
#[miniextendr]
pub fn nalgebra_dmatrix_trace(m: DMatrix<f64>) -> f64 {
    RMatrixOps::trace(&m)
}
