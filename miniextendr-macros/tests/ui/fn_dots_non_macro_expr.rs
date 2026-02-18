//! Test: dots must be a macro invocation.

use miniextendr_macros::miniextendr;

#[miniextendr(dots = "x")]
fn my_fn(_dots: &miniextendr_api::Dots) {}

fn main() {}
