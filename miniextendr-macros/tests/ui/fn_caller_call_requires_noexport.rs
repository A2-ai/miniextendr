//! Test: a `CallerCall` parameter on an exported function.
//!
//! Like `call = caller`, the marker attributes conditions to the wrapper's
//! caller, which only makes sense for a package-internal entry point behind a
//! hand-written R function (#1566).

use miniextendr_macros::miniextendr;

#[miniextendr]
pub fn exported_with_caller_call(x: i32, _call: miniextendr_api::CallerCall) -> i32 {
    x
}

fn main() {}
