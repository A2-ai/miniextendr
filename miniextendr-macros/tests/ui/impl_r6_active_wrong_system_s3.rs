//! Test: active marker on s3 class system should fail.
//! Exercises a non-env wrapper to confirm the validator message is wrapper-agnostic.

use miniextendr_macros::miniextendr;

#[derive(miniextendr_api::ExternalPtr)]
struct Foo;

#[miniextendr(s3)]
impl Foo {
    #[miniextendr(s3(active))]
    fn value(&self) -> i32 {
        0
    }
}

fn main() {}
