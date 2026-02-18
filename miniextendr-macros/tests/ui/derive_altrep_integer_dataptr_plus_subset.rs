//! Test: ALTREP integer rejects dataptr + subset together.

use miniextendr_macros::AltrepInteger;

#[derive(AltrepInteger)]
#[altrep(dataptr, subset)]
struct MyData {
    len: usize,
}

fn main() {}
