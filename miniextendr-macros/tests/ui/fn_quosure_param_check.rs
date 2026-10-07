//! Test: an R-side check on a `Quosure` parameter.
//!
//! `inherits(cols, ...)` in the wrapper would force the argument, which the
//! wrapper passes to `rlang::enquo()` unevaluated (#1835).

use miniextendr_macros::miniextendr;

#[miniextendr]
pub fn quosure_with_check(#[miniextendr(inherits = "data.frame")] cols: miniextendr_api::Quosure) -> i32 {
    let _ = cols;
    0
}

fn main() {}
