//! Test: #[r6(finalize)] on non-R6 class system should fail.

use miniextendr_macros::miniextendr;

#[derive(miniextendr_api::ExternalPtr)]
struct Foo;

#[miniextendr(env)]
impl Foo {
    #[miniextendr(env(finalize))]
    fn destroy(self) {}
}

fn main() {}
