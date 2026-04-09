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

// region: Upstream example-derived fixtures

/// Solve a linear system Ax = b via LU decomposition.
/// @param a_flat Flattened square matrix A (column-major).
/// @param b Right-hand side vector.
/// @param n Matrix dimension (n x n).
#[miniextendr]
pub fn nalgebra_solve(a_flat: Vec<f64>, b: Vec<f64>, n: i32) -> Option<Vec<f64>> {
    let n = n as usize;
    let a = DMatrix::from_column_slice(n, n, &a_flat);
    let b = DVector::from_column_slice(&b);
    a.lu().solve(&b).map(|x| x.data.into())
}

/// Compute the determinant of a square matrix.
/// @param m Numeric square matrix from R.
#[miniextendr]
pub fn nalgebra_determinant(m: DMatrix<f64>) -> f64 {
    m.determinant()
}

/// Compute the inverse of a square matrix. Panics if singular.
/// @param m Numeric square matrix from R.
#[miniextendr]
pub fn nalgebra_inverse(m: DMatrix<f64>) -> DMatrix<f64> {
    m.clone().try_inverse().expect("matrix is singular")
}

/// Compute real eigenvalues of a symmetric matrix.
/// Returns eigenvalues sorted in ascending order.
/// @param m Numeric symmetric matrix from R.
#[miniextendr]
pub fn nalgebra_eigenvalues(m: DMatrix<f64>) -> Vec<f64> {
    let eigen = m.symmetric_eigen();
    let mut vals: Vec<f64> = eigen.eigenvalues.iter().copied().collect();
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    vals
}

/// Construct a DMatrix from a row slice (row-major input).
/// @param data Flattened row-major data.
/// @param nrow Number of rows.
/// @param ncol Number of columns.
#[miniextendr]
pub fn nalgebra_from_row_slice(data: Vec<f64>, nrow: i32, ncol: i32) -> DMatrix<f64> {
    DMatrix::from_row_slice(nrow as usize, ncol as usize, &data)
}

/// Construct a DMatrix where each element equals row * ncol + col.
/// @param nrow Number of rows.
/// @param ncol Number of columns.
#[miniextendr]
pub fn nalgebra_from_fn(nrow: i32, ncol: i32) -> DMatrix<f64> {
    let nr = nrow as usize;
    let nc = ncol as usize;
    DMatrix::from_fn(nr, nc, |r, c| (r * nc + c) as f64)
}

/// Roundtrip a static SVector<f64, 3> through Rust.
/// @param x First element.
/// @param y Second element.
/// @param z Third element.
#[miniextendr]
pub fn nalgebra_svector3_roundtrip(x: f64, y: f64, z: f64) -> Vec<f64> {
    use miniextendr_api::nalgebra_impl::SVector;
    let v = SVector::<f64, 3>::new(x, y, z);
    vec![v[0], v[1], v[2]]
}

/// Reshape a matrix to new dimensions (total elements must match).
/// @param m Numeric matrix from R.
/// @param new_nrow New number of rows.
/// @param new_ncol New number of columns.
#[miniextendr]
pub fn nalgebra_reshape(m: DMatrix<f64>, new_nrow: i32, new_ncol: i32) -> DMatrix<f64> {
    let data: Vec<f64> = m.data.into();
    DMatrix::from_column_slice(new_nrow as usize, new_ncol as usize, &data)
}

// endregion
