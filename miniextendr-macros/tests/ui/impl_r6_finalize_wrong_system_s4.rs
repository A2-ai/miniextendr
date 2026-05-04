//! Test: finalize marker on s4 class system should fail.
//! Exercises a non-env wrapper to confirm the validator message is wrapper-agnostic.

use miniextendr_macros::miniextendr;

#[derive(miniextendr_api::ExternalPtr)]
struct Foo;

#[miniextendr(s4)]
impl Foo {
    #[miniextendr(s4(finalize))]
    fn destroy(self) {}
}

fn main() {}
