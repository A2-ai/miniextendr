//! Test: unknown unsafe() option on method.

use miniextendr_macros::miniextendr;

struct MyType;

#[miniextendr]
impl MyType {
    fn new() -> Self {
        MyType
    }

    #[miniextendr(unsafe(bogus))]
    fn bad(&self) -> i32 {
        42
    }
}

fn main() {}
