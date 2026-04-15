//! Test: several_ok without choices() should error

use miniextendr_macros::miniextendr;

#[miniextendr]
fn bad_several_ok(#[miniextendr(several_ok)] x: Vec<String>) {}

fn main() {}
