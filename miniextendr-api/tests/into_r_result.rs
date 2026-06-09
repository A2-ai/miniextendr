//! Integration tests for `Result<T, E>` → R conversions (`into_r/result.rs`).
//!
//! Covers the two deterministic, allocation-driven paths:
//!
//! - `Result<T, NullOnErr>` (the `Result<T, ()>` lowering): `Ok` → value,
//!   `Err(NullOnErr)` → `NULL` (NILSXP).
//! - `Result<T, E: Display>` (the `#[miniextendr(unwrap_in_r)]` path):
//!   `Ok` → value, `Err(msg)` → `list(error = msg)` (VECSXP).
//!
//! The default error-boundary path (`Result<T, E: Debug>` → panic → R error)
//! is intentionally not exercised here: it only produces deterministic output
//! once routed through the macro-generated tagged-condition wrapper, which an
//! `into_sexp()`-level unit test can't construct. See issue #760.

mod r_test_utils;

use miniextendr_api::SEXPTYPE;
use miniextendr_api::into_r::{IntoR, NullOnErr};
use miniextendr_api::prelude::{SEXP, SexpExt};

#[test]
fn into_r_result_suite() {
    r_test_utils::with_r_thread(|| {
        test_null_on_err_ok_is_integer_scalar();
        test_null_on_err_err_is_null();
        test_unwrap_in_r_ok_is_integer_scalar();
        test_unwrap_in_r_err_is_error_list();
    });
}

// region: Result<T, NullOnErr> (the `Result<T, ()>` lowering)

/// `Result::<i32, NullOnErr>::Ok(42)` → scalar INTSXP holding `42`.
fn test_null_on_err_ok_is_integer_scalar() {
    let sexp: SEXP = Result::<i32, NullOnErr>::Ok(42).into_sexp();
    assert_eq!(sexp.type_of(), SEXPTYPE::INTSXP);
    assert_eq!(sexp.xlength(), 1);
    assert_eq!(sexp.as_integer(), Some(42));
}

/// `Result::<i32, NullOnErr>::Err(NullOnErr)` → `NULL` (NILSXP).
fn test_null_on_err_err_is_null() {
    let sexp: SEXP = Result::<i32, NullOnErr>::Err(NullOnErr).into_sexp();
    assert_eq!(sexp.type_of(), SEXPTYPE::NILSXP);
    assert!(sexp.is_null_or_nil());
}

// endregion

// region: Result<T, E: Display> (the `#[miniextendr(unwrap_in_r)]` path)

/// `Result::<i32, String>::Ok(7)` → scalar INTSXP holding `7`.
fn test_unwrap_in_r_ok_is_integer_scalar() {
    let sexp: SEXP = Result::<i32, String>::Ok(7).into_sexp();
    assert_eq!(sexp.type_of(), SEXPTYPE::INTSXP);
    assert_eq!(sexp.xlength(), 1);
    assert_eq!(sexp.as_integer(), Some(7));
}

/// `Result::<i32, String>::Err("boom")` → `list(error = "boom")` (VECSXP).
fn test_unwrap_in_r_err_is_error_list() {
    let sexp: SEXP = Result::<i32, String>::Err("boom".to_string()).into_sexp();
    assert_eq!(sexp.type_of(), SEXPTYPE::VECSXP);
    assert_eq!(sexp.xlength(), 1);

    // The single element is named "error".
    let names = sexp.get_names();
    assert_eq!(names.type_of(), SEXPTYPE::STRSXP);
    assert_eq!(
        names.string_elt_str(0).map(|s| s.to_string()),
        Some("error".to_string())
    );

    // ... and carries the Display-formatted message as a length-1 STRSXP.
    let err_elt = sexp.vector_elt(0);
    assert_eq!(err_elt.type_of(), SEXPTYPE::STRSXP);
    assert_eq!(
        err_elt.string_elt_str(0).map(|s| s.to_string()),
        Some("boom".to_string())
    );
}

// endregion
