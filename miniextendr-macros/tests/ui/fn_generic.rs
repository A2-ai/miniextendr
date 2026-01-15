//! Test: generic function should fail.

use miniextendr_macros::miniextendr;

#[miniextendr]
fn bad_generic<T>(x: T) -> T {
    x
}

fn main() {}
