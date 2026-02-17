//! Test: prefer option must be a string literal.

use miniextendr_macros::miniextendr;

#[miniextendr(prefer = 1)]
fn my_fn() -> i32 {
    42
}

fn main() {}
