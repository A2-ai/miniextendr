mod r_test_utils;

#[cfg(feature = "nalgebra")]
use miniextendr_api::from_r::TryFromSexp;
#[cfg(feature = "nalgebra")]
use miniextendr_api::into_r::IntoR;
#[cfg(feature = "nalgebra")]
use miniextendr_api::{DMatrix, DVector, SMatrix, SVector};

#[cfg(feature = "nalgebra")]
#[test]
fn dvector_i32_blanket_impl() {
    // Verify blanket impl works for i32 (not just f64)
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{Rf_allocVector, Rf_protect, Rf_unprotect, SEXPTYPE, SexpExt};

        unsafe {
            let sexp = Rf_protect(Rf_allocVector(SEXPTYPE::INTSXP, 3));
            let slice: &mut [i32] = sexp.as_mut_slice();
            slice[0] = 10;
            slice[1] = 20;
            slice[2] = 30;

            let vec: DVector<i32> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(vec.len(), 3);
            assert_eq!(vec[0], 10);
            assert_eq!(vec[1], 20);
            assert_eq!(vec[2], 30);

            Rf_unprotect(1);
        }
    });
}

#[cfg(feature = "nalgebra")]
#[test]
fn dvector_f64_roundtrip() {
    r_test_utils::with_r_thread(|| {
        use nalgebra::DVector;

        let vec = DVector::from_vec(vec![1.5, 2.5, 3.5]);
        let sexp = vec.clone().into_sexp();
        let back: DVector<f64> = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back.len(), 3);
        assert_eq!(back[0], 1.5);
        assert_eq!(back[1], 2.5);
        assert_eq!(back[2], 3.5);
    });
}

#[cfg(feature = "nalgebra")]
#[test]
fn dmatrix_i32_blanket_impl() {
    // Verify blanket impl works for i32
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{Rf_allocMatrix, Rf_protect, Rf_unprotect, SEXPTYPE, SexpExt};

        unsafe {
            // Create 2x3 matrix
            let sexp = Rf_protect(Rf_allocMatrix(SEXPTYPE::INTSXP, 2, 3));
            let slice: &mut [i32] = sexp.as_mut_slice();
            // Column-major: [col1, col2, col3]
            slice[0] = 1; // row 1, col 1
            slice[1] = 2; // row 2, col 1
            slice[2] = 3; // row 1, col 2
            slice[3] = 4; // row 2, col 2
            slice[4] = 5; // row 1, col 3
            slice[5] = 6; // row 2, col 3

            let mat: DMatrix<i32> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(mat.nrows(), 2);
            assert_eq!(mat.ncols(), 3);
            assert_eq!(mat[(0, 0)], 1);
            assert_eq!(mat[(1, 0)], 2);
            assert_eq!(mat[(0, 1)], 3);
            assert_eq!(mat[(1, 1)], 4);
            assert_eq!(mat[(0, 2)], 5);
            assert_eq!(mat[(1, 2)], 6);

            Rf_unprotect(1);
        }
    });
}

#[cfg(feature = "nalgebra")]
#[test]
fn dmatrix_u8_blanket_impl() {
    // Verify blanket impl works for u8 (raw bytes)
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{Rf_allocMatrix, Rf_protect, Rf_unprotect, SEXPTYPE, SexpExt};

        unsafe {
            // Create 2x2 raw matrix
            let sexp = Rf_protect(Rf_allocMatrix(SEXPTYPE::RAWSXP, 2, 2));
            let slice: &mut [u8] = sexp.as_mut_slice();
            slice[0] = 10;
            slice[1] = 20;
            slice[2] = 30;
            slice[3] = 40;

            let mat: DMatrix<u8> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(mat.nrows(), 2);
            assert_eq!(mat.ncols(), 2);
            assert_eq!(mat[(0, 0)], 10);
            assert_eq!(mat[(1, 0)], 20);
            assert_eq!(mat[(0, 1)], 30);
            assert_eq!(mat[(1, 1)], 40);

            Rf_unprotect(1);
        }
    });
}

