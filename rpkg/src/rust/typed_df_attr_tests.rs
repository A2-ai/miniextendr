//! Fixtures for the `typed_df` attribute sugar (#722).
//!
//! Each function uses `#[miniextendr(typed_df(...))]` to declare an inline
//! data.frame schema; the macro auto-injects a validated `<param>_typed`
//! binding at the top of the function body.

use miniextendr_api::{SEXP, miniextendr};

/// Return the row count of a data.frame with `subject` (integer) and `weight`
/// (numeric) columns.
///
/// @param df A data.frame with at least `subject` (integer) and `weight`
///   (numeric) columns.
/// @return Integer scalar — the row count.
/// @export
#[miniextendr(typed_df(df = typed_dataframe!(subject: i32, weight: f64)))]
pub fn typed_df_attr_nrow(df: SEXP) -> i32 {
    df_typed.nrow() as i32
}

/// Sum the `weight` column of a data.frame with inline schema.
///
/// @param df A data.frame with at least `subject` (integer) and `weight`
///   (numeric) columns.
/// @return Numeric scalar — `sum(df$weight)`.
/// @export
#[miniextendr(typed_df(df = typed_dataframe!(subject: i32, weight: f64)))]
pub fn typed_df_attr_weight_sum(df: SEXP) -> f64 {
    df_typed.weight().iter().copied().sum()
}

/// Strict-mode (`@exact;`) with an optional column.
///
/// Only `x` (integer) and the optional `y` (numeric) are allowed;
/// any extra column causes a validation error.
///
/// @param df A data.frame containing exactly `x` (integer) and optionally `y`
///   (numeric).  Extra columns are rejected.
/// @return Integer scalar — `length(df$x) + length(df$y)` (or just
///   `length(df$x)` when `y` is absent).
/// @export
#[miniextendr(typed_df(df = typed_dataframe!(@exact; x: i32, y: Option<f64>)))]
pub fn typed_df_attr_exact_optional(df: SEXP) -> i32 {
    df_typed.x().len() as i32 + df_typed.y().map_or(0, |c| c.len() as i32)
}

/// Multi-param punchline: two data.frame inputs each get their own `_typed`
/// binding.
///
/// @param left A data.frame with at least an `a` (integer) column.
/// @param right A data.frame with at least a `b` (numeric) column.
/// @return Integer scalar — `nrow(left) + nrow(right)`.
/// @export
#[miniextendr(typed_df(left = typed_dataframe!(a: i32), right = typed_dataframe!(b: f64)))]
pub fn typed_df_attr_two(left: SEXP, right: SEXP) -> i32 {
    (left_typed.nrow() + right_typed.nrow()) as i32
}
