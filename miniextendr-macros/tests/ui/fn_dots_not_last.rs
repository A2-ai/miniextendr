//! Test: dots parameter not in last position should fail.

use miniextendr_macros::miniextendr;

// Stub type
struct Dots;

#[miniextendr]
fn bad_fn(dots: &Dots, x: i32) -> i32 {
    x
}

fn main() {}
