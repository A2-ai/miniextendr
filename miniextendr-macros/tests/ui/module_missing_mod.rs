//! Test: miniextendr_module without mod declaration

use miniextendr_macros::miniextendr_module;

miniextendr_module! {
    fn some_function;
}

fn main() {}
