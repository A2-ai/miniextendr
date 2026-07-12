//! Fixtures for the Rust panic *source location* surfaced into R error messages.
//!
//! When a generic `panic!` becomes an R error, the framework folds the panic
//! hook's `Location` into the condition message as a trailing
//! `\n(at file:line)`. These fixtures drive that on both thread paths:
//!
//! - MAIN thread — the fn takes a `SEXP` argument, a hard main-thread
//!   requirement, so the panic + hook + `catch_unwind` all fire on main.
//! - WORKER thread — `#[miniextendr(worker)]` + no args, so the panic + hook
//!   fire on the worker; the location is folded once at the worker (origin)
//!   thread and the main side treats the message verbatim.
//!
//! Both a *direct* panic (`panic!` in the fn body) and a *nested* panic (the fn
//! calls the plain, non-`#[miniextendr]` helper [`plain_boom`]) are covered so
//! the location points at the real `panic!` site, never the generated wrapper
//! glue — the reason the automatic `#[track_caller]` was dropped (#1121).
//!
//! The regression fixtures prove the typed branches are untouched:
//! `error!()`, a `Result::Err` return, and an `Option::None` return carry NO
//! `(at …)` suffix. Tests live in `rpkg/tests/testthat/test-panic-location.R`.

use miniextendr_api::miniextendr;
use miniextendr_api::prelude::SEXP;

// region: shared plain helper (the nested-panic origin)

/// Plain Rust helper — NOT `#[miniextendr]`, NOT `#[track_caller]`. The
/// nested-panic fixtures call this; the surfaced `(at …)` must point at the
/// `panic!` line inside THIS function, proving the location tracks the real
/// origin frame rather than the exported wrapper.
fn plain_boom() -> i32 {
    panic!("boom-nested")
}

// endregion

// region: MAIN-thread panics (SEXP arg forces main-thread dispatch)

/// Direct panic on the MAIN thread. The R error message must carry
/// `(at panic_location_tests.rs:<panic line>)`.
///
/// @param x Ignored; present only to force main-thread dispatch.
/// @export
#[miniextendr]
pub fn panic_location_main_direct(_x: SEXP) -> i32 {
    panic!("boom-main-direct")
}

/// Nested panic on the MAIN thread — delegates to [`plain_boom`]. The location
/// must resolve into `plain_boom`, not this wrapper.
///
/// @param x Ignored; present only to force main-thread dispatch.
/// @export
#[miniextendr]
pub fn panic_location_main_nested(_x: SEXP) -> i32 {
    plain_boom()
}

// endregion

// region: WORKER-thread panics (no args + worker opt-in)

/// Direct panic on the WORKER thread. Exercises the worker→main path where the
/// location is folded at the worker (origin) before the message crosses back.
///
/// @export
#[miniextendr(worker)]
pub fn panic_location_worker_direct() -> i32 {
    panic!("boom-worker-direct")
}

/// Nested panic on the WORKER thread — delegates to [`plain_boom`].
///
/// @export
#[miniextendr(worker)]
pub fn panic_location_worker_nested() -> i32 {
    plain_boom()
}

/// Panic inside a `with_r_thread` closure invoked from the WORKER thread
/// (#1245 Gap 1). The closure panics on the MAIN thread (where
/// `with_r_thread` routes it), then the worker re-panics to unwind out of
/// `run_on_worker` — the R error must carry the location of THIS file's
/// `panic!`, never `worker.rs` (the worker's own relay call site).
///
/// @export
#[miniextendr(worker)]
pub fn panic_location_worker_with_r_thread() -> i32 {
    miniextendr_api::with_r_thread(|| -> i32 { panic!("boom-with-r-thread") })
}

// endregion

// region: regression guards (typed branches must stay location-free)

/// Regression guard: `error!()` produces a typed `rust_error` condition with NO
/// `(at …)` suffix (it travels the `RCondition` branch, untouched by the
/// location feature). Takes a `SEXP` arg to stay on the main-thread typed path.
///
/// @param x Ignored; present only to force main-thread dispatch.
/// @export
#[miniextendr]
pub fn panic_location_regression_error(_x: SEXP) {
    miniextendr_api::error!("regression-error-no-location");
}

/// Regression guard: returning `Result::Err` gets NO `(at …)` suffix (it
/// travels the tagged-value return path, untouched by the location feature).
///
/// @param x Ignored; present only to force main-thread dispatch.
/// @export
#[miniextendr]
pub fn panic_location_regression_result_err(_x: SEXP) -> Result<i32, String> {
    Err("regression-result-err-no-location".to_string())
}

/// Regression guard: a required `Option::None` return raises the typed
/// NONE_ERR condition ("returned no value") with NO `(at …)` suffix.
/// `Option<()>` takes the raising path (unlike `Option<i32>`, whose `None`
/// maps to `NA`), so this actually exercises the untouched typed branch.
///
/// @param x Ignored; present only to force main-thread dispatch.
/// @export
#[miniextendr]
pub fn panic_location_regression_option_none(_x: SEXP) -> Option<()> {
    None
}

// endregion
