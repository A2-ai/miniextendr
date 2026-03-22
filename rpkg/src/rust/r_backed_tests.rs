//! R-backed zero-copy storage tests for nalgebra and ndarray.

use miniextendr_api::miniextendr;
use miniextendr_api::nalgebra_impl::{RDMatrix, RDVector};
use miniextendr_api::ndarray_impl::{RndMat, RndVec};

// region: nalgebra RDVector tests

/// @noRd
#[miniextendr]
pub fn r_backed_rdvector_roundtrip(v: RDVector<f64>) -> RDVector<f64> {
    v
}

/// @noRd
#[miniextendr]
pub fn r_backed_rdvector_norm(v: RDVector<f64>) -> f64 {
    v.norm()
}

/// @noRd
#[miniextendr]
pub fn r_backed_rdvector_sum(v: RDVector<f64>) -> f64 {
    v.iter().sum()
}

/// @noRd
#[miniextendr]
pub fn r_backed_rdvector_len(v: RDVector<f64>) -> i32 {
    v.len() as i32
}

/// @noRd
#[miniextendr]
pub fn r_backed_rdvector_dot(a: RDVector<f64>, b: RDVector<f64>) -> f64 {
    a.dot(&b)
}

/// @noRd
#[miniextendr]
pub fn r_backed_rdvector_scale(v: RDVector<f64>, factor: f64) -> RDVector<f64> {
    let mut v = v;
    v *= factor;
    v
}

/// @noRd
#[miniextendr]
pub fn r_backed_rdvector_int_roundtrip(v: RDVector<i32>) -> RDVector<i32> {
    v
}

/// @noRd
#[miniextendr]
pub fn r_backed_rdvector_int_sum(v: RDVector<i32>) -> i32 {
    v.iter().sum()
}

// endregion

// region: nalgebra RDMatrix tests

/// @noRd
#[miniextendr]
pub fn r_backed_rdmatrix_roundtrip(m: RDMatrix<f64>) -> RDMatrix<f64> {
    m
}

/// @noRd
#[miniextendr]
pub fn r_backed_rdmatrix_nrow(m: RDMatrix<f64>) -> i32 {
    m.nrows() as i32
}

/// @noRd
#[miniextendr]
pub fn r_backed_rdmatrix_ncol(m: RDMatrix<f64>) -> i32 {
    m.ncols() as i32
}

/// @noRd
#[miniextendr]
pub fn r_backed_rdmatrix_sum(m: RDMatrix<f64>) -> f64 {
    m.iter().sum()
}

/// @noRd
#[miniextendr]
pub fn r_backed_rdmatrix_trace(m: RDMatrix<f64>) -> f64 {
    m.trace()
}

/// @noRd
#[miniextendr]
pub fn r_backed_rdmatrix_scale(m: RDMatrix<f64>, factor: f64) -> RDMatrix<f64> {
    let mut m = m;
    m *= factor;
    m
}

// endregion

// region: ndarray RndVec tests

/// @noRd
#[miniextendr]
pub fn r_backed_rndvec_roundtrip(v: RndVec<f64>) -> RndVec<f64> {
    v
}

/// @noRd
#[miniextendr]
pub fn r_backed_rndvec_sum(v: RndVec<f64>) -> f64 {
    v.view().sum()
}

/// @noRd
#[miniextendr]
pub fn r_backed_rndvec_len(v: RndVec<f64>) -> i32 {
    v.len() as i32
}

/// @noRd
#[miniextendr]
pub fn r_backed_rndvec_double(v: RndVec<f64>) -> RndVec<f64> {
    let input = v.view();
    let mut result = unsafe { RndVec::<f64>::new(input.len(), |s| s.fill(0.0)) };
    result.view_mut().assign(&(&input * 2.0));
    result
}

/// @noRd
#[miniextendr]
pub fn r_backed_rndvec_int_roundtrip(v: RndVec<i32>) -> RndVec<i32> {
    v
}

// endregion

// region: ndarray RndMat tests

/// @noRd
#[miniextendr]
pub fn r_backed_rndmat_roundtrip(m: RndMat<f64>) -> RndMat<f64> {
    m
}

/// @noRd
#[miniextendr]
pub fn r_backed_rndmat_sum(m: RndMat<f64>) -> f64 {
    m.view().sum()
}

/// @noRd
#[miniextendr]
pub fn r_backed_rndmat_nrow(m: RndMat<f64>) -> i32 {
    m.nrow() as i32
}

/// @noRd
#[miniextendr]
pub fn r_backed_rndmat_ncol(m: RndMat<f64>) -> i32 {
    m.ncol() as i32
}

/// @noRd
#[miniextendr]
pub fn r_backed_rndmat_trace(m: RndMat<f64>) -> f64 {
    m.view().diag().sum()
}

/// @noRd
#[miniextendr]
pub fn r_backed_rndmat_fill(m: RndMat<f64>, value: f64) -> RndMat<f64> {
    let mut m = m;
    m.view_mut().fill(value);
    m
}

// endregion

// region: empty vector edge cases

/// @noRd
#[miniextendr]
pub fn r_backed_rdvector_empty_roundtrip(v: RDVector<f64>) -> RDVector<f64> {
    v
}

/// @noRd
#[miniextendr]
pub fn r_backed_rndvec_empty_sum(v: RndVec<f64>) -> f64 {
    v.view().sum()
}

// endregion
