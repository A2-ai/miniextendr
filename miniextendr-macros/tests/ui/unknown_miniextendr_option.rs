//! Test: unknown option in #[miniextendr(...)]

use miniextendr_macros::miniextendr;

#[miniextendr(unknown_option)]
fn my_fn() {}

fn main() {}
