//! Test: ALTREP complex rejects dataptr + subset together.

use miniextendr_macros::AltrepComplex;

#[derive(AltrepComplex)]
#[altrep(dataptr, subset)]
struct MyData {
    len: usize,
}

fn main() {}
