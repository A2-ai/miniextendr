//! Test: Rust `...` parses only as the last parameter, so a dots parameter with
//! another parameter after it gets an error that names the `&Dots` spelling (#1737).

use miniextendr_macros::miniextendr;

#[miniextendr]
pub fn named_not_last(sources: ..., dosing_type: i32) -> i32 {
    dosing_type
}

fn main() {}
