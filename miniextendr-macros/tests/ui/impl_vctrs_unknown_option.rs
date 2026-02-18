//! Test: unknown option in vctrs() on impl block.

use miniextendr_macros::miniextendr;

struct MyType;

#[miniextendr(vctrs(unknown_opt = true))]
impl MyType {
    fn new() -> Self {
        MyType
    }
}

fn main() {}
