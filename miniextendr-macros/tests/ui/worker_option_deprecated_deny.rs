//! Compile-fail test for #2117: an explicit `worker` option names the
//! `#[deprecated]` `miniextendr_api::worker::WORKER_OPTION`, so the
//! `deprecated` lint fires at the option. Under `#![deny(deprecated)]` that
//! turns into a hard compile error. The `SEXP` argument keeps the call on
//! R's main thread, so the fixture doesn't need the `worker-thread` feature.

#![deny(deprecated)]

use miniextendr_api::{SEXP, miniextendr};

#[miniextendr(worker)]
pub fn on_worker(x: SEXP) -> SEXP {
    x
}

fn main() {}
