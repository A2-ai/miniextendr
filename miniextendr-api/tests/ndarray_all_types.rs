mod r_test_utils;

#[cfg(feature = "ndarray")]
use miniextendr_api::from_r::TryFromSexp;
#[cfg(feature = "ndarray")]
use miniextendr_api::into_r::IntoR;
#[cfg(feature = "ndarray")]
use miniextendr_api::{Array0, Array1, Array2};

#[cfg(feature = "ndarray")]
#[test]
fn array1_all_rnative_types() {
    // Verify Array1 blanket impl works for all RNativeType: i32, f64, u8, RLogical
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{
            RLogical, Rf_allocVector, Rf_protect, Rf_unprotect, SEXPTYPE, SexpExt,
        };

        unsafe {
            // i32
            let sexp_int = Rf_protect(Rf_allocVector(SEXPTYPE::INTSXP, 3));
            sexp_int.set_integer_elt(0, 1);
            sexp_int.set_integer_elt(1, 2);
            sexp_int.set_integer_elt(2, 3);
            let arr_i32: Array1<i32> = TryFromSexp::try_from_sexp(sexp_int).unwrap();
            assert_eq!(arr_i32[0], 1);
            assert_eq!(arr_i32[1], 2);
            assert_eq!(arr_i32[2], 3);

            // f64
            let sexp_real = Rf_protect(Rf_allocVector(SEXPTYPE::REALSXP, 3));
            sexp_real.set_real_elt(0, 1.5);
            sexp_real.set_real_elt(1, 2.5);
            sexp_real.set_real_elt(2, 3.5);
            let arr_f64: Array1<f64> = TryFromSexp::try_from_sexp(sexp_real).unwrap();
            assert_eq!(arr_f64[0], 1.5);
            assert_eq!(arr_f64[1], 2.5);
            assert_eq!(arr_f64[2], 3.5);

            // u8
            let sexp_raw = Rf_protect(Rf_allocVector(SEXPTYPE::RAWSXP, 3));
            sexp_raw.set_raw_elt(0, 10);
            sexp_raw.set_raw_elt(1, 20);
            sexp_raw.set_raw_elt(2, 30);
            let arr_u8: Array1<u8> = TryFromSexp::try_from_sexp(sexp_raw).unwrap();
            assert_eq!(arr_u8[0], 10);
            assert_eq!(arr_u8[1], 20);
            assert_eq!(arr_u8[2], 30);

            // RLogical
            let sexp_lgl = Rf_protect(Rf_allocVector(SEXPTYPE::LGLSXP, 3));
            sexp_lgl.set_logical_elt(0, 1); // TRUE
            sexp_lgl.set_logical_elt(1, 0); // FALSE
            sexp_lgl.set_logical_elt(2, 1); // TRUE
            let arr_lgl: Array1<RLogical> = TryFromSexp::try_from_sexp(sexp_lgl).unwrap();
            assert_eq!(arr_lgl[0], RLogical::from(true));
            assert_eq!(arr_lgl[1], RLogical::from(false));
            assert_eq!(arr_lgl[2], RLogical::from(true));

            Rf_unprotect(4);
        }
    });
}

#[cfg(feature = "ndarray")]
#[test]
fn array0_scalar_all_types() {
    // Verify Array0 (scalar) works for all types
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::RLogical;
        use miniextendr_api::ffi::SEXP;

        // i32 scalar
        let sexp_int = SEXP::scalar_integer(42);
        let arr: Array0<i32> = TryFromSexp::try_from_sexp(sexp_int).unwrap();
        assert_eq!(arr[()], 42);

        // f64 scalar
        let sexp_real = SEXP::scalar_real(3.125);
        let arr: Array0<f64> = TryFromSexp::try_from_sexp(sexp_real).unwrap();
        assert_eq!(arr[()], 3.125);

        // RLogical scalar
        let sexp_lgl = SEXP::scalar_logical(true);
        let arr: Array0<RLogical> = TryFromSexp::try_from_sexp(sexp_lgl).unwrap();
        assert_eq!(arr[()], RLogical::from(true));
    });
}

#[cfg(feature = "ndarray")]
#[test]
fn array2_u8_blanket_impl() {
    // Verify Array2 works for u8 (raw matrices)
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{Rf_allocMatrix, Rf_protect, Rf_unprotect, SEXPTYPE, SexpExt};

        unsafe {
            // Create 2x2 raw matrix
            let sexp = Rf_protect(Rf_allocMatrix(SEXPTYPE::RAWSXP, 2, 2));
            sexp.set_raw_elt(0, 1);
            sexp.set_raw_elt(1, 2);
            sexp.set_raw_elt(2, 3);
            sexp.set_raw_elt(3, 4);

            let arr: Array2<u8> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(arr.nrows(), 2);
            assert_eq!(arr.ncols(), 2);
            assert_eq!(arr[[0, 0]], 1);
            assert_eq!(arr[[1, 0]], 2);
            assert_eq!(arr[[0, 1]], 3);
            assert_eq!(arr[[1, 1]], 4);

            Rf_unprotect(1);
        }
    });
}

#[cfg(feature = "ndarray")]
#[test]
fn arrayd_i32_from_vector() {
    // Test ArrayD created from R vector (1D)
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{Rf_allocVector, Rf_protect, Rf_unprotect, SEXPTYPE, SexpExt};
        use ndarray::ArrayD;

        unsafe {
            let sexp = Rf_protect(Rf_allocVector(SEXPTYPE::INTSXP, 5));
            for i in 0..5 {
                sexp.set_integer_elt(i as isize, i as i32);
            }

            let arr: ArrayD<i32> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(arr.ndim(), 1);
            assert_eq!(arr.shape(), &[5]);
            assert_eq!(arr[[0]], 0);
            assert_eq!(arr[[1]], 1);
            assert_eq!(arr[[4]], 4);

            Rf_unprotect(1);
        }
    });
}

#[cfg(feature = "ndarray")]
#[test]
fn array1_empty() {
    r_test_utils::with_r_thread(|| {
        use ndarray::Array1;

        let arr: Array1<i32> = Array1::from_vec(vec![]);
        let sexp = arr.into_sexp();
        let back: Array1<i32> = TryFromSexp::try_from_sexp(sexp).unwrap();
        assert_eq!(back.len(), 0);
    });
}
