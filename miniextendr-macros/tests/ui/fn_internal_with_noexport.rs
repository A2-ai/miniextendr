//! Test: `internal` and `noexport` are redundant together.

use miniextendr_macros::miniextendr;

#[miniextendr(internal, noexport)]
fn my_fn() {}

fn main() {}
