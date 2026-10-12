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
        use miniextendr_api::SEXPTYPE;
        use miniextendr_api::prelude::SexpExt;
        use miniextendr_api::sys::{Rf_allocVector, Rf_protect, Rf_unprotect};

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

#[test]
fn arrays_follow_slice_storage_for_non_native_elements() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::{OwnedProtect, SEXPTYPE, SexpExt};

        fn compare<T: Clone, const N: usize>(array: [T; N], kind: SEXPTYPE)
        where
            [T; N]: IntoR,
            <[T; N] as IntoR>::Error: std::fmt::Debug,
            for<'a> &'a [T]: IntoR,
        {
            let expected = unsafe { OwnedProtect::new(array.as_slice().into_sexp()) };
            let actual = unsafe { OwnedProtect::new(array.clone().into_sexp()) };
            assert_eq!(actual.get().type_of(), kind);
            assert!(actual.get().is_identical(expected.get()));
            let unchecked = unsafe { OwnedProtect::new(array.try_into_sexp_unchecked().unwrap()) };
            assert!(unchecked.get().is_identical(expected.get()));
        }

        macro_rules! compare {
            ($array:expr, $kind:ident) => {
                compare($array, SEXPTYPE::$kind)
            };
        }

        compare!([1_i64, 2], INTSXP);
        compare!([1_i64, 3_000_000_000], REALSXP);
        compare!([1_u64, 3_000_000_000], REALSXP);
        compare!([1_u32, 3_000_000_000], REALSXP);
        compare!([1_usize, 3_000_000_000], REALSXP);
        compare!([isize::MIN, 1], REALSXP);
        compare!([1_i8, 2], INTSXP);
        compare!([1_i16, 2], INTSXP);
        compare!([1_u16, 2], INTSXP);
        compare!([1.5_f32, 2.5], REALSXP);
        compare!([true, false], LGLSXP);
        compare!(
            [
                miniextendr_api::Rboolean::TRUE,
                miniextendr_api::Rboolean::FALSE
            ],
            LGLSXP
        );
        compare!([String::from("one"), String::from("two")], STRSXP);
        // Element references need not be 'static.
        let text = String::from("borrowed");
        compare!([text.as_str(), ""], STRSXP);
        compare!([] as [String; 0], STRSXP);
        compare!([] as [bool; 0], LGLSXP);
        compare!([] as [i64; 0], INTSXP);
    });
}

#[test]
fn optional_arrays_use_null_for_absence_and_keep_empty_vectors() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::{OwnedProtect, SEXPTYPE, SexpExt};
        assert!(None::<[String; 2]>.into_sexp().is_nil());
        assert!(
            unsafe { None::<[i64; 2]>.try_into_sexp_unchecked() }
                .unwrap()
                .is_nil()
        );
        let empty = unsafe { OwnedProtect::new(Some([] as [bool; 0]).into_sexp()) };
        assert_eq!(empty.get().type_of(), SEXPTYPE::LGLSXP);
        assert_eq!(empty.get().len(), 0);
        let some = Some([String::from("one"), String::from("two")]);
        let value = unsafe { OwnedProtect::new(some.into_sexp()) };
        assert_eq!(
            Vec::<String>::try_from_sexp(value.get()).unwrap(),
            ["one", "two"]
        );
    });
}
