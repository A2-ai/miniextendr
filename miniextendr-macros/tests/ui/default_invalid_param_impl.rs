//! Test: defaults(...) referencing non-existent parameters in impl methods

use miniextendr_macros::miniextendr;

struct Foo;

#[miniextendr(env)]
impl Foo {
    #[miniextendr(defaults(nonexistent = "42", another = "100"))]
    pub fn method(&self, x: i32) {}
}

fn main() {}
