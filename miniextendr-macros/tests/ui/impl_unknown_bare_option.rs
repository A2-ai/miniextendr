//! Test: an unknown bare identifier on an impl block.
//!
//! It is neither a class system nor an impl option, so the error lists every
//! impl option.

use miniextendr_macros::miniextendr;

struct MyType;

#[miniextendr(flutter)]
impl MyType {
    fn new() -> Self {
        MyType
    }
}

fn main() {}
