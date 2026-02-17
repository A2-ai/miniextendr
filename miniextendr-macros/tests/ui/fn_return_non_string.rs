//! Test: return option must be a string literal.

use miniextendr_macros::miniextendr;

#[miniextendr(return = 1)]
fn my_fn() -> i32 {
    42
}

fn main() {}
