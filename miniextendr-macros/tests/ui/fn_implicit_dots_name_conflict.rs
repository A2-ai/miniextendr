//! Test: unnamed dots (`_: ...`) bind `__miniextendr_dots`, so a parameter of
//! that name is refused.

use miniextendr_macros::miniextendr;

#[miniextendr]
pub fn conflicting_name(__miniextendr_dots: i32, _: ...) -> i32 {
    __miniextendr_dots
}

fn main() {}
