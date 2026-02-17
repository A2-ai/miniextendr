//! Test: ALTREP string rejects subset option.

use miniextendr_macros::AltrepString;

#[derive(AltrepString)]
#[altrep(subset)]
struct MyData {
    len: usize,
}

fn main() {}
