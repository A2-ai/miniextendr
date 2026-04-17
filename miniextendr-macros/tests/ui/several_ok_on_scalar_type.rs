//! Test: several_ok with a scalar Rust type should error at compile time.

use miniextendr_macros::miniextendr;

#[miniextendr]
fn bad_several_ok_scalar(#[miniextendr(choices("a", "b"), several_ok)] x: String) {}

fn main() {}