#[cfg(feature = "nalgebra")]
#[test]
fn dvector_empty() {
    r_test_utils::with_r_thread(|| {
        use nalgebra::DVector;

        let vec: DVector<f64> = DVector::from_vec(vec![]);
        let sexp = vec.into_sexp();
        let back: DVector<f64> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(back.len(), 0);
    });
}

// region: SVector tests (statically-sized vectors)

#[cfg(feature = "nalgebra")]
#[test]
fn svector_f64_roundtrip() {
    r_test_utils::with_r_thread(|| {
        // Note: SVector<T, D> = SMatrix<T, D, 1>, so it becomes an Rx1 matrix in R
        let vec: SVector<f64, 3> = SVector::from_column_slice(&[1.5, 2.5, 3.5]);
        let sexp = vec.into_sexp();

        // Convert back - expects a 3x1 matrix (column vector)
        let back: SVector<f64, 3> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(back.len(), 3);
        assert!((back[0] - 1.5).abs() < 1e-10);
        assert!((back[1] - 2.5).abs() < 1e-10);
        assert!((back[2] - 3.5).abs() < 1e-10);
    });
}

#[cfg(feature = "nalgebra")]
#[test]
fn svector_i32_from_r() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{Rf_allocMatrix, Rf_protect, Rf_unprotect, SEXPTYPE, SexpExt};

        unsafe {
            // Create 3x1 integer matrix (column vector)
            let sexp = Rf_protect(Rf_allocMatrix(SEXPTYPE::INTSXP, 3, 1));
            let slice: &mut [i32] = sexp.as_mut_slice();
            slice[0] = 10;
            slice[1] = 20;
            slice[2] = 30;

            let vec: SVector<i32, 3> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(vec.len(), 3);
            assert_eq!(vec[0], 10);
            assert_eq!(vec[1], 20);
            assert_eq!(vec[2], 30);

            Rf_unprotect(1);
        }
    });
}

#[cfg(feature = "nalgebra")]
#[test]
fn svector_length_mismatch() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{Rf_allocMatrix, Rf_protect, Rf_unprotect, SEXPTYPE, SexpExt};

        unsafe {
            // Create 4x1 matrix, but try to convert to SVector<f64, 3>
            let sexp = Rf_protect(Rf_allocMatrix(SEXPTYPE::REALSXP, 4, 1));
            let slice: &mut [f64] = sexp.as_mut_slice();
            slice[0] = 1.0;
            slice[1] = 2.0;
            slice[2] = 3.0;
            slice[3] = 4.0;

            let result: Result<SVector<f64, 3>, _> = TryFromSexp::try_from_sexp(sexp);
            assert!(result.is_err(), "Should fail: expected 3x1, got 4x1");

            Rf_unprotect(1);
        }
    });
}
// endregion

// region: SMatrix tests (statically-sized matrices)

