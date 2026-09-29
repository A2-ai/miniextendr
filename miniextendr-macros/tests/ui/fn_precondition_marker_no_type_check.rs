//! Test: `Checked<SEXP>` is rejected: `SEXP` has no R-side type check to
//! keep or drop.

use miniextendr_api::miniextendr;

#[miniextendr]
pub fn raw(x: miniextendr_api::Checked<miniextendr_api::SEXP>) -> miniextendr_api::SEXP {
    *x
}

fn main() {}
