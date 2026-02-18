//! Test: ALTREP list rejects dataptr option.

use miniextendr_macros::AltrepList;

#[derive(AltrepList)]
#[altrep(dataptr)]
struct MyData {
    len: usize,
}

fn main() {}
