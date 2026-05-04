//! Test: deep_clone marker on s7 class system should fail.
//! Exercises a non-env wrapper to confirm the validator message is wrapper-agnostic.

use miniextendr_macros::miniextendr;

#[derive(miniextendr_api::ExternalPtr)]
struct Foo;

#[miniextendr(s7)]
impl Foo {
    #[miniextendr(s7(deep_clone))]
    fn clone_fields(&self, _name: String, _value: miniextendr_api::Robj) {}
}

fn main() {}
