//! Test: impl block with multiple class systems should fail.

use miniextendr_macros::miniextendr;

struct Counter(i32);

#[miniextendr(r6, s4)]
impl Counter {
    fn new() -> Self {
        Counter(0)
    }
}

fn main() {}
