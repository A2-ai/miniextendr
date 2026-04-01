//! Test fixtures for into_r_as module (IntoRAs trait).

use miniextendr_api::ffi::SEXP;
use miniextendr_api::into_r_as::IntoRAs;
use miniextendr_api::prelude::*;

/// Convert a Vec<i64> to R integer vector via IntoRAs<i32>.
/// Widens i32 → i64, then narrows back via IntoRAs.
#[miniextendr]
pub fn into_r_as_i64_to_i32(x: Vec<i32>) -> SEXP {
    let wide: Vec<i64> = x.into_iter().map(|v| v as i64).collect();
    <Vec<i64> as IntoRAs<i32>>::into_r_as(wide).expect("IntoRAs<i32> failed")
}
