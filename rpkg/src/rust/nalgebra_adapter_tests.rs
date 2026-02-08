//! nalgebra adapter tests
use miniextendr_api::nalgebra_impl::{DMatrix, DVector, RMatrixOps, RVectorOps};
use miniextendr_api::{miniextendr, miniextendr_module};

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

miniextendr_module! {
    mod nalgebra_adapter_tests;
    fn nalgebra_dvector_roundtrip;
    fn nalgebra_dvector_len;
    fn nalgebra_dvector_sum;
    fn nalgebra_dvector_norm;
    fn nalgebra_dvector_dot;
    fn nalgebra_dmatrix_roundtrip;
    fn nalgebra_dmatrix_nrows;
    fn nalgebra_dmatrix_ncols;
    fn nalgebra_dmatrix_transpose;
    fn nalgebra_dmatrix_trace;
}
