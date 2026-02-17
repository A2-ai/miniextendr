//! Test: ALTREP raw rejects subset option.

use miniextendr_macros::AltrepRaw;

#[derive(AltrepRaw)]
#[altrep(subset)]
struct MyData {
    len: usize,
}

fn main() {}
