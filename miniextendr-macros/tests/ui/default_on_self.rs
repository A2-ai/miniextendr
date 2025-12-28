//! Test: default on self parameter is not allowed

use miniextendr_macros::miniextendr;

struct Foo;

#[miniextendr(env)]
impl Foo {
    #[miniextendr(defaults(self = "NULL"))]
    pub fn method(&self) {}
}

fn main() {}
