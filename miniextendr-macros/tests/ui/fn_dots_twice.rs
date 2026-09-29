//! Test: a function takes at most one `...`, so two `&Dots` parameters are refused.

use miniextendr_macros::miniextendr;

#[miniextendr]
pub fn two_dots(
    x: i32,
    a: &miniextendr_api::dots::Dots,
    b: &miniextendr_api::dots::Dots,
) -> i32 {
    let _ = (a, b);
    x
}

fn main() {}
