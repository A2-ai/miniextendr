//! Test: trait method with typed receiver should fail.

use miniextendr_macros::miniextendr;

#[miniextendr]
pub trait BadTypedSelf {
    fn foo(self: &Self);
}

fn main() {}
