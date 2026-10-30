//! Test: a per-parameter `coerce` on a type it has no conversion for (#2274).
//!
//! The option names the parameter, so doing nothing would hide a mistake; a
//! function-wide `coerce` skips such a type instead
//! (`pass/coerce_optional_params.rs`).

use miniextendr_api::miniextendr;

#[miniextendr]
pub fn coerced_string(#[miniextendr(coerce)] x: String) -> String {
    x
}

#[miniextendr]
pub fn coerced_optional_vec(#[miniextendr(coerce)] x: Option<Vec<i32>>) -> bool {
    x.is_some()
}

#[miniextendr]
pub fn coerced_slice(#[miniextendr(coerce)] x: &[f64]) -> f64 {
    x.iter().sum()
}

fn main() {}
