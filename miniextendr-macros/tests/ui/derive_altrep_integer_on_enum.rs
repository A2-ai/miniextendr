//! Test: the ALTREP family derives only support structs; on an enum the
//! error points at the `enum` keyword.

use miniextendr_macros::AltrepInteger;

/// Not a struct.
#[derive(AltrepInteger)]
pub enum Bad {
    A,
    B,
}

fn main() {}
