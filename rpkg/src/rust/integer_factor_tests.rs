//! Integer conversions refuse a factor in Rust, where no R-side check runs
//! (#1660). Each fixture turns its R precondition off, so a direct call and
//! the wrapper both reach the Rust conversion with the factor itself.

use miniextendr_api::prelude::*;

/// Length of an integer slice.
/// @param x An integer vector.
#[miniextendr(no_preconditions, noexport)]
pub fn factor_refusal_slice(x: &[i32]) -> i32 {
    i32::try_from(x.len()).unwrap()
}

/// An integer vector, returned as read.
/// @param x An integer vector.
#[miniextendr(no_preconditions, noexport)]
pub fn factor_refusal_vec(x: Vec<i32>) -> Vec<i32> {
    x
}

/// An integer vector that may hold NA, returned as read.
/// @param x An integer vector.
#[miniextendr(no_preconditions, noexport)]
pub fn factor_refusal_option_vec(x: Vec<Option<i32>>) -> Vec<Option<i32>> {
    x
}

/// An integer vector widened to `i64`.
/// @param x An integer or double vector.
#[miniextendr(no_preconditions, noexport)]
pub fn factor_refusal_i64_vec(x: Vec<i64>) -> Vec<i64> {
    x
}
