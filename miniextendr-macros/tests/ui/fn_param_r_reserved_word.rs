//! Test: a parameter whose R argument is an R reserved word is rejected.
//!
//! The R argument drops the leading underscores, so `_if` becomes `if`, and
//! `function(if)` does not parse: the whole wrappers file would fail to load.

use miniextendr_macros::miniextendr;

#[miniextendr]
pub fn pick(_if: bool) -> bool {
    true
}

fn main() {}
