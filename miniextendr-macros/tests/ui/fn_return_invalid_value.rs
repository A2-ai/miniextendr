//! Test: invalid prefer option value.

use miniextendr_macros::miniextendr;

#[miniextendr(prefer = "bogus")]
fn my_fn() -> i32 {
    42
}

fn main() {}
