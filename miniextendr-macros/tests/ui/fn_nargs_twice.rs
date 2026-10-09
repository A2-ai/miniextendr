//! Test: two `NArgs` parameters on one function.
//!
//! Both would receive the same `nargs()`, so a function takes at most one
//! (#1860).

use miniextendr_api::miniextendr;

#[miniextendr]
pub fn counted_twice(x: i32, n: miniextendr_api::NArgs, again: miniextendr_api::NArgs) -> i32 {
    let _ = (n, again);
    x
}

fn main() {}
