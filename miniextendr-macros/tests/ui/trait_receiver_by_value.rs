//! Test: trait method with by-value self should fail.

use miniextendr_macros::miniextendr;

#[miniextendr]
pub trait BadSelfByValue {
    fn foo(self);
}

fn main() {}
