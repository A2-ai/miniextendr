//! Round-trip tests for the string ndarray conversions (issue #1348):
//! `Array<String, D>` / `Array<Option<String>, D>` <-> R character arrays.
//!
//! Covers matrices, a higher-dimensional shape, empty dimensions, UTF-8
//! content, NA handling, and layout independence (standard vs Fortran
//! memory order must produce the same column-major R storage).

mod r_test_utils;

#[cfg(feature = "ndarray")]
use miniextendr_api::from_r::TryFromSexp;
#[cfg(feature = "ndarray")]
use miniextendr_api::into_r::IntoR;
#[cfg(feature = "ndarray")]
use miniextendr_api::{Array0, Array1, Array2, Array3, ArrayD, IxDyn, ShapeBuilder};

/// Helper: owned strings from string literals.
#[cfg(feature = "ndarray")]
fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| (*x).to_string()).collect()
}

#[cfg(feature = "ndarray")]
#[test]
fn string_matrix_round_trip_column_major() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::prelude::SexpExt;
        use miniextendr_api::sys::{Rf_protect, Rf_unprotect};

        // Standard (row-major) layout 2x3 matrix:
        //   a c e
        //   b d f
        let arr = Array2::from_shape_vec((2, 3), s(&["a", "c", "e", "b", "d", "f"])).unwrap();
        assert!(arr.is_standard_layout());

        unsafe {
            let sexp = Rf_protect(arr.clone().into_sexp());

            // R stores column-major: a b c d e f
            let stored: Vec<String> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(stored, s(&["a", "b", "c", "d", "e", "f"]));

            // dim attribute is c(2L, 3L)
            let dim = sexp.get_dim();
            let dim_slice: &[i32] = dim.as_slice();
            assert_eq!(dim_slice, &[2, 3]);

            // Round-trip back to the same logical matrix
            let back: Array2<String> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(back, arr);

            Rf_unprotect(1);
        }
    });
}

#[cfg(feature = "ndarray")]
#[test]
fn string_matrix_layout_independence() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::sys::{Rf_protect, Rf_unprotect};

        // The same logical matrix in standard (C) and Fortran memory layout
        // must produce identical R storage.
        let c_order = Array2::from_shape_vec((2, 3), s(&["a", "c", "e", "b", "d", "f"])).unwrap();
        let f_order =
            Array2::from_shape_vec((2, 3).f(), s(&["a", "b", "c", "d", "e", "f"])).unwrap();
        assert_eq!(c_order, f_order);
        assert!(c_order.is_standard_layout());
        assert!(!f_order.is_standard_layout());

        unsafe {
            let sexp_c = Rf_protect(c_order.into_sexp());
            let sexp_f = Rf_protect(f_order.into_sexp());

            let stored_c: Vec<String> = TryFromSexp::try_from_sexp(sexp_c).unwrap();
            let stored_f: Vec<String> = TryFromSexp::try_from_sexp(sexp_f).unwrap();
            assert_eq!(stored_c, s(&["a", "b", "c", "d", "e", "f"]));
            assert_eq!(stored_c, stored_f);

            Rf_unprotect(2);
        }
    });
}

#[cfg(feature = "ndarray")]
#[test]
fn option_string_matrix_na_round_trip() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::SEXP;
        use miniextendr_api::prelude::SexpExt;
        use miniextendr_api::sys::{Rf_protect, Rf_unprotect};

        let arr = Array2::from_shape_vec(
            (2, 2),
            vec![
                Some("a".to_string()),
                None,
                Some("d".to_string()),
                Some("".to_string()),
            ],
        )
        .unwrap();

        unsafe {
            let sexp = Rf_protect(arr.clone().into_sexp());

            // Column-major storage: [0,0]="a", [1,0]="d", [0,1]=NA, [1,1]=""
            assert_eq!(sexp.string_elt(2), SEXP::na_string());
            let stored: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(
                stored,
                vec![
                    Some("a".to_string()),
                    Some("d".to_string()),
                    None,
                    Some("".to_string()),
                ]
            );

            let back: Array2<Option<String>> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(back, arr);

            Rf_unprotect(1);
        }
    });
}

#[cfg(feature = "ndarray")]
#[test]
fn string_matrix_na_is_lossy_without_option() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::sys::{Rf_protect, Rf_unprotect};

        // An R character vector with NA read as Array1<String>: NA -> ""
        // (mirrors the documented Vec<String> contract).
        let with_na: Vec<Option<String>> = vec![Some("x".to_string()), None];

        unsafe {
            let sexp = Rf_protect(with_na.into_sexp());
            let arr: Array1<String> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(arr[0], "x");
            assert_eq!(arr[1], "");
            Rf_unprotect(1);
        }
    });
}

