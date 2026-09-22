//! Test: a `Call` marker parameter on a `fast` function.
//!
//! `fast` (and `no_call_attribution`, `call = none`) emit `.call = NULL`; the
//! marker asks for the wrapper's `match.call()`. The two disagree (#1566).

use miniextendr_macros::miniextendr;

#[miniextendr(fast)]
pub fn fast_with_marker(x: i32, _call: miniextendr_api::Call) -> i32 {
    x
}

fn main() {}
