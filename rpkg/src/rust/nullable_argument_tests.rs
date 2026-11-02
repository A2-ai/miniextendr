//! `Nullable<T>` arguments read as `Option<T>` (#2294): the same R-side
//! checks, the same conversion message and condition class, and `coerce`,
//! `no_na` and `match_arg` look through it. Each fixture has an `Option<T>`
//! twin. Tested in `tests/testthat/test-nullable-arguments.R`.
use miniextendr_api::{Missing, Nullable, miniextendr};

use crate::match_arg_tests::Mode;

/// `none`, or the value.
fn describe<T: std::fmt::Debug>(x: Option<T>) -> String {
    x.map_or_else(|| "none".to_string(), |v| format!("{v:?}"))
}

// region: plain arguments, with the R-side checks

/// An optional integer: `NULL` and `NA` read `None`.
/// @param x `NULL`, `NA` or an integer.
/// @noRd
#[miniextendr(noexport)]
pub fn nullable_arg_i32(x: Nullable<i32>) -> String {
    describe(x.0)
}

/// `nullable_arg_i32()` with `Option<i32>`.
/// @param x `NULL`, `NA` or an integer.
/// @noRd
#[miniextendr(noexport)]
pub fn option_arg_i32(x: Option<i32>) -> String {
    describe(x)
}

/// An optional string: `NULL` and `NA` read `None`.
/// @param x `NULL`, `NA` or a string.
/// @noRd
#[miniextendr(noexport)]
pub fn nullable_arg_string(x: Nullable<String>) -> String {
    describe(x.0)
}

/// `nullable_arg_string()` with `Option<String>`.
/// @param x `NULL`, `NA` or a string.
/// @noRd
#[miniextendr(noexport)]
pub fn option_arg_string(x: Option<String>) -> String {
    describe(x)
}

/// An optional integer vector: `NULL` reads `None`.
/// @param x `NULL` or an integer vector.
/// @noRd
#[miniextendr(noexport)]
pub fn nullable_arg_i32_vec(x: Nullable<Vec<i32>>) -> String {
    describe(x.0)
}

/// `nullable_arg_i32_vec()` with `Option<Vec<i32>>`.
/// @param x `NULL` or an integer vector.
/// @noRd
#[miniextendr(noexport)]
pub fn option_arg_i32_vec(x: Option<Vec<i32>>) -> String {
    describe(x)
}

/// An omittable optional integer: an omitted argument is `Absent`.
/// @param x `NULL`, `NA`, an integer, or nothing.
/// @noRd
#[miniextendr(noexport)]
pub fn nullable_arg_missing_i32(x: Missing<Nullable<i32>>) -> String {
    match x {
        Missing::Absent => "absent".to_string(),
        Missing::Present(x) => describe(x.0),
    }
}

// endregion

// region: the Rust conversion alone

/// `nullable_arg_i32()` without the R-side checks: a wrong type reaches the
/// Rust conversion, which words it as for `Option<i32>`.
/// @param x `NULL`, `NA` or an integer.
/// @noRd
#[miniextendr(noexport, no_preconditions)]
pub fn nullable_arg_i32_unchecked(x: Nullable<i32>) -> String {
    describe(x.0)
}

/// `nullable_arg_i32_unchecked()` with `Option<i32>`.
/// @param x `NULL`, `NA` or an integer.
/// @noRd
#[miniextendr(noexport, no_preconditions)]
pub fn option_arg_i32_unchecked(x: Option<i32>) -> String {
    describe(x)
}

// endregion

// region: coerce, no_na, match_arg

/// A coerced optional integer: a whole-number double converts, `NA` of any
/// type reads `None`.
/// @param x `NULL`, `NA`, or an integer-like scalar.
/// @noRd
#[miniextendr(noexport)]
pub fn nullable_arg_coerce_i32(#[miniextendr(coerce)] x: Nullable<i32>) -> String {
    describe(x.0)
}

/// `nullable_arg_coerce_i32()` with `Option<i32>`.
/// @param x `NULL`, `NA`, or an integer-like scalar.
/// @noRd
#[miniextendr(noexport)]
pub fn option_arg_coerce_i32(#[miniextendr(coerce)] x: Option<i32>) -> String {
    describe(x)
}

/// A coerced, omittable optional double.
/// @param x `NULL`, `NA`, a number-like scalar, or nothing.
/// @noRd
#[miniextendr(noexport)]
pub fn nullable_arg_coerce_missing_f64(#[miniextendr(coerce)] x: Missing<Nullable<f64>>) -> String {
    match x {
        Missing::Absent => "absent".to_string(),
        Missing::Present(x) => describe(x.0),
    }
}

/// `no_na` on an optional numeric vector: `NULL` passes, an `NA` element is
/// refused.
/// @param x `NULL` or doubles without `NA`.
/// @noRd
#[miniextendr(noexport)]
pub fn nullable_arg_no_na(#[miniextendr(no_na)] x: Nullable<Vec<f64>>) -> String {
    describe(x.0)
}

/// `nullable_arg_no_na()` with `Option<Vec<f64>>`.
/// @param x `NULL` or doubles without `NA`.
/// @noRd
#[miniextendr(noexport)]
pub fn option_arg_no_na(#[miniextendr(no_na)] x: Option<Vec<f64>>) -> String {
    describe(x)
}

/// An optional choice: the formal defaults to `NULL`, which reads `None`.
/// @param mode `NULL`, or one of the `Mode` choices.
/// @noRd
#[miniextendr(noexport)]
pub fn nullable_arg_mode(#[miniextendr(match_arg)] mode: Nullable<Mode>) -> String {
    describe(mode.0)
}

/// `nullable_arg_mode()` with `Option<Mode>`.
/// @param mode `NULL`, or one of the `Mode` choices.
/// @noRd
#[miniextendr(noexport)]
pub fn option_arg_mode(#[miniextendr(match_arg)] mode: Option<Mode>) -> String {
    describe(mode)
}

/// An omittable optional choice: the formal keeps the choices, an omitted
/// argument is `Absent` and `NULL` is `None`.
/// @param mode `NULL`, one of the `Mode` choices, or nothing.
/// @noRd
#[miniextendr(noexport)]
pub fn nullable_arg_missing_mode(
    #[miniextendr(match_arg)] mode: Missing<Nullable<Mode>>,
) -> String {
    match mode {
        Missing::Absent => "absent".to_string(),
        Missing::Present(mode) => describe(mode.0),
    }
}

// endregion
