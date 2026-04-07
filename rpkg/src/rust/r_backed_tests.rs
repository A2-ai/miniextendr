//! R-backed zero-copy storage tests for nalgebra and ndarray.

use miniextendr_api::miniextendr;
use miniextendr_api::nalgebra_impl::{RDMatrix, RDVector};
use miniextendr_api::ndarray_impl::{RndMat, RndVec};

// region: nalgebra RDVector tests

/// Test nalgebra RDVector<f64> identity roundtrip through R.
/// @param v Numeric vector input.
#[miniextendr]
pub fn r_backed_rdvector_roundtrip(v: RDVector<f64>) -> RDVector<f64> {
    v
}

/// Test computing the Euclidean norm of an RDVector<f64>.
/// @param v Numeric vector input.
#[miniextendr]
pub fn r_backed_rdvector_norm(v: RDVector<f64>) -> f64 {
    v.norm()
}

/// Test summing all elements of an RDVector<f64>.
/// @param v Numeric vector input.
#[miniextendr]
pub fn r_backed_rdvector_sum(v: RDVector<f64>) -> f64 {
    v.iter().sum()
}

/// Test getting the length of an RDVector<f64>.
/// @param v Numeric vector input.
#[miniextendr]
pub fn r_backed_rdvector_len(v: RDVector<f64>) -> i32 {
    v.len() as i32
}

/// Test computing the dot product of two RDVector<f64>.
/// @param a Numeric vector input.
/// @param b Numeric vector input.
#[miniextendr]
pub fn r_backed_rdvector_dot(a: RDVector<f64>, b: RDVector<f64>) -> f64 {
    a.dot(&b)
}

/// Test scaling an RDVector<f64> by a scalar factor.
/// @param v Numeric vector input.
/// @param factor Numeric scaling factor.
#[miniextendr]
pub fn r_backed_rdvector_scale(v: RDVector<f64>, factor: f64) -> RDVector<f64> {
    let mut v = v;
    v *= factor;
    v
}

/// Test nalgebra RDVector<i32> identity roundtrip through R.
/// @param v Integer vector input.
#[miniextendr]
pub fn r_backed_rdvector_int_roundtrip(v: RDVector<i32>) -> RDVector<i32> {
    v
}

/// Test summing all elements of an RDVector<i32>.
/// @param v Integer vector input.
#[miniextendr]
pub fn r_backed_rdvector_int_sum(v: RDVector<i32>) -> i32 {
    v.iter().sum()
}

// endregion

// region: nalgebra RDMatrix tests

/// Test nalgebra RDMatrix<f64> identity roundtrip through R.
/// @param m Numeric matrix input.
#[miniextendr]
pub fn r_backed_rdmatrix_roundtrip(m: RDMatrix<f64>) -> RDMatrix<f64> {
    m
}

/// Test getting the number of rows of an RDMatrix<f64>.
/// @param m Numeric matrix input.
#[miniextendr]
pub fn r_backed_rdmatrix_nrow(m: RDMatrix<f64>) -> i32 {
    m.nrows() as i32
}

/// Test getting the number of columns of an RDMatrix<f64>.
/// @param m Numeric matrix input.
#[miniextendr]
pub fn r_backed_rdmatrix_ncol(m: RDMatrix<f64>) -> i32 {
    m.ncols() as i32
}

/// Test summing all elements of an RDMatrix<f64>.
/// @param m Numeric matrix input.
#[miniextendr]
pub fn r_backed_rdmatrix_sum(m: RDMatrix<f64>) -> f64 {
    m.iter().sum()
}

/// Test computing the trace (sum of diagonal) of an RDMatrix<f64>.
/// @param m Numeric square matrix input.
#[miniextendr]
pub fn r_backed_rdmatrix_trace(m: RDMatrix<f64>) -> f64 {
    m.trace()
}

