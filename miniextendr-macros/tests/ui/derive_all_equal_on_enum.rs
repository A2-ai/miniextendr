//! Test: AllEqual derives on a struct; on an enum the error points at the
//! `enum` keyword.

use miniextendr_api::AllEqual;

/// An enum has no fields to compare.
#[derive(AllEqual)]
pub enum Bad {
    A,
    B,
}

fn main() {}
