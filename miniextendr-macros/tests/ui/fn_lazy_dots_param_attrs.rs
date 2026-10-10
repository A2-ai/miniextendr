//! Test: a per-parameter option on a `LazyDots` parameter.
//!
//! The parameter is R's `...`, which the wrapper passes unforced; an option
//! would force it or has nothing to act on (#1892).

use miniextendr_api::{LazyDots, miniextendr};

#[miniextendr]
pub fn coerced_lazy_dots(x: i32, #[miniextendr(coerce)] rest: LazyDots) -> i32 {
    let _ = rest;
    x
}

fn main() {}
