//! Compile-pass test for the three spellings of condition-call attribution
//! (#1566): a `Call` / `CallerCall` marker parameter, the
//! `call = wrapper | caller` attribute (path and string forms) and agreeing
//! combinations, with and without `no_preconditions`, which is independent of
//! the attribution. The marker is not an R formal: it is bound from the C
//! wrapper's hidden call slot, so the generated wrapper takes only `x`.

#![allow(dead_code)]

use miniextendr_api::{Call, CallerCall, miniextendr};

/// `Call` selects `wrapper` attribution; the body sees the wrapper's `sys.call()`.
#[miniextendr]
pub fn with_call(x: i32, call: Call) -> i32 {
    let _ = call.sexp();
    x
}

/// `CallerCall` selects `caller` attribution (needs `noexport` / `internal`).
#[miniextendr(noexport)]
pub fn with_caller_call_impl(x: i32, call: CallerCall) -> i32 {
    let _: Call = call.into();
    x
}

/// The marker may sit anywhere in the signature; the R formals skip it.
#[miniextendr(internal)]
pub fn marker_first(_call: CallerCall, x: i32) -> i32 {
    x
}

/// Marker and attribute agreeing is fine.
#[miniextendr(noexport, call = caller)]
pub fn agreeing_marker(x: i32, _call: CallerCall) -> i32 {
    x
}

#[miniextendr(call = wrapper)]
pub fn attr_wrapper(x: i32) -> i32 {
    x
}

#[miniextendr(noexport, call = caller)]
pub fn attr_caller_impl(x: i32) -> i32 {
    x
}

#[miniextendr(noexport, call = "caller")]
pub fn attr_caller_string_impl(x: i32) -> i32 {
    x
}

/// `no_preconditions` drops the R-side checks only; a marker still gets the call.
#[miniextendr(no_preconditions)]
pub fn unchecked_with_marker(x: i32, _call: Call) -> i32 {
    x
}

#[miniextendr(noexport, no_preconditions, call = caller)]
pub fn unchecked_caller_impl(x: i32) -> Result<i32, String> {
    Ok(x)
}

fn main() {}
