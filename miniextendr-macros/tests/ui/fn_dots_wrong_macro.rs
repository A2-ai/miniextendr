//! Test: dots must use typed_list! macro.

use miniextendr_macros::miniextendr;

#[miniextendr(dots = vec!(i32))]
fn my_fn(_dots: &miniextendr_api::Dots) {}

fn main() {}
