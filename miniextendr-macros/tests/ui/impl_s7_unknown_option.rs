//! Test: unknown option in s7().

use miniextendr_macros::miniextendr;

struct MyType;

#[miniextendr(s7(unknown_opt = true))]
impl MyType {
    fn new() -> Self {
        MyType
    }
}

fn main() {}
