//! Test: unknown class system identifier.

use miniextendr_macros::miniextendr;

struct MyType;

#[miniextendr(flutter)]
impl MyType {
    fn new() -> Self {
        MyType
    }
}

fn main() {}
