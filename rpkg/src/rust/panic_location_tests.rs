//! Fixtures for the Rust panic *source location* surfaced into R error messages.
//!
//! When a generic `panic!` becomes an R error, the framework folds the panic
//! hook's `Location` into the condition message as a trailing
//! `\n(at file:line)`.
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

// region: shared plain helper (the nested-panic origin)

/// Plain Rust helper — NOT `#[miniextendr]`, NOT `#[track_caller]`. The
/// nested-panic fixtures call this; the surfaced `(at …)` must point at the
/// `panic!` line inside THIS function, proving the location tracks the real
/// origin frame rather than the exported wrapper.
fn plain_boom() -> i32 {
    panic!("boom-nested")
}

// endregion

// region: panics

/// Direct panic. The R error message must carry
/// `(at panic_location_tests.rs:<panic line>)`.
///
/// @export
#[miniextendr]
pub fn panic_location_main_direct() -> i32 {
    panic!("boom-main-direct")
}

/// Nested panic — delegates to [`plain_boom`]. The location
/// must resolve into `plain_boom`, not this wrapper.
///
/// @export
#[miniextendr]
pub fn panic_location_main_nested() -> i32 {
    plain_boom()
}

// endregion

// region: regression guards (typed branches must stay location-free)

/// Regression guard: `error!()` produces a typed `rust_error` condition with NO
/// `(at …)` suffix (it travels the `RCondition` branch, untouched by the
/// location feature).
///
/// @export
#[miniextendr]
pub fn panic_location_regression_error() {
    miniextendr_api::error!("regression-error-no-location");
}

/// Regression guard: returning `Result::Err` gets NO `(at …)` suffix (it
/// travels the tagged-value return path, untouched by the location feature).
///
/// @export
#[miniextendr]
pub fn panic_location_regression_result_err() -> Result<i32, String> {
    Err("regression-result-err-no-location".to_string())
}

/// Regression guard: a required `Option::None` return raises the typed
/// NONE_ERR condition ("returned no value") with NO `(at …)` suffix.
/// `Option<()>` takes the raising path (unlike `Option<i32>`, whose `None`
/// maps to `NA`), so this actually exercises the untouched typed branch.
///
/// @export
#[miniextendr]
pub fn panic_location_regression_option_none() -> Option<()> {
    None
}

// endregion
