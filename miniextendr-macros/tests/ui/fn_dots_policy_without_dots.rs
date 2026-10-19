//! Test: `dots = empty | forward(..) | data` on a function without `...`.
//!
//! A standalone wrapper's `...` is the function's own dots parameter: without
//! one there is nothing to check (R refuses an unknown argument itself) and
//! nothing to pass on (#2157).

use miniextendr_macros::miniextendr;

#[miniextendr(dots = empty)]
pub fn no_dots_to_check(x: i32) -> i32 {
    x
}

#[miniextendr(dots = forward(allow = ["scale"]))]
pub fn no_dots_to_forward(x: f64) -> f64 {
    x
}

fn main() {}
