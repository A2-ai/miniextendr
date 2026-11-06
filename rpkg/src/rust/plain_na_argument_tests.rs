//! A plain `NA`, R's untyped missing value, given to an optional scalar
//! argument reads as `None`, as `NULL` does (#2305): through the R-side
//! check, and through the Rust conversion alone (`no_preconditions`, or a
//! `.Call()` that skips the wrapper). Every other value keeps its check.
//! Tested in `tests/testthat/test-plain-na-arguments.R`.
use miniextendr_api::{Missing, Nullable, Rcomplex, miniextendr};

use crate::match_arg_tests::Mode;

/// `none`, or the value.
fn describe<T: std::fmt::Debug>(x: Option<T>) -> String {
    x.map_or_else(|| "none".to_string(), |v| format!("{v:?}"))
}

// region: optional scalars, with the R-side checks

/// An optional integer.
/// @param x `NULL`, `NA` or an integer.
/// @noRd
#[miniextendr(noexport)]
pub fn plain_na_i32(x: Option<i32>) -> String {
    describe(x)
}

/// An optional double.
/// @param x `NULL`, `NA` or a double.
/// @noRd
#[miniextendr(noexport)]
pub fn plain_na_f64(x: Option<f64>) -> String {
    describe(x)
}

/// An optional string.
/// @param x `NULL`, `NA` or a string.
/// @noRd
#[miniextendr(noexport)]
pub fn plain_na_string(x: Option<String>) -> String {
    describe(x)
}

/// An optional borrowed string.
/// @param x `NULL`, `NA` or a string.
/// @noRd
#[miniextendr(noexport)]
pub fn plain_na_str(x: Option<&str>) -> String {
    describe(x)
}

/// An optional logical.
/// @param x `NULL`, `NA`, `TRUE` or `FALSE`.
/// @noRd
#[miniextendr(noexport)]
pub fn plain_na_bool(x: Option<bool>) -> String {
    describe(x)
}

/// An optional raw byte.
/// @param x `NULL`, `NA` or a raw value.
/// @noRd
#[miniextendr(noexport)]
pub fn plain_na_raw(x: Option<u8>) -> String {
    describe(x)
}

/// An optional complex number.
/// @param x `NULL`, `NA` or a complex value.
/// @noRd
#[miniextendr(noexport)]
pub fn plain_na_complex(x: Option<Rcomplex>) -> String {
    describe(x.map(|c| (c.r, c.i)))
}

/// An optional 64-bit integer, read from any numeric type.
/// @param x `NULL`, `NA` or a whole number.
/// @noRd
#[miniextendr(noexport)]
pub fn plain_na_i64(x: Option<i64>) -> String {
    describe(x)
}

/// An optional unsigned integer.
/// @param x `NULL`, `NA` or a non-negative whole number.
/// @noRd
#[miniextendr(noexport)]
pub fn plain_na_u32(x: Option<u32>) -> String {
    describe(x)
}

// endregion

// region: the Nullable twins

/// `plain_na_i32()` with `Nullable<i32>`.
/// @param x `NULL`, `NA` or an integer.
/// @noRd
#[miniextendr(noexport)]
pub fn plain_na_i32_nullable(x: Nullable<i32>) -> String {
    describe(x.0)
}

/// `plain_na_f64()` with `Nullable<f64>`.
/// @param x `NULL`, `NA` or a double.
/// @noRd
#[miniextendr(noexport)]
pub fn plain_na_f64_nullable(x: Nullable<f64>) -> String {
    describe(x.0)
}

/// `plain_na_string()` with `Nullable<String>`.
/// @param x `NULL`, `NA` or a string.
/// @noRd
#[miniextendr(noexport)]
pub fn plain_na_string_nullable(x: Nullable<String>) -> String {
    describe(x.0)
}

/// `plain_na_bool()` with `Nullable<bool>`.
/// @param x `NULL`, `NA`, `TRUE` or `FALSE`.
/// @noRd
#[miniextendr(noexport)]
pub fn plain_na_bool_nullable(x: Nullable<bool>) -> String {
    describe(x.0)
}

// endregion

// region: the Rust conversion alone

/// `plain_na_f64()` without the R-side checks.
/// @param x `NULL`, `NA` or a double.
/// @noRd
#[miniextendr(noexport, no_preconditions)]
pub fn plain_na_f64_unchecked(x: Option<f64>) -> String {
    describe(x)
}

/// `plain_na_string()` without the R-side checks.
/// @param x `NULL`, `NA` or a string.
/// @noRd
#[miniextendr(noexport, no_preconditions)]
pub fn plain_na_string_unchecked(x: Option<String>) -> String {
    describe(x)
}

/// `plain_na_bool()` without the R-side checks.
/// @param x `NULL`, `NA`, `TRUE` or `FALSE`.
/// @noRd
#[miniextendr(noexport, no_preconditions)]
pub fn plain_na_bool_unchecked(x: Option<bool>) -> String {
    describe(x)
}

// endregion

// region: optional choices

/// An optional `match_arg` choice.
/// @param mode `NULL`, `NA`, or one of the `Mode` choices.
/// @noRd
#[miniextendr(noexport)]
pub fn plain_na_mode(#[miniextendr(match_arg)] mode: Option<Mode>) -> String {
    describe(mode)
}

/// `plain_na_mode()` with `Nullable<Mode>`.
/// @param mode `NULL`, `NA`, or one of the `Mode` choices.
/// @noRd
#[miniextendr(noexport)]
pub fn plain_na_mode_nullable(#[miniextendr(match_arg)] mode: Nullable<Mode>) -> String {
    describe(mode.0)
}

/// An omittable optional choice: an omitted argument is `Absent`, `NULL` and
/// `NA` are `None`.
/// @param mode `NULL`, `NA`, one of the `Mode` choices, or nothing.
/// @noRd
#[miniextendr(noexport)]
pub fn plain_na_missing_mode(#[miniextendr(match_arg)] mode: Missing<Option<Mode>>) -> String {
    match mode {
        Missing::Absent => "absent".to_string(),
        Missing::Present(mode) => describe(mode),
    }
}

/// An optional `choices(...)` string.
/// @param color `NULL`, `NA`, or one of the colors.
/// @noRd
#[miniextendr(noexport)]
pub fn plain_na_color(
    #[miniextendr(choices("red", "green", "blue"))] color: Option<String>,
) -> String {
    describe(color)
}

// endregion
