//! Test: Vctrs can only be applied to structs.

use miniextendr_macros::Vctrs;

#[derive(Vctrs)]
enum Bad {
    A,
    B,
}

fn main() {}
