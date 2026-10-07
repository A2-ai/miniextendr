//! Test: `no_default` leaves a choice parameter's R formal without a default
//! (#1828), so a `default` on the same parameter contradicts it and must
//! error at compile time.

use miniextendr_macros::miniextendr;

#[miniextendr]
fn bad_no_default(#[miniextendr(match_arg, no_default, default = "\"a\"")] mode: String) {}

fn main() {}
