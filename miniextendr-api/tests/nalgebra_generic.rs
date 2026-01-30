mod r_test_utils;

#[cfg(feature = "nalgebra")]
use miniextendr_api::from_r::TryFromSexp;
#[cfg(feature = "nalgebra")]
use miniextendr_api::into_r::IntoR;
#[cfg(feature = "nalgebra")]
use miniextendr_api::{DMatrix, DVector};

#[cfg(feature = "nalgebra")]
#[test]
fn dvector_i32_blanket_impl() {
    // Verify blanket impl works for i32 (not just f64)
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{Rf_allocVector, Rf_protect, Rf_unprotect, INTEGER, SEXPTYPE};

        unsafe {
            let sexp = Rf_protect(Rf_allocVector(SEXPTYPE::INTSXP, 3));
            let ptr = INTEGER(sexp);
            *ptr.add(0) = 10;
            *ptr.add(1) = 20;
            *ptr.add(2) = 30;

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
        use miniextendr_api::ffi::{Rf_allocMatrix, Rf_protect, Rf_unprotect, INTEGER, SEXPTYPE};

        unsafe {
            // Create 2x3 matrix
            let sexp = Rf_protect(Rf_allocMatrix(SEXPTYPE::INTSXP, 2, 3));
            let ptr = INTEGER(sexp);
            // Column-major: [col1, col2, col3]
            *ptr.add(0) = 1;  // row 1, col 1
            *ptr.add(1) = 2;  // row 2, col 1
            *ptr.add(2) = 3;  // row 1, col 2
            *ptr.add(3) = 4;  // row 2, col 2
            *ptr.add(4) = 5;  // row 1, col 3
            *ptr.add(5) = 6;  // row 2, col 3

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
        use miniextendr_api::ffi::{Rf_allocMatrix, Rf_protect, Rf_unprotect, RAW, SEXPTYPE};

        unsafe {
            // Create 2x2 raw matrix
            let sexp = Rf_protect(Rf_allocMatrix(SEXPTYPE::RAWSXP, 2, 2));
            let ptr = RAW(sexp);
            *ptr.add(0) = 10;
            *ptr.add(1) = 20;
            *ptr.add(2) = 30;
            *ptr.add(3) = 40;

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
