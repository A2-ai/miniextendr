//! Tests for batched diagnostics on the numeric-coercion vector conversions
//! (`from_numeric_vec_with` / `coerce_slice_to_vec` and the `Vec<T>` /
//! `Vec<Option<T>>` / `HashSet<T>` / `BTreeSet<T>` impls that route through
//! them).
//!
//! Extends the #1143 string-parse batching to the numeric-coercion paths (#1192):
//! instead of bailing on the first failing element, the shells walk the whole
//! vector and return one batched `SexpError::InvalidValue` via `BatchedErrors`
//! (capped at 10 + "and N more"). Each distinct reason is listed once with the
//! 1-based positions of the elements that failed with it, as R numbers them:
//! `value out of range (elements 1, 3)`. The per-element coercion error is
//! formatted with `Display` (e.g. "value out of range"), not `Debug`, and the
//! Rust type is not in the message (an argument error carries it as
//! `e$rust_type`, #1591).

mod r_test_utils;

use std::collections::{BTreeSet, HashSet};

use miniextendr_api::from_r::{SexpError, TryFromSexp};
use miniextendr_api::into_r::IntoR;

/// The batched message behind an `InvalidValue`, without `Display`'s
/// `invalid value: ` prefix.
fn batched_message(err: SexpError) -> String {
    match err {
        SexpError::InvalidValue(msg) => msg,
        other => panic!("expected a batched InvalidValue, got {other:?}"),
    }
}

/// `Vec<u32>` fed `c(-1, 5, -3)` reports both failing elements (1 and 3) in one
/// batched message, not just the first.
#[test]
fn vec_coerce_batches_all_failing_indices() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec![-1i32, 5, -3].into_sexp();
        let err = <Vec<u32> as TryFromSexp>::try_from_sexp(sexp).unwrap_err();
        let msg = batched_message(err);
        // Element 2 (the valid `5`) is not listed; Display, not Debug:
        // "value out of range", never "Overflow".
        assert_eq!(msg, "value out of range (elements 1, 3)");
    });
}

/// The all-valid happy path succeeds (no false batching).
#[test]
fn vec_coerce_happy_path_ok() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec![1i32, 5, 3].into_sexp();
        let out = <Vec<u32> as TryFromSexp>::try_from_sexp(sexp).unwrap();
        assert_eq!(out, vec![1u32, 5, 3]);
    });
}

/// More than 10 failures are capped: the first 10 positions are listed and the
/// remainder is summarized as "and N more".
#[test]
fn vec_coerce_batch_caps_at_ten_and_summarizes_rest() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec![-1i32; 15].into_sexp();
        let err = <Vec<u32> as TryFromSexp>::try_from_sexp(sexp).unwrap_err();
        let msg = batched_message(err);
        assert_eq!(
            msg,
            "value out of range (elements 1, 2, 3, 4, 5, 6, 7, 8, 9, 10); and 5 more"
        );
    });
}

/// `Vec<u32>` refuses an `NA` element like any other element that does not
/// convert, in the same batched message.
#[test]
fn vec_coerce_na_is_one_of_the_failures() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec![Some(-1i32), None, Some(5i32)].into_sexp();
        let err = <Vec<u32> as TryFromSexp>::try_from_sexp(sexp).unwrap_err();
        let msg = batched_message(err);
        assert_eq!(
            msg,
            "value out of range (element 1); NA is not allowed (element 2)"
        );
    });
}

/// `Vec<Option<u32>>`: `NA` maps to `None` (passthrough) while real coercion
/// failures batch with their original position.
#[test]
fn option_vec_na_passes_through_while_failures_batch() {
    r_test_utils::with_r_thread(|| {
        // c(NA, -1, 5): element 1 is NA (-> None), element 2 fails, element 3 is fine.
        let sexp = vec![None, Some(-1i32), Some(5i32)].into_sexp();
        let err = <Vec<Option<u32>> as TryFromSexp>::try_from_sexp(sexp).unwrap_err();
        let msg = batched_message(err);
        assert_eq!(msg, "value out of range (element 2)");
    });
}

/// `Vec<Option<u32>>` with only NA + valid elements succeeds, preserving `None`.
#[test]
fn option_vec_na_and_valid_only_is_ok() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec![Some(1i32), None, Some(5i32)].into_sexp();
        let out = <Vec<Option<u32>> as TryFromSexp>::try_from_sexp(sexp).unwrap();
        assert_eq!(out, vec![Some(1u32), None, Some(5u32)]);
    });
}

/// `HashSet<u32>` inherits the batching from `try_from_sexp_numeric_vec`.
#[test]
fn hashset_inherits_batching() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec![-1i32, 5, -3].into_sexp();
        let err = <HashSet<u32> as TryFromSexp>::try_from_sexp(sexp).unwrap_err();
        let msg = batched_message(err);
        assert_eq!(msg, "value out of range (elements 1, 3)");
    });
}

/// `BTreeSet<i8>` inherits the batching too.
#[test]
fn btreeset_inherits_batching() {
    r_test_utils::with_r_thread(|| {
        // 300 overflows i8; -200 underflows i8; 5 is fine.
        let sexp = vec![300i32, 5, -200].into_sexp();
        let err = <BTreeSet<i8> as TryFromSexp>::try_from_sexp(sexp).unwrap_err();
        let msg = batched_message(err);
        assert_eq!(msg, "value out of range (elements 1, 3)");
    });
}

/// `Vec<bool>` batches via `coerce_slice_to_vec`: NA logicals fail per-element
/// with the `LogicalCoerceError` Display text.
#[test]
fn vec_bool_batches_via_coerce_slice() {
    r_test_utils::with_r_thread(|| {
        // c(NA, TRUE, NA): elements 1 and 3 are NA logical, which bool rejects.
        let sexp = vec![None::<bool>, Some(true), None::<bool>].into_sexp();
        let err = <Vec<bool> as TryFromSexp>::try_from_sexp(sexp).unwrap_err();
        let msg = batched_message(err);
        assert_eq!(msg, "NA is not allowed (elements 1, 3)");
    });
}

/// A non-numeric SEXPTYPE is a type error (worded in R terms by the argument
/// error), not an `InvalidValue` quoting a SEXPTYPE name.
#[test]
fn vec_coerce_refuses_a_character_vector_with_a_type_error() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec!["a".to_string()].into_sexp();
        let err = <Vec<u32> as TryFromSexp>::try_from_sexp(sexp).unwrap_err();
        assert!(matches!(err, SexpError::Type(_)), "{err:?}");
    });
}
