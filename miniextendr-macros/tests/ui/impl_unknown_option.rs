//! Test: unknown impl option should fail.

use miniextendr_macros::miniextendr;

struct Foo;

#[miniextendr(env, bogus = "x")]
impl Foo {
    fn value(&self) -> i32 {
        0
    }
}

fn main() {}
