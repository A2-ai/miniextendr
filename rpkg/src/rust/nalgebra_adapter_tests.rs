//! nalgebra adapter tests
use miniextendr_api::miniextendr;
use miniextendr_api::nalgebra_impl::{DMatrix, DVector, RMatrixOps, RVectorOps};

/// @noRd
#[miniextendr]
pub fn nalgebra_dvector_roundtrip(v: DVector<f64>) -> DVector<f64> {
    v
}

/// @noRd
#[miniextendr]
pub fn nalgebra_dvector_len(v: DVector<f64>) -> i32 {
    RVectorOps::len(&v)
}

/// @noRd
#[miniextendr]
pub fn nalgebra_dvector_sum(v: DVector<f64>) -> f64 {
    RVectorOps::sum(&v)
}

/// @noRd
#[miniextendr]
pub fn nalgebra_dvector_norm(v: DVector<f64>) -> f64 {
    v.norm()
}

/// @noRd
#[miniextendr]
pub fn nalgebra_dvector_dot(a: DVector<f64>, b: DVector<f64>) -> f64 {
    RVectorOps::dot(&a, &b)
}

/// @noRd
#[miniextendr]
pub fn nalgebra_dmatrix_roundtrip(m: DMatrix<f64>) -> DMatrix<f64> {
    m
}

/// @noRd
#[miniextendr]
pub fn nalgebra_dmatrix_nrows(m: DMatrix<f64>) -> i32 {
    RMatrixOps::nrows(&m)
}

/// @noRd
#[miniextendr]
pub fn nalgebra_dmatrix_ncols(m: DMatrix<f64>) -> i32 {
    RMatrixOps::ncols(&m)
}

/// @noRd
#[miniextendr]
pub fn nalgebra_dmatrix_transpose(m: DMatrix<f64>) -> DMatrix<f64> {
    RMatrixOps::transpose(&m)
}

/// @noRd
#[miniextendr]
pub fn nalgebra_dmatrix_trace(m: DMatrix<f64>) -> f64 {
    RMatrixOps::trace(&m)
}
