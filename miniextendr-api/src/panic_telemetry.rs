//! Structured panic telemetry for debugging Rust panics that become R errors.
//!
//! Three separate panic→R-error paths exist in miniextendr (ALTREP trampolines,
//! connection callbacks, and unwind_protect). This module provides a unified
//! hook point that fires before each panic is converted to an R error.
//!
//! # Relationship to the three emission paths
//!
//! This module is **observability, not emission**. It does not raise R
//! conditions and does not change how panics become errors; it only lets a
//! caller observe them. The three actual emission paths are documented on
//! [`crate::error_value`]:
//!
//! - `panic!(msg)` — escape hatch; tagged-SEXP with `kind = "panic"`.
//! - [`crate::error!`] / [`crate::warning!`] / [`crate::message!`] /
//!   [`crate::condition!`] — typed conditions via `RCondition` panic payloads.
//! - `Result<_, E: std::error::Error>` with [`crate::condition::AsRError`] —
//!   value-style propagation; tagged-SEXP with `kind = "result_err"`.
//!
//! `fire` is invoked from each catch site *before* the panic message is encoded
//! into a tagged condition value, so a registered hook sees every Rust-origin
//! failure regardless of which of the three paths produced it. Typical uses:
//! routing to `tracing` / `log`, capturing stack snapshots for crash reports,
//! or surfacing the panic message in test harnesses where R's stderr is
//! buffered. The hook must not raise R conditions itself (it runs inside the
//! panic-handling path) and must not panic — secondary panics from the hook
//! are caught and silently suppressed by `fire` (this crate, `pub(crate)`).
//!
//! # Usage
//!
//! ```ignore
//! use miniextendr_api::panic_telemetry::{set_panic_telemetry_hook, PanicReport, PanicSource};
//!
//! set_panic_telemetry_hook(|report| {
//!     eprintln!("[{:?}] panic: {}", report.source, report.message);
//! });
//! ```
//!
//! # Threads
//!
//! Every panic→R-error boundary runs on R's main thread, so the hook lives
//! there: a thread-local slot, set and fired on that thread, and the closure
//! need not be `Send` or `Sync`. A hook set on another thread stays in that
//! thread's slot and never sees the main thread's panics.
//!
//! # Performance
//!
//! `fire()` reads a thread-local slot and, with a hook set, clones its `Rc`.
//! The hook only fires on panic paths, never on hot paths.

use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

/// Describes where a panic originated before being converted to an R error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanicSource {
    /// Panic inside an ALTREP trampoline (caught by `catch_altrep_panic`).
    Altrep,
    /// Panic inside `with_r_unwind_protect` (caught by `with_r_unwind_protect_sourced`).
    UnwindProtect,
    /// Panic inside a connection callback trampoline.
    Connection,
}

/// A structured panic report passed to the telemetry hook.
pub struct PanicReport<'a> {
    /// The panic message (extracted from the panic payload).
    pub message: &'a str,
    /// Which panic→R-error boundary caught this panic.
    pub source: PanicSource,
}

type Hook = Rc<dyn Fn(&PanicReport)>;

std::thread_local! {
    static HOOK: RefCell<Option<Hook>> = const { RefCell::new(None) };
}

/// Register a panic telemetry hook on R's main thread.
///
/// The hook is called with a [`PanicReport`] each time a Rust panic is about
/// to be converted into an R error. Only one hook can be active at a time;
/// calling this again replaces (and drops) the previous hook.
///
/// Call it on R's main thread, where every panic→R-error boundary runs (see
/// [Threads](self#threads)). It is safe to call `set_panic_telemetry_hook` or
/// `clear_panic_telemetry_hook` from within a hook: the slot is not borrowed
/// while the hook runs.
pub fn set_panic_telemetry_hook(f: impl Fn(&PanicReport) + 'static) {
    // `replace` hands the previous hook back after the borrow ends, so its
    // destructor may set or clear the hook too.
    drop(HOOK.replace(Some(Rc::new(f))));
}

/// Remove the current panic telemetry hook, if any.
pub fn clear_panic_telemetry_hook() {
    drop(HOOK.take());
}

/// Fire the telemetry hook if one is set.
///
/// Called internally at each panic→R-error conversion site.
///
/// The hook is cloned (as `Rc`) out of the slot before it runs, so it can
/// call `set_panic_telemetry_hook` or `clear_panic_telemetry_hook`. Secondary
/// panics from the hook are caught and silently suppressed.
pub(crate) fn fire(message: &str, source: PanicSource) {
    let hook = HOOK.with_borrow(Option::clone);

    if let Some(hook) = hook {
        let report = PanicReport { message, source };
        // Suppress secondary panics from the hook — we're already on a
        // panic→R-error path and a double-panic would abort.
        let _ = catch_unwind(AssertUnwindSafe(|| hook(&report)));
    }
}

#[cfg(test)]
mod tests;
