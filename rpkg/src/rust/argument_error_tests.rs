//! One argument-error condition whichever side catches the bad argument
//! (#1591), driven by `tests/testthat/test-argument-errors.R`.
//!
//! `arg_error_ratio` takes scalar `AsNumeric`s: two values fail the R-side
//! length check. `arg_error_peak` takes an `AsNumericVec`: `"BLQ"` passes the
//! R-side type check and fails the Rust conversion. Both raise the same
//! classes, `kind = "conversion"` and `e$param`; the conversion adds
//! `e$rust_type`. rpkg sets no `conversion_error_class`; the configured case
//! lives in `tests/cross-package/producer.pkg`. The `arg_error_body*`
//! fixtures raise the same condition from a function body with `arg_error!`
//! (#1740).

use crate::match_arg_tests::{FillMode, Mode};
use miniextendr_api::{
    AsNumeric, AsNumericVec, DataFrame, MatchArg, arg_error, defer_warning, miniextendr,
};

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

/// `no_na` on a vector marker: its NA check says `must not contain NA`, as a
/// `Vec<T>`'s does.
/// @param obs Numbers, strings or factor labels, none of them `NA`.
#[miniextendr(internal)]
pub fn arg_error_no_na_peak(#[miniextendr(no_na)] obs: AsNumericVec) -> Option<f64> {
    obs.0.into_iter().flatten().reduce(f64::max)
}
// endregion

// region: per-element failures, 1-based and batched

/// `Vec<u16>` whose R-side whole-number check passes values the conversion
/// refuses (out of range, `NA`): every failing element is listed, by reason
/// and 1-based position.
/// @param counts Whole numbers from 0 to 65535.
#[miniextendr(internal)]
pub fn arg_error_u16s(counts: Vec<u16>) -> i32 {
    counts.into_iter().map(i32::from).sum()
}

/// A tuple argument: an unnamed list of a whole number and a string.
/// @param pair `list(<integer>, <string>)`.
#[miniextendr(internal)]
pub fn arg_error_pair(pair: (i32, String)) -> String {
    format!("{}:{}", pair.0, pair.1)
}

/// A data frame behind a newtype the macro does not see into at the
/// parameter: the derive declares what its inner type accepts (`a data
/// frame`), so a frame that fails later is refused in the same words.
#[derive(miniextendr_api::TryFromSexp)]
pub struct FrameArg(pub DataFrame);

/// The number of rows of a data frame given through a newtype.
/// @param x A data frame.
/// @noRd
#[miniextendr(noexport)]
pub fn frame_arg(x: FrameArg) -> i32 {
    i32::try_from(x.0.nrow()).expect("fewer than 2^31 rows")
}

/// `strict` input: a logical, raw, fractional or out-of-range value is the
/// argument error, not a panic.
/// @param n A whole number.
/// @param ids Whole numbers.
#[miniextendr(internal, strict)]
pub fn arg_error_strict_inputs(n: i64, ids: Vec<i64>) -> String {
    format!("{n}:{}", ids.len())
}
// endregion

// region: match_arg choices without the attribute

/// A `MatchArg` enum parameter without `#[miniextendr(match_arg)]`: no R-side
/// `match.arg()` check, so a bad value reaches the conversion, whose error
/// still names the choices.
/// @param speed One of `"Fast"`, `"Safe"`, `"Debug"`.
#[miniextendr(internal)]
pub fn arg_error_plain_mode(speed: Mode) -> String {
    format!("{speed:?}")
}
// endregion

// region: arg_error! from a body (#1740)

/// The message `match_arg_fill()`'s wrapper raises for a `fill` that is not
/// a choice.
const FILL_NOT_A_CHOICE: &str = r#"'fill' should be one of "drop", "draw", "error""#;

/// Whether `fill` is one of [`FillMode`]'s choices, exactly.
fn is_fill_choice(fill: &str) -> bool {
    FillMode::from_choice(fill).is_some()
}

/// `fill` checked by the body: anything but an exact choice raises the
/// argument error `match_arg_fill()`'s wrapper raises.
/// @param fill A string.
#[miniextendr(internal)]
pub fn arg_error_body(fill: &str) -> String {
    if !is_fill_choice(fill) {
        arg_error!(param = "fill", "{FILL_NOT_A_CHOICE}");
    }
    fill.to_string()
}

/// [`arg_error_body`] with `call = none`.
/// @param fill A string.
#[miniextendr(internal)]
pub fn arg_error_body_callless(fill: &str) -> String {
    if !is_fill_choice(fill) {
        arg_error!(call = none, param = "fill", "{FILL_NOT_A_CHOICE}");
    }
    fill.to_string()
}

/// A deferred warning, then `arg_error!`: the warning is signalled first.
/// @param fill A string.
#[miniextendr(internal)]
pub fn arg_error_after_deferred(fill: &str) -> String {
    defer_warning!(class = "pkg_note", "checking {fill:?}");
    if !is_fill_choice(fill) {
        arg_error!(param = "fill", "{FILL_NOT_A_CHOICE}");
    }
    fill.to_string()
}

/// [`arg_error_body`] on the worker thread.
/// @param fill A string.
#[cfg(feature = "worker-thread")]
#[miniextendr(internal, worker)]
pub fn arg_error_body_worker(fill: String) -> String {
    if !is_fill_choice(&fill) {
        arg_error!(param = "fill", "{FILL_NOT_A_CHOICE}");
    }
    fill
}

/// [`arg_error_body`] under `call = caller`: the condition names the call a
/// helper passes as `.call`, as the wrapper's own checks do.
/// @param fill A string.
/// @noRd
#[miniextendr(noexport, call = caller)]
pub fn arg_error_body_caller_impl(fill: &str) -> String {
    if !is_fill_choice(fill) {
        arg_error!(param = "fill", "{FILL_NOT_A_CHOICE}");
    }
    fill.to_string()
}
// endregion
