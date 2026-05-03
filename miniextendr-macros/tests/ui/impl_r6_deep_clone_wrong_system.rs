//! Test: #[r6(deep_clone)] on non-R6 class system should fail.

use miniextendr_macros::miniextendr;

#[derive(miniextendr_api::ExternalPtr)]
struct Foo;

#[miniextendr(env)]
impl Foo {
    #[miniextendr(env(deep_clone))]
    fn clone_fields(&self, _name: String, _value: miniextendr_api::Robj) {}
}

fn main() {}
