//! Test: unknown lifecycle key on method.

use miniextendr_macros::miniextendr;

struct MyType;

#[miniextendr]
impl MyType {
    fn new() -> Self {
        MyType
    }

    #[miniextendr(lifecycle(unknown = "val"))]
    fn bad(&self) -> i32 {
        42
    }
}

fn main() {}
