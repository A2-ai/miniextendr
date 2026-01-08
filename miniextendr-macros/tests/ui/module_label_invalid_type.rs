//! Test: module impl label must be a string literal.

use miniextendr_macros::miniextendr_module;

miniextendr_module! {
    mod test;
    impl Foo as methods;
}

fn main() {}
