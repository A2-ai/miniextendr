//! #1991: generated wrappers resolve their own standard-library names. This
//! module shadows them, so it compiles only if every emitted reference is
//! absolute. The UI fixture `tests/ui/pass/shadowed_standard_names.rs` covers
//! the other paths. A worker wrapper needs the destination crate's
//! `worker-thread` feature, and the Vctrs derive miniextendr-api's `vctrs`
//! feature, which the UI tests' crate has neither of.
#![allow(dead_code)]

use miniextendr_api::miniextendr;

struct Some;
struct None;
struct Ok;
struct Err;
struct Option;
struct Result;
struct Box;
fn drop(_: u8) {}

/// Double `x` on the worker thread.
/// @param x An integer.
#[cfg(any(feature = "worker-thread", feature = "worker-default"))]
#[miniextendr(worker, noexport)]
pub fn shadowed_std_names_worker(x: i32) -> ::core::result::Result<i32, String> {
    if x < 0 {
        ::core::result::Result::Err(format!("negative: {x}"))
    } else {
        ::core::result::Result::Ok(x * 2)
    }
}

/// A vctrs class derived with the standard names shadowed.
#[cfg(feature = "vctrs")]
#[derive(miniextendr_api::Vctrs)]
#[vctrs(class = "shadowed_percent", base = "double")]
pub struct ShadowedPercent {
    #[vctrs(data)]
    values: Vec<f64>,
}
