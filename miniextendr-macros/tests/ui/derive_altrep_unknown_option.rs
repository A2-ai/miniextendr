//! Test: ALTREP derive rejects unknown #[altrep(...)] keys.

use miniextendr_macros::AltrepInteger;

#[derive(AltrepInteger)]
#[altrep(typo_option)]
struct MyData {
    len: usize,
}

fn main() {}
