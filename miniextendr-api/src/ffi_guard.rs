//! Unified FFI guard for catching panics at Rust-R boundaries.
//!
//! Four modules independently catch panics at FFI boundaries: `worker.rs`,
//! `altrep_bridge.rs`, `unwind_protect.rs`, and `connection.rs`. This module
//! extracts the common pattern into a single `guarded_ffi_call` function.
//!
//! ## Guard Modes
//!
//! - [`GuardMode::CatchUnwind`]: Wraps the closure in `catch_unwind`. On panic,
//!   fires telemetry and calls `r_stop` (which diverges via `Rf_error`).
//!   Used by worker and connection trampolines.
//!
//! - [`GuardMode::RUnwind`]: Uses `R_UnwindProtect` to catch both Rust panics
//!   and R longjmps. Used by ALTREP callbacks that call R APIs.
//!
//! The ALTREP-specific `Unsafe` mode (no protection at all) stays in
//! `altrep_bridge.rs` since it has no general applicability.

use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::panic_telemetry::PanicSource;
use crate::unwind_protect::panic_payload_to_string;

/// FFI guard mode controlling how panics are caught at Rust-R boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardMode {
    /// `catch_unwind` only. On panic: fire telemetry, then `r_stop()` (diverges).
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
/// - For [`GuardMode::CatchUnwind`]: calls `r_stop` (diverges — never returns).
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
/// `GuardMode::CatchUnwind` diverges on panic (`r_stop` never returns), so no
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
/// Unlike [`guarded_ffi_call`] with `CatchUnwind` (which diverges via `r_stop`),
/// this variant returns the `fallback` value instead of raising an R error.
/// This is needed for connection trampolines where panicking through R/C frames
/// is UB but calling `r_stop` is also undesirable (the caller expects a return
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