#[cfg(feature = "ndarray")]
#[test]
fn string_array3_utf8_round_trip() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::sys::{Rf_protect, Rf_unprotect};

        // Higher-dimensional shape (2x3x2) with UTF-8 (non-ASCII) content.
        let data: Vec<Option<String>> = (0..12)
            .map(|i| {
                if i % 5 == 0 {
                    None
                } else {
                    Some(format!("héllø-{i}-日本語"))
                }
            })
            .collect();
        let arr = Array3::from_shape_vec((2, 3, 2), data).unwrap();

        unsafe {
            let sexp = Rf_protect(arr.clone().into_sexp());

            // Column-major spot checks: linear index i + 2*j + 6*k.
            let stored: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(stored.len(), 12);
            assert_eq!(stored[1], arr[[1, 0, 0]]);
            assert_eq!(stored[2], arr[[0, 1, 0]]);
            assert_eq!(stored[7], arr[[1, 0, 1]]);

            let back: Array3<Option<String>> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(back, arr);

            Rf_unprotect(1);
        }
    });
}

#[cfg(feature = "ndarray")]
#[test]
fn string_arrayd_round_trip() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::prelude::SexpExt;
        use miniextendr_api::sys::{Rf_protect, Rf_unprotect};

        let data: Vec<String> = (0..16).map(|i| format!("d-{i}")).collect();
        let arr = ArrayD::from_shape_vec(IxDyn(&[2, 2, 2, 2]), data).unwrap();

        unsafe {
            let sexp = Rf_protect(arr.clone().into_sexp());

            let dim = sexp.get_dim();
            let dim_slice: &[i32] = dim.as_slice();
            assert_eq!(dim_slice, &[2, 2, 2, 2]);

            let back: ArrayD<String> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(back, arr);

            Rf_unprotect(1);
        }
    });
}

#[cfg(feature = "ndarray")]
#[test]
fn string_matrix_empty_dims() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::prelude::SexpExt;
        use miniextendr_api::sys::{Rf_protect, Rf_unprotect};

        // 0x3 matrix: zero elements, but the dim attribute must survive.
        let arr = Array2::<Option<String>>::from_shape_vec((0, 3), vec![]).unwrap();

        unsafe {
            let sexp = Rf_protect(arr.clone().into_sexp());

            assert_eq!(sexp.len(), 0);
            let dim = sexp.get_dim();
            let dim_slice: &[i32] = dim.as_slice();
            assert_eq!(dim_slice, &[0, 3]);

            let back: Array2<Option<String>> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(back.dim(), (0, 3));

            Rf_unprotect(1);
        }

        // Empty 1-D array: plain character(0), no dim.
        let empty = Array1::<String>::from_vec(vec![]);
        unsafe {
            let sexp = Rf_protect(empty.into_sexp());
            assert_eq!(sexp.len(), 0);
            assert!(sexp.get_dim().is_null_or_nil());
            let back: Array1<String> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(back.len(), 0);
            Rf_unprotect(1);
        }
    });
}

#[cfg(feature = "ndarray")]
#[test]
fn string_scalar_and_vector_shapes() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::prelude::SexpExt;
        use miniextendr_api::sys::{Rf_protect, Rf_unprotect};

        // Array0 <-> length-1 character vector without dim.
        let scalar = Array0::from_elem((), "solo".to_string());
        unsafe {
            let sexp = Rf_protect(scalar.clone().into_sexp());
            assert_eq!(sexp.len(), 1);
            assert!(sexp.get_dim().is_null_or_nil());
            let back: Array0<String> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(back, scalar);
            Rf_unprotect(1);
        }

        // Array1 -> plain character vector without dim.
        let vec1 = Array1::from_vec(s(&["p", "q", "r"]));
        unsafe {
            let sexp = Rf_protect(vec1.clone().into_sexp());
            assert!(sexp.get_dim().is_null_or_nil());

            // A plain vector reads back as an n x 1 column via Array2
            // (same contract as the numeric conversions).
            let col: Array2<String> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(col.dim(), (3, 1));
            assert_eq!(col[[1, 0]], "q");

            let back: Array1<String> = TryFromSexp::try_from_sexp(sexp).unwrap();
            assert_eq!(back, vec1);
            Rf_unprotect(1);
        }
    });
}

#[cfg(feature = "ndarray")]
#[test]
fn string_array_dim_mismatch_errors() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::sys::{Rf_protect, Rf_unprotect};

        // A 2-D character matrix must not parse as Array3<String>.
        let arr = Array2::from_shape_vec((2, 2), s(&["a", "b", "c", "d"])).unwrap();
        unsafe {
            let sexp = Rf_protect(arr.into_sexp());
            let res: Result<Array3<String>, _> = TryFromSexp::try_from_sexp(sexp);
            assert!(res.is_err());
            // A non-STRSXP input must fail the element walk.
            let ints = Rf_protect(vec![1i32, 2].into_sexp());
            let res: Result<Array1<String>, _> = TryFromSexp::try_from_sexp(ints);
            assert!(res.is_err());
            Rf_unprotect(2);
        }
    });
}
