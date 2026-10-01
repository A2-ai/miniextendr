//! `Option<DataFrame>` and `Option<Either<L, R>>` parameters: `NULL` is
//! `None`, and any other value converts as the inner type (a data frame
//! keeps its attributes and `NA` cells, an `Either` tries its left arm first
//! and fails with both arms' reasons). Tested in
//! `tests/testthat/test-optional-inputs.R`.
use miniextendr_api::{AsNumericVec, DataFrame, Either, IntoR, SEXP, miniextendr};

/// Accept `NULL` or a data frame, and return nothing.
/// @param x `NULL`, or a value of the inner type: a data frame for
///   `optional_frame()`, a single double or string for `optional_choice()`,
///   numbers or a data frame for `optional_grid()`.
#[miniextendr]
pub fn optional_frame(_x: Option<DataFrame>) {}

/// Accept `NULL`, a single double or a single string, and return nothing.
/// @param x `NULL`, or a value of the inner type: a data frame for
///   `optional_frame()`, a single double or string for `optional_choice()`,
///   numbers or a data frame for `optional_grid()`.
#[miniextendr]
pub fn optional_choice(_x: Option<Either<f64, String>>) {}

/// Accept `NULL`, numbers (read like `as.numeric()`) or a data frame, and
/// return nothing.
/// @param x `NULL`, or a value of the inner type: a data frame for
///   `optional_frame()`, a single double or string for `optional_choice()`,
///   numbers or a data frame for `optional_grid()`.
#[miniextendr]
pub fn optional_grid(_x: Option<Either<AsNumericVec, DataFrame>>) {}

/// `optional_frame()` that hands the converted value back: `NULL`, or the
/// data frame it was given (the same object).
/// @param x `NULL` or a data frame.
/// @noRd
#[miniextendr(noexport)]
pub fn optional_frame_echo(x: Option<DataFrame>) -> SEXP {
    x.map_or_else(SEXP::nil, IntoR::into_sexp)
}

/// The data frame parameter without the `Option`, for comparing errors.
/// @param x A data frame.
/// @noRd
#[miniextendr(noexport)]
pub fn required_frame_echo(x: DataFrame) -> DataFrame {
    x
}

/// `left:<value>` or `right:<value>`.
fn describe_choice(x: Either<f64, String>) -> String {
    match x {
        Either::Left(d) => format!("left:{d}"),
        Either::Right(s) => format!("right:{s}"),
    }
}

/// `optional_choice()` that says what it got: `none`, `left:<double>` or
/// `right:<string>`.
/// @param x `NULL`, a single double, or a single string.
/// @noRd
#[miniextendr(noexport)]
pub fn optional_choice_tag(x: Option<Either<f64, String>>) -> String {
    x.map_or_else(|| "none".to_string(), describe_choice)
}

/// The choice parameter without the `Option`, for comparing errors.
/// @param x A single double or a single string.
/// @noRd
#[miniextendr(noexport)]
pub fn required_choice_tag(x: Either<f64, String>) -> String {
    describe_choice(x)
}

/// `numbers:<count>:<NA count>` or `frame:<rows>x<cols>`.
fn describe_grid(x: Either<AsNumericVec, DataFrame>) -> String {
    match x {
        Either::Left(v) => {
            let missing = v.0.iter().filter(|n| n.is_none()).count();
            format!("numbers:{}:{missing}", v.0.len())
        }
        Either::Right(frame) => format!("frame:{}x{}", frame.nrow(), frame.ncol()),
    }
}

/// `optional_grid()` that says what it got: `none`,
/// `numbers:<count>:<NA count>` or `frame:<rows>x<cols>`.
/// @param x `NULL`, a numeric, character or factor vector, or a data frame.
/// @noRd
#[miniextendr(noexport)]
pub fn optional_grid_tag(x: Option<Either<AsNumericVec, DataFrame>>) -> String {
    x.map_or_else(|| "none".to_string(), describe_grid)
}

/// The grid parameter without the `Option`, for comparing errors.
/// @param x A numeric, character or factor vector, or a data frame.
/// @noRd
#[miniextendr(noexport)]
pub fn required_grid_tag(x: Either<AsNumericVec, DataFrame>) -> String {
    describe_grid(x)
}

/// `no_na` on an optional data frame: the R guard (`is.null(x) ||
/// !anyNA(x)`) refuses a frame with an `NA` cell, as for a bare `DataFrame`
/// parameter.
/// @param x `NULL` or a data frame without `NA` cells.
/// @noRd
#[miniextendr(noexport)]
pub fn optional_frame_no_na(#[miniextendr(no_na)] x: Option<DataFrame>) -> String {
    x.map_or_else(
        || "none".to_string(),
        |frame| format!("frame:{}x{}", frame.nrow(), frame.ncol()),
    )
}

/// `no_na` on an optional numbers-or-frame `Either`: no R guard runs, the
/// arm the value converted to checks it. `NULL` passes, numbers with an `NA`
/// (or text read as `NA`) are refused, and a data frame keeps its `NA` cells.
/// @param x `NULL`, numbers without `NA`, or a data frame.
/// @noRd
#[miniextendr(noexport)]
pub fn optional_grid_no_na(
    #[miniextendr(no_na)] x: Option<Either<AsNumericVec, DataFrame>>,
) -> String {
    x.map_or_else(|| "none".to_string(), describe_grid)
}
