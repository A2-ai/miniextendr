//! Test: ALTREP real rejects subset option.

use miniextendr_macros::AltrepReal;

#[derive(AltrepReal)]
#[altrep(subset)]
struct MyData {
    len: usize,
}

fn main() {}
