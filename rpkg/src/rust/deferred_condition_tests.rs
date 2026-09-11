//! Deferred conditions (#1448): a `#[miniextendr]` call signals a warning,
//! message or condition *and* returns its value.
//!
//! - `defer_warning(payload)` / `defer_message` / `defer_condition` take any
//!   `RConditionError` payload, here a `#[derive(RConditionError)]` enum and
//!   struct; the `defer_*!` macros take the `warning!` grammar.
//! - Signalling happens when the wrapper returns, in queue order, before the
//!   value (or the error) reaches R; `conditionCall()` is the wrapper's call.
//! - The rng, worker-thread and s3-method fixtures exercise the other C
//!   wrapper templates and the class-wrapping R side.

use miniextendr_api::condition::{RConditionError, RError};
use miniextendr_api::{
    TryFromSexp as _, defer_condition, defer_message, defer_warning, miniextendr, rust_error,
};

// region: Typed payloads

/// Warnings a package attaches to a result. `message` formats the fields, so
/// no `Display` impl is needed; classes default to `pkg_warning_<variant>`
/// plus the family `pkg_warning`.
#[derive(Debug, RConditionError)]
#[condition(class = "pkg_warning")]
pub enum PkgWarning {
    #[condition(message = "dropped {dropped} of {total} rows")]
    Truncated { dropped: i32, total: i32 },
    #[condition(message = "column `{column}` coerced to {to}")]
    Coerced {
        column: String,
        to: String,
        #[condition(skip)]
        detail: Vec<u8>,
    },
    #[condition(class = "pkg_warning_slow_path", message = "took the slow path")]
    Slow,
}

/// A struct payload: the message comes from `Display`, `attempts` is renamed
/// and `range` rides along as its `Debug` rendering.
#[derive(Debug, RConditionError)]
pub struct RetryNotice {
    #[condition(rename = "n_attempts")]
    attempts: i32,
    #[condition(debug)]
    range: std::ops::RangeInclusive<i32>,
}

impl std::fmt::Display for RetryNotice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "succeeded after {} attempts", self.attempts)
    }
}

// endregion

// region: Free functions (main-thread template)

/// Returns `n - 2` and warns about the two dropped rows.
/// @param n Row count.
#[miniextendr]
pub fn deferred_warning_value(n: i32) -> i32 {
    defer_warning(PkgWarning::Truncated {
        dropped: 2,
        total: n,
    });
    n - 2
}

/// Three conditions in one call, two typed and one from the macro form, to
/// check queue order and mixed kinds. Returns 3.
#[miniextendr]
pub fn deferred_mixed_order() -> i32 {
    defer_warning(PkgWarning::Slow);
    defer_message!(data = ("step", 2), "step {} of 3", 2);
    defer_warning!(
        class = "pkg_warning_final",
        data = { code = 7 },
        "final {}",
        "warning"
    );
    3
}

/// `defer_message` with a struct payload (Display message, renamed and
/// Debug fields). Returns `"done"`.
#[miniextendr]
pub fn deferred_message_struct() -> String {
    defer_message(RetryNotice {
        attempts: 3,
        range: 1..=5,
    });
    "done".to_string()
}

/// `defer_condition!` with a custom class: silent without a handler.
#[miniextendr]
pub fn deferred_condition_value() -> f64 {
    defer_condition!(class = "pkg_progress", data = ("pct", 50.0), "halfway");
    0.5
}

/// `defer_condition` with a typed payload on a plain condition.
#[miniextendr]
pub fn deferred_condition_typed() -> i32 {
    defer_condition(RError::new("audit").class("pkg_audit").data("rows", 4));
    4
}

/// A deferred warning followed by a classed `Err`: the warning is signalled
/// first, then the error.
/// @param fail Whether to return `Err`.
#[miniextendr]
pub fn deferred_then_error(fail: bool) -> Result<i32, RError> {
    defer_warning(PkgWarning::Coerced {
        column: "x".into(),
        to: "double".into(),
        detail: vec![1, 2],
    });
    if fail {
        return Err(RError::new("conversion failed").class(["pkg_failed", "pkg_error"]));
    }
    Ok(1)
}

/// A deferred warning followed by `rust_error!` (the panic transport).
#[miniextendr]
pub fn deferred_then_panic() {
    defer_warning!("about to fail");
    rust_error!(class = "pkg_boom", "boom");
}

/// A deferred warning followed by an immediate `warning!` (which aborts the
/// call): both arrive, deferred first, and the value is `NULL`.
#[miniextendr]
pub fn deferred_then_warning_abort() -> i32 {
    defer_warning!("queued");
    miniextendr_api::warning!("immediate");
}

/// Unit return: R gets `invisible(NULL)` plus the warning.
#[miniextendr]
pub fn deferred_unit() {
    defer_warning!("side effect only");
}

/// The `rng` arm of the main-thread template (Get/PutRNGstate around the
/// body). Returns 1.
#[miniextendr(rng)]
pub fn deferred_rng_value() -> i32 {
    defer_warning!("rng arm");
    1
}

/// Nested calls: evaluates an R closure that itself calls
/// `deferred_warning_value()`; the inner call flushes only its own warning,
/// this call flushes only "outer".
/// @param f An R function of no arguments.
#[miniextendr]
pub fn deferred_nested(f: miniextendr_api::SEXP) -> i32 {
    use miniextendr_api::expression::RCall;
    use miniextendr_api::{OwnedProtect, sys};
    defer_warning!(class = "pkg_outer", "outer");
    // Plain `Rf_eval` rather than `RCall::eval` (`R_tryEval` runs under
    // `R_ToplevelExec`, which hides the caller's handlers), so the inner
    // call's warning reaches the same `withCallingHandlers` as this one's.
    let inner = unsafe {
        let call = OwnedProtect::new(RCall::from_sexp(f).build());
        sys::Rf_eval(call.get(), sys::R_GlobalEnv)
    };
    i32::try_from_sexp(inner).expect("integer") + 100
}

// endregion

// region: Worker-thread template

#[cfg(feature = "worker-thread")]
mod worker {
    use super::PkgWarning;
    use miniextendr_api::{defer_warning, miniextendr};

    /// Worker-thread template: the push happens on the worker, the signal on
    /// the main thread after the result is back. Returns 9.
    #[miniextendr(worker)]
    pub fn deferred_worker_value() -> i32 {
        defer_warning(PkgWarning::Truncated {
            dropped: 1,
            total: 10,
        });
        9
    }
}

// endregion

// region: s3 methods (class-wrapping R side)

/// A handle whose methods defer conditions: the class-method wrappers must
/// still wrap the returned value / handle.
#[derive(miniextendr_api::ExternalPtr)]
pub struct DeferredCounter {
    count: i32,
}

#[miniextendr(s3)]
impl DeferredCounter {
    /// A counter starting at zero.
    pub fn new() -> Self {
        DeferredCounter { count: 0 }
    }

    /// Nudge the counter up by one; warns once it passes `limit`.
    /// @param limit Warn above this value.
    pub fn nudge(&mut self, limit: i32) -> i32 {
        self.count += 1;
        if self.count > limit {
            defer_warning!(
                class = "pkg_counter_high",
                data = ("count", self.count),
                "count {} above {limit}",
                self.count
            );
        }
        self.count
    }

    /// A copy of the counter (the `Self`-returning arm) with a deferred message.
    pub fn fork(&self) -> Self {
        defer_message!("forked");
        DeferredCounter { count: self.count }
    }
}

// endregion