/// Test scaling all elements of an RDMatrix<f64> by a scalar factor.
/// @param m Numeric matrix input.
/// @param factor Numeric scaling factor.
#[miniextendr]
pub fn r_backed_rdmatrix_scale(m: RDMatrix<f64>, factor: f64) -> RDMatrix<f64> {
    let mut m = m;
    m *= factor;
    m
}

// endregion

// region: ndarray RndVec tests

/// Test ndarray RndVec<f64> identity roundtrip through R.
/// @param v Numeric vector input.
#[miniextendr]
pub fn r_backed_rndvec_roundtrip(v: RndVec<f64>) -> RndVec<f64> {
    v
}

/// Test summing all elements of an RndVec<f64>.
/// @param v Numeric vector input.
#[miniextendr]
pub fn r_backed_rndvec_sum(v: RndVec<f64>) -> f64 {
    v.view().sum()
}

/// Test getting the length of an RndVec<f64>.
/// @param v Numeric vector input.
#[miniextendr]
pub fn r_backed_rndvec_len(v: RndVec<f64>) -> i32 {
    v.len() as i32
}

/// Test doubling each element of an RndVec<f64> via ndarray operations.
/// @param v Numeric vector input.
#[miniextendr]
pub fn r_backed_rndvec_double(v: RndVec<f64>) -> RndVec<f64> {
    let input = v.view();
    let mut result = unsafe { RndVec::<f64>::new(input.len(), |s| s.fill(0.0)) };
    result.view_mut().assign(&(&input * 2.0));
    result
}

/// Test ndarray RndVec<i32> identity roundtrip through R.
/// @param v Integer vector input.
#[miniextendr]
pub fn r_backed_rndvec_int_roundtrip(v: RndVec<i32>) -> RndVec<i32> {
    v
}

// endregion

// region: ndarray RndMat tests

/// Test ndarray RndMat<f64> identity roundtrip through R.
/// @param m Numeric matrix input.
#[miniextendr]
pub fn r_backed_rndmat_roundtrip(m: RndMat<f64>) -> RndMat<f64> {
    m
}

/// Test summing all elements of an RndMat<f64>.
/// @param m Numeric matrix input.
#[miniextendr]
pub fn r_backed_rndmat_sum(m: RndMat<f64>) -> f64 {
    m.view().sum()
}

/// Test getting the number of rows of an RndMat<f64>.
/// @param m Numeric matrix input.
#[miniextendr]
pub fn r_backed_rndmat_nrow(m: RndMat<f64>) -> i32 {
    m.nrow() as i32
}

/// Test getting the number of columns of an RndMat<f64>.
/// @param m Numeric matrix input.
#[miniextendr]
pub fn r_backed_rndmat_ncol(m: RndMat<f64>) -> i32 {
    m.ncol() as i32
}

/// Test computing the trace (sum of diagonal) of an RndMat<f64>.
/// @param m Numeric square matrix input.
#[miniextendr]
pub fn r_backed_rndmat_trace(m: RndMat<f64>) -> f64 {
    m.view().diag().sum()
}

/// Test filling all elements of an RndMat<f64> with a constant value.
/// @param m Numeric matrix input.
/// @param value Numeric fill value.
#[miniextendr]
pub fn r_backed_rndmat_fill(m: RndMat<f64>, value: f64) -> RndMat<f64> {
    let mut m = m;
    m.view_mut().fill(value);
    m
}

// endregion

// region: empty vector edge cases

/// Test RDVector<f64> roundtrip with an empty (length-0) vector.
/// @param v Numeric vector input (expected empty).
#[miniextendr]
pub fn r_backed_rdvector_empty_roundtrip(v: RDVector<f64>) -> RDVector<f64> {
    v
}

/// Test summing an empty RndVec<f64> (should return 0.0).
/// @param v Numeric vector input (expected empty).
#[miniextendr]
pub fn r_backed_rndvec_empty_sum(v: RndVec<f64>) -> f64 {
    v.view().sum()
}

// endregion
