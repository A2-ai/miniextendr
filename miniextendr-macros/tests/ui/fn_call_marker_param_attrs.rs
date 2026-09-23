//! Test: per-parameter options on a `Call` marker.
//!
//! The marker is bound from the call slot, not from an R argument, so
//! `coerce` / `match_arg` / `choices` / `default` have nothing to act on (#1566).

use miniextendr_macros::miniextendr;

#[miniextendr]
pub fn marker_with_default(x: i32, #[miniextendr(default = "NULL")] _call: miniextendr_api::Call) -> i32 {
    x
}

fn main() {}
