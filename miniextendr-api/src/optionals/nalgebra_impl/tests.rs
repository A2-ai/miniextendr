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

mod altrep_no_na_tests;
