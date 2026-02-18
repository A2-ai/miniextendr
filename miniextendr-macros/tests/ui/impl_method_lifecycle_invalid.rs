//! Test: invalid lifecycle stage on method.

use miniextendr_macros::miniextendr;

struct MyType;

#[miniextendr]
impl MyType {
    fn new() -> Self {
        MyType
    }

    #[miniextendr(lifecycle = "bogus")]
    fn bad(&self) -> i32 {
        42
    }
}

fn main() {}
