//! Test: unsafe() without arguments

use miniextendr_macros::miniextendr;

#[miniextendr(unsafe())]
fn my_fn() {}

fn main() {}
