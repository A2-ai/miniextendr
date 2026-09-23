//! Test: two condition-call markers on one function.
//!
//! The call slot is a single value, so at most one `Call` / `CallerCall`
//! parameter may receive it (#1566).

use miniextendr_macros::miniextendr;

#[miniextendr(noexport)]
pub fn twice(x: i32, _first: miniextendr_api::Call, _second: miniextendr_api::CallerCall) -> i32 {
    x
}

fn main() {}
