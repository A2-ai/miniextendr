//! Test: trait method with generics should fail.

use miniextendr_macros::miniextendr;

#[miniextendr]
pub trait BadGeneric {
    fn foo<T>(&self, _x: T);
}

fn main() {}
