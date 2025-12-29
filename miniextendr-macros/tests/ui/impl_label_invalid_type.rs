//! Test: label value must be a string literal.

use miniextendr_macros::miniextendr;

struct Foo;

#[miniextendr(label = constructors)]
impl Foo {
    fn new() -> Self {
        Foo
    }
}

fn main() {}
