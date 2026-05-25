//! Unified FFI guard for catching panics at Rust-R boundaries.
//!
//! Four modules independently catch panics at FFI boundaries: `worker.rs`,
//! `altrep_bridge.rs`, `unwind_protect.rs`, and `connection.rs`. This module
//! extracts the common pattern into a single `guarded_ffi_call` function.
//!
//! Most user code never calls anything here. The proc-macro layer
//! (`#[miniextendr]`) inserts the right guard at every Rust → R boundary it
//! generates. Reach for these helpers when you're writing a callback or
//! trampoline that the macros don't already cover (custom connections,
//! manual ALTREP, raw FFI shims).
//!
//! ## Guard Modes
//!
//! - [`GuardMode::CatchUnwind`]: Wraps the closure in `catch_unwind`. On panic,
//!   fires telemetry and raises an R error via `Rf_error` (diverges).
//!   Used by worker and connection trampolines.
//!
//! - [`GuardMode::RUnwind`]: Uses `R_UnwindProtect` to catch both Rust panics
//!   and R longjmps. Used by ALTREP callbacks that call R APIs. Routes
//!   through `crate::unwind_protect::with_r_unwind_protect_sourced`
//!   (crate-private).
//!
//! The ALTREP-specific `Unsafe` mode (no protection at all) stays in
//! `altrep_bridge.rs` since it has no general applicability.
//!
//! ## Tradeoffs vs raising R errors directly
//!
//! Don't reach for `Rf_error` / `Rf_errorcall` to fail out of a callback —
//! the longjmp skips Rust destructors and the lint **MXL300** rejects it.
//! Panic instead; whichever guard mode you pick converts the panic into the
//! tagged-condition transport ([`crate::error_value`]) or, on the ALTREP
//! `RUnwind` path, raises a structured `rust_*` condition via the
//! crate-private `raise_rust_condition_via_stop` helper.
//!
//! ## Cross references
//!
//! - [`crate::worker::with_r_thread`] — main-thread routing entry point.
//! - [`crate::unwind_protect::with_r_unwind_protect`] — the user-facing R
//!   error catcher; consumed by `RUnwind` mode.
//! - `docs/FFI_GUARD.md` — guard taxonomy and panic flow.

use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::panic_telemetry::PanicSource;
use crate::unwind_protect::panic_payload_to_string;

/// FFI guard mode controlling how panics are caught at Rust-R boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardMode {
    /// `catch_unwind` only. On panic: fire telemetry, then `Rf_error` (diverges).
    ///
    /// Use when R longjmps cannot occur (the closure does not call R APIs).
    CatchUnwind,
    /// `R_UnwindProtect`. Catches both Rust panics and R longjmps.
    ///
    /// Use when the closure may call R APIs that can error.
    RUnwind,
}

/// Execute `f` inside an FFI guard selected by `mode`.
///
/// On panic:
/// - Extracts the panic message from the payload.
/// - Fires [`crate::panic_telemetry`] with `source`.
/// - For [`GuardMode::CatchUnwind`]: raises R error via `Rf_error` (diverges — never returns).
/// - For [`GuardMode::RUnwind`]: delegates to `with_r_unwind_protect_sourced`.
///
/// # Parameters
///
/// - `f`: The closure to execute.
/// - `mode`: Which guard strategy to use.
/// - `source`: Attribution for telemetry if a panic occurs.
///
/// # Note on `fallback`
///
/// `GuardMode::CatchUnwind` diverges on panic (`Rf_error` never returns), so no
/// fallback value is needed. If you need a fallback (e.g. connection trampolines
/// that must return a value on panic without calling R), use
/// [`guarded_ffi_call_with_fallback`] instead.
#[inline]
pub fn guarded_ffi_call<F, R>(f: F, mode: GuardMode, source: PanicSource) -> R
where
    F: FnOnce() -> R,
{
    match mode {
        GuardMode::CatchUnwind => match catch_unwind(AssertUnwindSafe(f)) {
            Ok(val) => val,
            Err(payload) => {
                let msg = panic_payload_to_string(payload.as_ref());
                crate::panic_telemetry::fire(&msg, source);
                crate::error::r_stop(&msg)
            }
        },
        GuardMode::RUnwind => crate::unwind_protect::with_r_unwind_protect_sourced(f, None, source),
    }
}

/// Execute `f` inside a `CatchUnwind` guard, returning `fallback` on panic.
///
/// Unlike [`guarded_ffi_call`] with `CatchUnwind` (which diverges via `Rf_error`),
/// this variant returns the `fallback` value instead of raising an R error.
/// This is needed for connection trampolines where panicking through R/C frames
/// is UB but raising an R error is also undesirable (the caller expects a return
/// value indicating failure).
///
/// Telemetry is fired before returning the fallback.
#[inline]
pub fn guarded_ffi_call_with_fallback<F, R>(f: F, fallback: R, source: PanicSource) -> R
where
    F: FnOnce() -> R,
{
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(val) => val,
        Err(payload) => {
            let msg = panic_payload_to_string(payload.as_ref());
            crate::panic_telemetry::fire(&msg, source);
            fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catch_unwind_returns_value_on_success() {
        let result = guarded_ffi_call(|| 42, GuardMode::CatchUnwind, PanicSource::Worker);
        assert_eq!(result, 42);
    }

    #[test]
    fn fallback_returns_value_on_success() {
        let result = guarded_ffi_call_with_fallback(|| 42, -1, PanicSource::Connection);
        assert_eq!(result, 42);
    }

    #[test]
    fn fallback_returns_fallback_on_panic() {
        let result = guarded_ffi_call_with_fallback(|| panic!("boom"), -1, PanicSource::Connection);
        assert_eq!(result, -1);
    }

    #[test]
    fn fallback_fires_telemetry_on_panic() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let fired = std::sync::Arc::new(AtomicBool::new(false));
        let fired_clone = fired.clone();

        crate::panic_telemetry::set_panic_telemetry_hook(move |report| {
            assert_eq!(report.source, PanicSource::Connection);
            assert!(report.message.contains("test panic"));
            fired_clone.store(true, Ordering::SeqCst);
        });

        let _ =
            guarded_ffi_call_with_fallback(|| panic!("test panic"), 0i32, PanicSource::Connection);

        assert!(fired.load(Ordering::SeqCst), "telemetry hook should fire");
        crate::panic_telemetry::clear_panic_telemetry_hook();
    }
}
