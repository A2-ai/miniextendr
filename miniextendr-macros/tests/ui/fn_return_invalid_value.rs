//! Test: invalid return option value.

use miniextendr_macros::miniextendr;

#[miniextendr(return = "bogus")]
fn my_fn() -> i32 {
    42
}

fn main() {}