#[cfg(feature = "nalgebra")]
#[test]
fn smatrix_f64_roundtrip() {
    r_test_utils::with_r_thread(|| {
        // 2x3 matrix
        let mat: SMatrix<f64, 2, 3> = SMatrix::from_column_slice(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let sexp = mat.into_sexp();

        let back: SMatrix<f64, 2, 3> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(back.nrows(), 2);
        assert_eq!(back.ncols(), 3);
        // Column-major: [[1, 3, 5], [2, 4, 6]]
        assert!((back[(0, 0)] - 1.0).abs() < 1e-10);
        assert!((back[(1, 0)] - 2.0).abs() < 1e-10);
        assert!((back[(0, 1)] - 3.0).abs() < 1e-10);
        assert!((back[(1, 1)] - 4.0).abs() < 1e-10);
        assert!((back[(0, 2)] - 5.0).abs() < 1e-10);
        assert!((back[(1, 2)] - 6.0).abs() < 1e-10);
    });
}

#[cfg(feature = "nalgebra")]
#[test]
fn smatrix_i32_from_r() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{Rf_allocMatrix, Rf_protect, Rf_unprotect, SEXPTYPE, SexpExt};

        unsafe {
            // Create 2x2 integer matrix
            let sexp = Rf_protect(Rf_allocMatrix(SEXPTYPE::INTSXP, 2, 2));
            let slice: &mut [i32] = sexp.as_mut_slice();
            slice[0] = 1;
            slice[1] = 2;
            slice[2] = 3;
            slice[3] = 4;

            let mat: SMatrix<i32, 2, 2> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(mat.nrows(), 2);
            assert_eq!(mat.ncols(), 2);
            // Column-major: [[1, 3], [2, 4]]
            assert_eq!(mat[(0, 0)], 1);
            assert_eq!(mat[(1, 0)], 2);
            assert_eq!(mat[(0, 1)], 3);
            assert_eq!(mat[(1, 1)], 4);

            Rf_unprotect(1);
        }
    });
}

#[cfg(feature = "nalgebra")]
#[test]
fn smatrix_dimension_mismatch() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{Rf_allocMatrix, Rf_protect, Rf_unprotect, SEXPTYPE, SexpExt};

        unsafe {
            // Create 3x3 matrix, but try to convert to SMatrix<f64, 2, 2>
            let sexp = Rf_protect(Rf_allocMatrix(SEXPTYPE::REALSXP, 3, 3));
            let slice: &mut [f64] = sexp.as_mut_slice();
            for (i, slot) in slice.iter_mut().enumerate() {
                *slot = i as f64;
            }

            let result: Result<SMatrix<f64, 2, 2>, _> = TryFromSexp::try_from_sexp(sexp);
            assert!(result.is_err(), "Should fail: expected 2x2, got 3x3");

            Rf_unprotect(1);
        }
    });
}

#[cfg(feature = "nalgebra")]
#[test]
fn smatrix_4x4_transformation() {
    // Common use case: 4x4 transformation matrix for 3D graphics
    r_test_utils::with_r_thread(|| {
        // Identity matrix
        let mat: SMatrix<f64, 4, 4> = SMatrix::from_column_slice(&[
            1.0, 0.0, 0.0, 0.0, // col 1
            0.0, 1.0, 0.0, 0.0, // col 2
            0.0, 0.0, 1.0, 0.0, // col 3
            0.0, 0.0, 0.0, 1.0, // col 4
        ]);

        let sexp = mat.into_sexp();
        let back: SMatrix<f64, 4, 4> = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back.nrows(), 4);
        assert_eq!(back.ncols(), 4);
        // Check identity
        for i in 0..4 {
            for j in 0..4 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (back[(i, j)] - expected).abs() < 1e-10,
                    "Expected {} at ({}, {}), got {}",
                    expected,
                    i,
                    j,
                    back[(i, j)]
                );
            }
        }
    });
}

#[cfg(feature = "nalgebra")]
#[test]
fn option_smatrix_some() {
    r_test_utils::with_r_thread(|| {
        let mat: Option<SMatrix<f64, 2, 2>> =
            Some(SMatrix::from_column_slice(&[1.0, 2.0, 3.0, 4.0]));
        let sexp = mat.into_sexp();

        let back: Option<SMatrix<f64, 2, 2>> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(back.is_some());
        let m = back.unwrap();
        assert_eq!(m[(0, 0)], 1.0);
    });
}

#[cfg(feature = "nalgebra")]
#[test]
fn option_smatrix_none() {
    r_test_utils::with_r_thread(|| {
        let mat: Option<SMatrix<f64, 2, 2>> = None;
        let sexp = mat.into_sexp();

        let back: Option<SMatrix<f64, 2, 2>> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert!(back.is_none());
    });
}
// endregion
