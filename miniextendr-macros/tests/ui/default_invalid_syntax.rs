//! Test: default attribute with invalid R syntax should fail.

use miniextendr_macros::miniextendr;

#[miniextendr]
fn bad_default(#[default = "{{{{"] x: i32) -> i32 {
    x
}

fn main() {}
