//! Test: RFactor can only be applied to enums.

use miniextendr_macros::RFactor;

#[derive(RFactor)]
struct Bad {
    x: i32,
}

fn main() {}
