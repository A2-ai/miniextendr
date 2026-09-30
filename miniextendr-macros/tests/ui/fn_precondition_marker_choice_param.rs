//! Test: a precondition marker on a `choices(...)` parameter is rejected:
//! `match.arg()` validates it, not the type checks.

use miniextendr_api::miniextendr;

#[miniextendr]
pub fn pick(#[miniextendr(choices("fast", "slow"))] mode: miniextendr_api::Unchecked<String>) -> String {
    mode.into_inner()
}

fn main() {}
