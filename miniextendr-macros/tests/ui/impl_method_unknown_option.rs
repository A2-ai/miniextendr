//! Test: unknown method-level option.

use miniextendr_macros::miniextendr;

struct MyType;

#[miniextendr]
impl MyType {
    fn new() -> Self {
        MyType
    }

    #[miniextendr(bogus_option)]
    fn bad(&self) -> i32 {
        42
    }
}

fn main() {}
