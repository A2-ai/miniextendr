//! Test: ALTREP list rejects serialize option.

use miniextendr_macros::AltrepList;

#[derive(AltrepList)]
#[altrep(serialize)]
struct MyData {
    len: usize,
}

fn main() {}
