//! Test: RFactor only supports fieldless enum variants.

use miniextendr_macros::RFactor;

#[derive(RFactor)]
enum Bad {
    A,
    B { x: i32 },
}

fn main() {}
