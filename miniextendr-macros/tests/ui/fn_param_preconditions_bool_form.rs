//! Test: the function attribute's `preconditions = bool` form on a parameter
//! is rejected, also next to a parameter option, instead of being dropped.

use miniextendr_api::miniextendr;

#[miniextendr]
pub fn scale(#[miniextendr(no_na, preconditions = false)] x: f64) -> f64 {
    x
}

fn main() {}
