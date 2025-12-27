//! Test: #[r6(active)] on R6 class system should fail (not implemented).

use miniextendr_macros::miniextendr;

#[derive(miniextendr_api::ExternalPtr)]
struct Foo;

#[miniextendr(r6)]
impl Foo {
    #[miniextendr(r6(active))]
    fn value(&self) -> i32 {
        0
    }
}

fn main() {}
