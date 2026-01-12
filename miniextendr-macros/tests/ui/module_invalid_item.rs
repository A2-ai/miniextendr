//! Test: invalid item in miniextendr_module! should fail.

use miniextendr_macros::miniextendr_module;

miniextendr_module! {
    mod test;
    let x = 5;  // Invalid: only fn, struct, impl, use allowed
}

fn main() {}
