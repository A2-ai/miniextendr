//! Conditions raised without a call (#1725): `call = none` on the condition
//! macros, `RError::without_call()` and `#[condition(call = none)]`, the
//! per-condition equivalent of R's `warning(..., call. = FALSE)`.
//!
//! Each fixture pairs a call-less condition with a sibling from the same
//! function that keeps the wrapper's call, so the tests tell the opt-out from
//! a call that is missing for another reason. Tests live in
//! `rpkg/tests/testthat/test-callless-conditions.R`.

use miniextendr_api::condition::{RConditionError, RError};
use miniextendr_api::{
    Call, defer_condition, defer_message, defer_warning, miniextendr, rust_condition, rust_error,
    warning,
};

// region: Typed payloads

/// Warnings about a table of user overrides: `Overridden` is about the rows,
/// not the call, so it carries none; `Settled` keeps the call.
#[derive(Debug, RConditionError)]
#[condition(class = "pkg_override")]
pub enum OverrideWarning {
    #[condition(
        call = none,
        message = "row {row} overrides an earlier row for profile {profile}"
    )]
    Overridden { row: i32, profile: i32 },
    #[condition(message = "{n} rows settled")]
    Settled { n: i32 },
}

/// A notice that is call-less for the whole type.
#[derive(Debug, RConditionError)]
#[condition(class = "pkg_notice", call = none, message = "settled {n} rows")]
pub struct SettledNotice {
    n: i32,
}

/// Row errors: `Malformed` is call-less, `Missing` keeps the call.
#[derive(Debug, RConditionError)]
#[condition(class = "pkg_row_error")]
pub enum RowError {
    #[condition(call = none, message = "row {row} is malformed")]
    Malformed { row: i32 },
    #[condition(message = "row {row} is missing")]
    Missing { row: i32 },
}

// endregion

// region: Immediate conditions (main-thread template)

/// An immediate warning about the data, raised without a call.
/// @param callless Whether to drop the call (`FALSE` keeps the wrapper's call).
#[miniextendr]
pub fn callless_warning(callless: bool) {
    if callless {
        warning!(
            call = none,
            class = "pkg_override",
            data = { row = 2, profile = 3 },
            "row 2 overrides an earlier row for profile 3"
        );
    }
    warning!(
        class = "pkg_override",
        data = { row = 2, profile = 3 },
        "row 2 overrides an earlier row for profile 3"
    );
}

/// An immediate error, raised without a call.
/// @param callless Whether to drop the call (`FALSE` keeps the wrapper's call).
#[miniextendr]
pub fn callless_error(callless: bool) {
    if callless {
        rust_error!(
            call = none,
            class = ["pkg_bad_row", "pkg_error"],
            data = { row = 4 },
            "row 4 is malformed"
        );
    }
    rust_error!(
        class = ["pkg_bad_row", "pkg_error"],
        data = { row = 4 },
        "row 4 is malformed"
    );
}

/// An immediate plain condition, signalled without a call.
/// @param callless Whether to drop the call (`FALSE` keeps the wrapper's call).
#[miniextendr]
pub fn callless_condition(callless: bool) {
    if callless {
        rust_condition!(
            call = none,
            class = "pkg_progress",
            data = { pct = 50 },
            "halfway"
        );
    }
    rust_condition!(class = "pkg_progress", data = { pct = 50 }, "halfway");
}

// endregion

// region: Deferred conditions

/// Five deferred conditions and the value: a call-less macro warning, a
/// call-less derived variant, a sibling variant that keeps the call, a
/// call-less derived struct (`defer_condition`) and a call-less `RError`.
/// Returns `n`.
/// @param n Row count.
#[miniextendr]
pub fn callless_deferred(n: i32) -> i32 {
    defer_warning!(
        call = none,
        class = "pkg_override",
        data = { row = 2 },
        "row 2 overrides an earlier row"
    );
    defer_warning(OverrideWarning::Overridden { row: 5, profile: 3 });
    defer_warning(OverrideWarning::Settled { n });
    defer_condition(SettledNotice { n });
    defer_warning(
        RError::new("rows were reordered")
            .class("pkg_reorder")
            .without_call(),
    );
    n
}

