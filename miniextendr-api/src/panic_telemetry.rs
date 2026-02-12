//! Structured panic telemetry for debugging Rust panics that become R errors.
//!
//! Three separate panic→R-error paths exist in miniextendr (worker thread, ALTREP
//! trampolines, and unwind_protect). This module provides a unified hook point
//! that fires before each panic is converted to an R error.
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
//! # Performance
//!
//! `fire()` takes a read lock (uncontended in normal use). The hook only fires
//! on panic paths, never on hot paths.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, RwLock};

/// Describes where a panic originated before being converted to an R error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanicSource {
    /// Panic on the worker thread (caught by `run_on_worker`).
    Worker,
    /// Panic inside an ALTREP trampoline (caught by `catch_altrep_panic`).
    Altrep,
    /// Panic inside `with_r_unwind_protect` (caught by `panic_payload_to_r_error`).
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

type Hook = Arc<dyn Fn(&PanicReport) + Send + Sync>;

static HOOK: RwLock<Option<Hook>> = RwLock::new(None);

/// Register a panic telemetry hook.
///
/// The hook is called with a [`PanicReport`] each time a Rust panic is about
/// to be converted into an R error. Only one hook can be active at a time;
/// calling this again replaces (and drops) the previous hook.
///
/// # Thread Safety
///
/// The hook may be called from any thread (worker thread, main R thread, etc.).
/// Ensure your closure is safe to call concurrently.
///
/// It is safe to call `set_panic_telemetry_hook` or `clear_panic_telemetry_hook`
/// from within a hook — the lock is released before the hook is invoked.
pub fn set_panic_telemetry_hook(f: impl Fn(&PanicReport) + Send + Sync + 'static) {
    let mut guard = HOOK.write().unwrap_or_else(|e| e.into_inner());
    *guard = Some(Arc::new(f));
}

/// Remove the current panic telemetry hook, if any.
pub fn clear_panic_telemetry_hook() {
    let mut guard = HOOK.write().unwrap_or_else(|e| e.into_inner());
    *guard = None;
}

/// Fire the telemetry hook if one is set.
///
/// Called internally at each panic→R-error conversion site.
///
/// The hook is cloned (as `Arc`) and the lock is dropped before invocation,
/// so the hook can safely call `set_panic_telemetry_hook` or
/// `clear_panic_telemetry_hook` without deadlocking. Secondary panics from
/// the hook are caught and silently suppressed.
pub(crate) fn fire(message: &str, source: PanicSource) {
    // Clone the Arc while holding the read lock, then drop the lock
    // before invoking. This prevents deadlock if the hook calls
    // set/clear_panic_telemetry_hook (which take a write lock).
    let hook = {
        let guard = HOOK.read().unwrap_or_else(|e| e.into_inner());
        guard.as_ref().cloned()
    };

    if let Some(hook) = hook {
        let report = PanicReport { message, source };
        // Suppress secondary panics from the hook — we're already on a
        // panic→R-error path and a double-panic would abort.
        let _ = catch_unwind(AssertUnwindSafe(|| hook(&report)));
    }
}
