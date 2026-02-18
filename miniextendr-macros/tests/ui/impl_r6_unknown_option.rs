//! Test: unknown option in r6().

use miniextendr_macros::miniextendr;

struct MyType;

#[miniextendr(r6(unknown_opt = true))]
impl MyType {
    fn new() -> Self {
        MyType
    }
}

fn main() {}
