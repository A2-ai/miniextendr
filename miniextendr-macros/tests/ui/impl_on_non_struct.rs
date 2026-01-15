//! Test: #[miniextendr] impl on non-struct should fail.

use miniextendr_macros::miniextendr;

enum NotAStruct {
    A,
    B,
}

#[miniextendr]
impl NotAStruct {
    fn value(&self) -> i32 {
        0
    }
}

fn main() {}
