mod r_test_utils;

use miniextendr_api::from_r::TryFromSexp;
use miniextendr_api::into_r::IntoR;

#[test]
fn fixed_array_u8_32_roundtrip() {
    // Test [u8; 32] for SHA256-like use cases
    r_test_utils::with_r_thread(|| {
        let arr: [u8; 32] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];

        let sexp = arr.into_sexp();
        let back: [u8; 32] = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back, arr);
    });
}

#[test]
fn fixed_array_i32_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let arr: [i32; 5] = [10, 20, 30, 40, 50];

        let sexp = arr.into_sexp();
        let back: [i32; 5] = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back, arr);
    });
}

#[test]
fn fixed_array_f64_roundtrip() {
    r_test_utils::with_r_thread(|| {
        let arr: [f64; 3] = [1.5, 2.5, 3.5];

        let sexp = arr.into_sexp();
        let back: [f64; 3] = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back, arr);
    });
}

#[test]
fn fixed_array_length_mismatch_error() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::ffi::{Rf_allocVector, Rf_protect, Rf_unprotect, SEXPTYPE, SexpExt};

        unsafe {
            // Create R vector with 5 elements
            let sexp = Rf_protect(Rf_allocVector(SEXPTYPE::INTSXP, 5));
            let slice: &mut [i32] = sexp.as_mut_slice();
            for (i, slot) in slice.iter_mut().enumerate() {
                *slot = i as i32;
            }

            // Try to convert to [i32; 3] (wrong size)
            let result: Result<[i32; 3], _> = TryFromSexp::try_from_sexp(sexp);

            // Should fail with length error
            assert!(result.is_err());
            let err = result.unwrap_err();
            let msg = format!("{}", err);
            assert!(msg.contains("expected") || msg.contains("length"));

            Rf_unprotect(1);
        }
    });
}

#[test]
fn fixed_array_empty() {
    r_test_utils::with_r_thread(|| {
        let arr: [i32; 0] = [];

        let sexp = arr.into_sexp();
        let back: [i32; 0] = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back, arr);
    });
}

#[test]
fn fixed_array_single_element() {
    r_test_utils::with_r_thread(|| {
        let arr: [f64; 1] = [42.0];

        let sexp = arr.into_sexp();
        let back: [f64; 1] = TryFromSexp::try_from_sexp(sexp).unwrap();

        assert_eq!(back, arr);
    });
}
