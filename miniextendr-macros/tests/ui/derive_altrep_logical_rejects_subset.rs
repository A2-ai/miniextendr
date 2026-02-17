//! Test: ALTREP logical rejects subset option.

use miniextendr_macros::AltrepLogical;

#[derive(AltrepLogical)]
#[altrep(subset)]
struct MyData {
    len: usize,
}

fn main() {}
