//! Test: `dots = typed_list!(..)` on a `LazyDots` parameter.
//!
//! `typed_list!` validates the forced `list(...)` of a `&Dots` parameter;
//! `LazyDots` forces nothing (#1892).

use miniextendr_api::{LazyDots, miniextendr};

#[miniextendr(dots = typed_list!(x => numeric()))]
pub fn typed_lazy_dots(rest: LazyDots) -> i32 {
    let _ = rest;
    0
}

fn main() {}
