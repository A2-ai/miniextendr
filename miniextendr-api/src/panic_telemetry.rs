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
//! When no hook is set, `fire()` performs a single atomic load and returns.

use std::sync::atomic::{AtomicPtr, Ordering};

/// Describes where a panic originated before being converted to an R error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanicSource {
    /// Panic on the worker thread (caught by `run_on_worker`).
    Worker,
    /// Panic inside an ALTREP trampoline (caught by `catch_altrep_panic`).
    Altrep,
    /// Panic inside `with_r_unwind_protect` (caught by `panic_payload_to_r_error`).
    UnwindProtect,
}

/// A structured panic report passed to the telemetry hook.
pub struct PanicReport<'a> {
    /// The panic message (extracted from the panic payload).
    pub message: &'a str,
    /// Which panic→R-error boundary caught this panic.
    pub source: PanicSource,
}

/// Type-erased hook function pointer.
///
/// We store a raw pointer to a leaked `Box<dyn Fn(&PanicReport) + Send + Sync>`.
/// This avoids the overhead of `Arc`/`Mutex` on the hot path — the hook is
/// set once and read many times.
static HOOK: AtomicPtr<()> = AtomicPtr::new(std::ptr::null_mut());

/// Register a panic telemetry hook.
///
/// The hook is called with a [`PanicReport`] each time a Rust panic is about
/// to be converted into an R error. Only one hook can be active at a time;
/// calling this again replaces the previous hook.
///
/// # Thread Safety
///
/// The hook may be called from any thread (worker thread, main R thread, etc.).
/// Ensure your closure is safe to call concurrently.
pub fn set_panic_telemetry_hook(f: impl Fn(&PanicReport) + Send + Sync + 'static) {
    let boxed: Box<dyn Fn(&PanicReport) + Send + Sync> = Box::new(f);
    let leaked = Box::into_raw(Box::new(boxed));
    let old = HOOK.swap(leaked.cast(), Ordering::Release);
    if !old.is_null() {
        // Drop the previous hook
        unsafe {
            drop(Box::from_raw(
                old.cast::<Box<dyn Fn(&PanicReport) + Send + Sync>>(),
            ));
        }
    }
}

/// Fire the telemetry hook if one is set.
///
/// Called internally at each panic→R-error conversion site. When no hook is
/// registered, this is a single atomic load returning immediately.
pub(crate) fn fire(message: &str, source: PanicSource) {
    let ptr = HOOK.load(Ordering::Acquire);
    if ptr.is_null() {
        return;
    }
    // SAFETY: ptr was produced by Box::into_raw(Box::new(boxed_fn)) and is
    // never deallocated while loaded (only swapped in set_panic_telemetry_hook).
    let hook =
        unsafe { &*ptr.cast::<Box<dyn Fn(&PanicReport) + Send + Sync>>() };
    let report = PanicReport { message, source };
    hook(&report);
}
