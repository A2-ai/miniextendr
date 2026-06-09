//! Fixtures for `#[miniextendr(error_direct)]` (issue #665).
//!
//! With `error_direct`, error-shaped failures (`panic!()` / `error!()` /
//! `RCondition::Error`) are raised **directly from C** via
//! `Rf_eval(stop(structure(...)))`, skipping the R-side
//! `.miniextendr_raise_condition` → `stop(structure(...))` re-raise. The C-built
//! condition object must carry the same `rust_*` class layering the R-side path
//! produces, so a `tryCatch` over these fixtures sees identical classes.
//!
//! Warnings / messages / generic conditions still fall back to the tagged-SEXP
//! path (R-side raise), since `stop()` only does errors.
//!
//! Tests live in `rpkg/tests/testthat/test-error-direct.R`.

use miniextendr_api::miniextendr;

type RCondition = miniextendr_api::condition::RCondition;

// region: error_direct — error-shaped (raised directly from C)

/// Raise a `rust_error` directly from C via `error!()`.
///
/// Class layering must match `.miniextendr_raise_condition`'s `error` arm:
/// `c("rust_error", "simpleError", "error", "condition")`.
///
/// @export
#[miniextendr(error_direct)]
pub fn demo_error_direct(msg: &str) {
    miniextendr_api::error!("{msg}");
}

/// No-arg variant for the gctorture sweep (exercises the C-side condition build
/// + `stop()` longjmp on the error path).
///
/// @export
#[miniextendr(error_direct)]
pub fn demo_error_direct_fixed() {
    miniextendr_api::error!("error_direct fixed message");
}

/// Raise a `rust_error` with a custom class prepended, directly from C.
///
/// Class layering must match `.miniextendr_raise_condition`'s `error` arm with
/// a custom class: `c(<class>, "rust_error", "simpleError", "error", "condition")`.
///
/// @export
#[miniextendr(error_direct)]
pub fn demo_error_direct_custom_class(class: &str, msg: &str) {
    std::panic::panic_any(RCondition::Error {
        message: msg.to_string(),
        class: Some(class.to_string()),
    });
}

/// Raise via a plain `panic!()` (kind = "panic") directly from C.
///
/// Class layering must match `.miniextendr_raise_condition`'s `panic` arm:
/// `c("rust_error", "simpleError", "error", "condition")`.
///
/// @export
#[miniextendr(error_direct)]
pub fn demo_panic_direct(msg: &str) {
    panic!("{msg}");
}

// endregion

// region: error_direct — non-error signals (fall back to tagged-SEXP path)

/// `warning!()` under `error_direct` — falls back to the R-side raise so the
/// warning is signalled (not raised as an error). Class layering must match
/// `.miniextendr_raise_condition`'s `warning` arm.
///
/// @export
#[miniextendr(error_direct)]
pub fn demo_warning_direct(msg: &str) {
    miniextendr_api::warning!("{msg}");
}

/// `message!()` under `error_direct` — falls back to the R-side raise so the
/// message is emitted (not raised as an error).
///
/// @export
#[miniextendr(error_direct)]
pub fn demo_message_direct(msg: &str) {
    miniextendr_api::message!("{msg}");
}

// endregion

// region: control — same fixture without error_direct (R-side raise)

/// Control fixture: same as the error_direct variant but on the default path.
///
/// Identical to `demo_error_direct` but without `error_direct`, so the error
/// travels the default tagged-SEXP, R-side `.miniextendr_raise_condition` path.
/// Used to prove the two paths produce the same `tryCatch`-visible classes.
///
/// @export
#[miniextendr]
pub fn demo_error_indirect(msg: &str) {
    miniextendr_api::error!("{msg}");
}

// endregion
