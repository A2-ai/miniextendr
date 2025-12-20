//! Test: pattern parameter (not supported)

use miniextendr_macros::miniextendr;

#[miniextendr]
fn destructure_tuple((a, b): (i32, i32)) -> i32 {
    a + b
}

fn main() {}