/// A deferred message with and without `call = none`: a message carries no
/// call either way. Returns 1.
#[miniextendr]
pub fn callless_deferred_message() -> i32 {
    defer_message!(call = none, class = "pkg_note", "noted without a call");
    defer_message!(class = "pkg_note", "noted");
    1
}

// endregion

// region: Result errors

/// A `Result` error built with `RError::without_call()`.
/// @param callless Whether to drop the call (`FALSE` keeps the wrapper's call).
#[miniextendr]
pub fn callless_result(callless: bool) -> Result<i32, RError> {
    let err = RError::new("profile 3 has no rows")
        .class(["pkg_empty_profile", "pkg_error"])
        .data("profile", 3);
    Err(if callless { err.without_call() } else { err })
}

/// A derived `Result` error whose `Malformed` variant is call-less.
/// @param malformed `TRUE` raises the call-less `Malformed`, `FALSE` the
///   `Missing` sibling, which keeps the call.
#[miniextendr]
pub fn callless_result_derived(malformed: bool) -> Result<i32, RowError> {
    Err(if malformed {
        RowError::Malformed { row: 7 }
    } else {
        RowError::Missing { row: 7 }
    })
}

// endregion

// region: Raising guard (the ALTREP `RUnwind` transport)

/// `with_r_unwind_protect_or_raise`, the raising guard ALTREP `RUnwind`
/// callbacks use, given this wrapper's call: a call-less error drops it.
/// @param callless Whether to drop the call (`FALSE` keeps the guard's call).
#[miniextendr]
pub fn callless_raise_guard(callless: bool, call: Call) -> i32 {
    miniextendr_api::unwind_protect::with_r_unwind_protect_or_raise(
        || -> i32 {
            if callless {
                rust_error!(
                    call = none,
                    class = "pkg_guard",
                    data = { step = 1 },
                    "guard error"
                );
            }
            rust_error!(class = "pkg_guard", data = { step = 1 }, "guard error")
        },
        Some(call.sexp()),
    )
}

// endregion

// region: Worker-thread template

#[cfg(feature = "worker-thread")]
mod worker {
    use super::OverrideWarning;
    use miniextendr_api::{defer_warning, miniextendr, rust_error, warning};

    /// Worker-thread template, deferred: a call-less warning and a sibling
    /// that keeps the call, queued on the worker and signalled on the main
    /// thread. Returns 7.
    #[miniextendr(worker)]
    pub fn callless_worker_deferred() -> i32 {
        defer_warning!(call = none, class = "pkg_override", "worker data warning");
        defer_warning(OverrideWarning::Settled { n: 7 });
        7
    }

    /// Worker-thread template, immediate: the warning raised on the worker
    /// reaches R with its class, data and call choice.
    /// @param callless Whether to drop the call (`FALSE` keeps the wrapper's call).
    #[miniextendr(worker)]
    pub fn callless_worker_warning(callless: bool) {
        if callless {
            warning!(
                call = none,
                class = "pkg_override",
                data = { row = 1 },
                "worker warning"
            );
        }
        warning!(class = "pkg_override", data = { row = 1 }, "worker warning");
    }

    /// Worker-thread template, immediate error with a class vector and data.
    /// @param callless Whether to drop the call (`FALSE` keeps the wrapper's call).
    #[miniextendr(worker)]
    pub fn callless_worker_error(callless: bool) {
        if callless {
            rust_error!(
                call = none,
                class = ["pkg_worker_error", "pkg_error"],
                data = { code = 7 },
                "worker error"
            );
        }
        rust_error!(
            class = ["pkg_worker_error", "pkg_error"],
            data = { code = 7 },
            "worker error"
        );
    }
}

// endregion
