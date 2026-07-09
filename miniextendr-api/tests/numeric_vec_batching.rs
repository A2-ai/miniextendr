//! Tests for batched diagnostics on the numeric-coercion vector conversions
//! (`from_numeric_vec_with` / `coerce_slice_to_vec` and the `Vec<T>` /
//! `Vec<Option<T>>` / `HashSet<T>` / `BTreeSet<T>` impls that route through
//! them).
//!
//! Extends the #1143 string-parse batching to the numeric-coercion paths (#1192):
//! instead of bailing on the first failing element, the shells walk the whole
//! vector, accumulate per-element failures as `"invalid value at index <i>: <err>"`,
//! and return one batched `SexpError::InvalidValue` via `batch_conversion_errors`
//! (capped at 10 + "and N more"). The per-element coercion error is formatted with
//! `Display` (e.g. "value out of range"), not `Debug`.

mod r_test_utils;

use std::collections::{BTreeSet, HashSet};

use miniextendr_api::from_r::TryFromSexp;
use miniextendr_api::into_r::IntoR;

/// `Vec<u32>` fed `c(-1, 5, -3)` reports both failing indices (0 and 2) in one
/// batched message, not just the first.
#[test]
fn vec_coerce_batches_all_failing_indices() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec![-1i32, 5, -3].into_sexp();
        let err = <Vec<u32> as TryFromSexp>::try_from_sexp(sexp).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Vec<u32> conversion failed"), "got: {msg}");
        assert!(msg.contains("invalid value at index 0"), "got: {msg}");
        assert!(msg.contains("invalid value at index 2"), "got: {msg}");
        // index 1 (the valid `5`) must not appear.
        assert!(!msg.contains("at index 1"), "got: {msg}");
        // Display, not Debug: "value out of range", never "Overflow".
        assert!(msg.contains("value out of range"), "got: {msg}");
        assert!(!msg.contains("Overflow"), "got: {msg}");
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

/// More than 10 failures are capped: the first 10 indices are listed and the
/// remainder is summarized as "and N more".
#[test]
fn vec_coerce_batch_caps_at_ten_and_summarizes_rest() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec![-1i32; 15].into_sexp();
        let err = <Vec<u32> as TryFromSexp>::try_from_sexp(sexp).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Vec<u32> conversion failed"), "got: {msg}");
        assert!(msg.contains("invalid value at index 0"), "got: {msg}");
        assert!(msg.contains("invalid value at index 9"), "got: {msg}");
        assert!(!msg.contains("at index 10"), "got: {msg}");
        assert!(msg.contains("and 5 more"), "got: {msg}");
    });
}

/// `Vec<Option<u32>>`: `NA` maps to `None` (passthrough) while real coercion
/// failures batch with their original index.
#[test]
fn option_vec_na_passes_through_while_failures_batch() {
    r_test_utils::with_r_thread(|| {
        // c(NA, -1, 5): index 0 is NA (-> None), index 1 fails, index 2 is fine.
        let sexp = vec![None, Some(-1i32), Some(5i32)].into_sexp();
        let err = <Vec<Option<u32>> as TryFromSexp>::try_from_sexp(sexp).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("Vec<Option<u32>> conversion failed"),
            "got: {msg}"
        );
        assert!(msg.contains("invalid value at index 1"), "got: {msg}");
        // NA (index 0) is not a failure; index 2 is valid.
        assert!(!msg.contains("at index 0"), "got: {msg}");
        assert!(!msg.contains("at index 2"), "got: {msg}");
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

/// `HashSet<u32>` inherits the batching from `try_from_sexp_numeric_vec`; the
/// container label is the set type.
#[test]
fn hashset_inherits_batching() {
    r_test_utils::with_r_thread(|| {
        let sexp = vec![-1i32, 5, -3].into_sexp();
        let err = <HashSet<u32> as TryFromSexp>::try_from_sexp(sexp).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("HashSet<u32> conversion failed"), "got: {msg}");
        assert!(msg.contains("invalid value at index 0"), "got: {msg}");
        assert!(msg.contains("invalid value at index 2"), "got: {msg}");
    });
}

/// `BTreeSet<i8>` inherits the batching too (distinct container label).
#[test]
fn btreeset_inherits_batching() {
    r_test_utils::with_r_thread(|| {
        // 300 overflows i8; -200 underflows i8; 5 is fine.
        let sexp = vec![300i32, 5, -200].into_sexp();
        let err = <BTreeSet<i8> as TryFromSexp>::try_from_sexp(sexp).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("BTreeSet<i8> conversion failed"), "got: {msg}");
        assert!(msg.contains("invalid value at index 0"), "got: {msg}");
        assert!(msg.contains("invalid value at index 2"), "got: {msg}");
        assert!(!msg.contains("at index 1"), "got: {msg}");
    });
}

/// `Vec<bool>` batches via `coerce_slice_to_vec`: NA logicals fail per-element
/// with the `LogicalCoerceError` Display text, batched under the `Vec<bool>`
/// label.
#[test]
fn vec_bool_batches_via_coerce_slice() {
    r_test_utils::with_r_thread(|| {
        // c(NA, TRUE, NA): indices 0 and 2 are NA logical, which bool rejects.
        let sexp = vec![None::<bool>, Some(true), None::<bool>].into_sexp();
        let err = <Vec<bool> as TryFromSexp>::try_from_sexp(sexp).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Vec<bool> conversion failed"), "got: {msg}");
        assert!(msg.contains("invalid value at index 0"), "got: {msg}");
        assert!(msg.contains("invalid value at index 2"), "got: {msg}");
        assert!(msg.contains("NA cannot be converted to bool"), "got: {msg}");
    });
}
