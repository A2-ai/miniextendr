//! Test: an ALTREP family derive on an enum without `#[altrep(manual)]` is
//! pointed at `manual` and at `Coerce`; the error points at the `enum` keyword.

use miniextendr_macros::AltrepInteger;

/// Not a struct.
#[derive(AltrepInteger)]
pub enum Bad {
    A,
    B,
}

fn main() {}
