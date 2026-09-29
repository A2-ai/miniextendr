//! Test: Rust `...` next to an explicit `&Dots` parameter is a second `...`.

use miniextendr_macros::miniextendr;

#[miniextendr]
pub fn both_dots(rest: &miniextendr_api::dots::Dots, x: i32, more: ...) -> i32 {
    let _ = (rest, more);
    x
}

fn main() {}
