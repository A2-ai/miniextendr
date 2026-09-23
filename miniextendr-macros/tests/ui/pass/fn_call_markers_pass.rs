//! Compile-pass test for the three spellings of condition-call attribution
//! (#1566): a `Call` / `CallerCall` marker parameter, the
//! `call = none | wrapper | caller` attribute (path and string forms, plus the
//! `no_call_attribution` / `fast` / `no_fast` shorthands) and agreeing
//! combinations. The marker is not an R formal: it is bound from the C
//! wrapper's hidden call slot, so the generated wrapper takes only `x`.

#![allow(dead_code)]

use miniextendr_api::{Call, CallerCall, miniextendr};

/// `Call` selects `wrapper` attribution; the body sees the wrapper's `match.call()`.
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

#[miniextendr(call = none)]
pub fn attr_none(x: i32) -> i32 {
    x
}

#[miniextendr(call = "none", no_call_attribution)]
pub fn attr_none_string_and_shorthand(x: i32) -> i32 {
    x
}

#[miniextendr(fast, call = none)]
pub fn attr_fast_agrees(x: i32) -> i32 {
    x
}

#[miniextendr(no_fast, call = wrapper)]
pub fn attr_no_fast_agrees(x: i32) -> i32 {
    x
}

fn main() {}
