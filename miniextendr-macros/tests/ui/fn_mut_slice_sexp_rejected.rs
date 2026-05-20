//! Test: &mut [SEXP] parameters are rejected (SEXP is not RNativeType).

use miniextendr_api::sys::SEXP;
use miniextendr_macros::miniextendr;

#[miniextendr]
fn mutate_sexp_slice(_data: &mut [SEXP]) {}

fn main() {}
