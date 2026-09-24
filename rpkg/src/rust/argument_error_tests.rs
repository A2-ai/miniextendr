//! One argument-error condition whichever side catches the bad argument
//! (#1591), driven by `tests/testthat/test-argument-errors.R`.
//!
//! `arg_error_ratio` takes scalar `AsNumeric`s: two values fail the R-side
//! length check. `arg_error_peak` takes an `AsNumericVec`: `"BLQ"` passes the
//! R-side type check and fails the Rust conversion. Both raise the same
//! classes, `kind = "conversion"` and `e$param`; the conversion adds
//! `e$rust_type`. rpkg sets no `conversion_error_class`; the configured case
//! lives in `tests/cross-package/producer.pkg`.

use miniextendr_api::{AsNumeric, AsNumericVec, miniextendr};

// region: the two paths

/// Ratio of two numbers read like `as.numeric()`.
/// @param num,den A number, string or factor label of length 1.
#[miniextendr(internal)]
pub fn arg_error_ratio(num: AsNumeric, den: AsNumeric) -> Option<f64> {
    Some(num.0? / den.0?)
}

/// Largest value of a vector read like `as.numeric()`.
/// @param dv Numbers, strings or factor labels.
#[miniextendr(internal)]
pub fn arg_error_peak(dv: AsNumericVec) -> Option<f64> {
    dv.0.into_iter().flatten().reduce(f64::max)
}

/// `arg_error_ratio` behind a hand-written R function
/// (`R/call_attribution.R`, `arg_error_ratio_caller()`): the length check
/// names the caller's call.
/// @param num,den A number, string or factor label of length 1.
/// @noRd
#[miniextendr(noexport, call = caller)]
pub fn arg_error_ratio_caller_impl(num: AsNumeric, den: AsNumeric) -> Option<f64> {
    Some(num.0? / den.0?)
}

/// `arg_error_peak` behind a hand-written R function
/// (`R/call_attribution.R`, `arg_error_peak_caller()`): the conversion error
/// names the caller's call.
/// @param dv Numbers, strings or factor labels.
/// @noRd
#[miniextendr(noexport, call = caller)]
pub fn arg_error_peak_caller_impl(dv: AsNumericVec) -> Option<f64> {
    dv.0.into_iter().flatten().reduce(f64::max)
}
// endregion

// region: conversion wording without the R-side checks

/// `no_preconditions`: every bad `x` reaches the Rust conversion, which says
/// what `x` must be in R terms.
/// @param x An integer of length 1.
#[miniextendr(internal, no_preconditions)]
pub fn arg_error_int_unchecked(x: i32) -> i32 {
    x
}

/// `no_preconditions` on a logical scalar.
/// @param flag `TRUE` or `FALSE`.
#[miniextendr(internal, no_preconditions)]
pub fn arg_error_flag_unchecked(flag: bool) -> bool {
    !flag
}

/// `no_preconditions` on a string.
/// @param s A single string.
#[miniextendr(internal, no_preconditions)]
pub fn arg_error_string_unchecked(s: String) -> String {
    s
}

/// `no_preconditions` on a character vector.
/// @param xs A character vector.
#[miniextendr(internal, no_preconditions)]
pub fn arg_error_strings_unchecked(xs: Vec<String>) -> i32 {
    i32::try_from(xs.len()).expect("vector length fits i32")
}
// endregion

// region: R-side checks the author names

/// `no_na`, `inherits` and `choices` on one function: each failure is the
/// same argument error.
/// @param x A classed double vector without NA.
/// @param mode One of `"fast"`, `"slow"`.
#[miniextendr(internal)]
pub fn arg_error_named_checks(
    #[miniextendr(no_na)]
    #[miniextendr(inherits = "mx_num")]
    x: Vec<f64>,
    #[miniextendr(choices("fast", "slow"))] mode: &str,
) -> String {
    format!("{mode}: {}", x.iter().sum::<f64>())
}
// endregion
