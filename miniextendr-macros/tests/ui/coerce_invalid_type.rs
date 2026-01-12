//! Test: coerce attribute with invalid type should fail.

use miniextendr_macros::miniextendr;

#[miniextendr]
fn bad_coerce(#[coerce = "not_a_type"] x: i32) -> i32 {
    x
}

fn main() {}
