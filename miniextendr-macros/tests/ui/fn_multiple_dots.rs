//! Test: function with multiple dots parameters should fail.

use miniextendr_macros::miniextendr;

// Stub type
struct Dots;

#[miniextendr]
fn bad_fn(dots1: &Dots, dots2: &Dots) -> i32 {
    42
}

fn main() {}
