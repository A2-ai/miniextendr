//! Test: `&LazyDots`.
//!
//! `LazyDots` is R's `...` passed unforced; the wrapper passes the frame only
//! when it is the parameter's whole type, taken by value (#1892).

use miniextendr_api::{LazyDots, miniextendr};

#[miniextendr]
pub fn borrowed_lazy_dots(rest: &LazyDots) -> i32 {
    let _ = rest;
    0
}

fn main() {}
