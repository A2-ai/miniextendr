//! Test: `label = "..."` is no longer an impl block option. The impl blocks of
//! a type are merged into one class without it (#1776).

use miniextendr_macros::miniextendr;

struct Foo;

#[miniextendr(env, label = "constructors")]
impl Foo {
    fn new() -> Self {
        Foo
    }
}

fn main() {}
