//! Test: `Option<LazyDots>`.
//!
//! The dots are always present (`len()` is 0 when the call passes none), so
//! `LazyDots` is taken by value, never wrapped (#1892).

use miniextendr_api::{LazyDots, miniextendr};

#[miniextendr]
pub fn optional_lazy_dots(rest: Option<LazyDots>) -> i32 {
    let _ = rest;
    0
}

fn main() {}
