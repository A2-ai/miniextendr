//! Test: invalid lifecycle stage value.

use miniextendr_macros::miniextendr;

#[miniextendr(lifecycle = "bogus")]
fn my_fn() -> i32 {
    42
}

fn main() {}
