//! Test: `&Dots` and `LazyDots` in one signature.
//!
//! Both are R's `...`, and a function takes at most one (#1892).

use miniextendr_api::dots::Dots;
use miniextendr_api::{LazyDots, miniextendr};

#[miniextendr]
pub fn two_dots(forced: &Dots, lazy: LazyDots) -> i32 {
    let _ = (forced, lazy);
    0
}

fn main() {}
