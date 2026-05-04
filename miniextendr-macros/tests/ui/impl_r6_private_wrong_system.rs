//! Test: #[r6(private)] on non-R6 class system should fail.

use miniextendr_macros::miniextendr;

#[derive(miniextendr_api::ExternalPtr)]
struct Foo;

#[miniextendr(env)]
impl Foo {
    #[miniextendr(env(private))]
    fn value(&self) -> i32 {
        0
    }
}

fn main() {}
