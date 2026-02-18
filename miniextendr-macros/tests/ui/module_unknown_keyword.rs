//! Test: unknown keyword in miniextendr_module.

use miniextendr_macros::miniextendr_module;

miniextendr_module! {
    mod test_pkg;
    trait Foo;
}

fn main() {}
