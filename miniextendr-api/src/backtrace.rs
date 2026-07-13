//! Configurable panic hook for miniextendr-based R packages.
//!
//! The hook is process-global (`std::panic::set_hook` writes to a process
//! slot), but its closure lives in the DLL's code. If the package is
//! unloaded (e.g. `library.dynam.unload` / `dyn.unload`) without removing
//! the hook, the next panic anywhere in the process jumps to unmapped
//! memory and tears down the SEH state on Windows — surfacing as "failed
//! to initiate panic, error 5" in the next DLL that tries to unwind (#277).
//!
//! `miniextendr_panic_hook()` installs; `miniextendr_panic_hook_uninstall()`
//! takes it back off. Both are idempotent and paired by the init / unload
//! code in `worker.rs`.

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};

/// True iff this DLL instance has installed the panic hook.
///
/// Per-DLL: each dyn.load of the compiled artifact gets a fresh static, so
/// the install/uninstall lifecycle is scoped to one load.
static INSTALLED: AtomicBool = AtomicBool::new(false);

thread_local! {
    /// Source location of the most recent panic on *this* thread, captured by
    /// the panic hook. `Location` borrows the `PanicHookInfo`, so we snapshot an
    /// owned `(file, line)`.
    ///
    /// Per-thread because the hook fires on the panicking thread: a worker-thread
    /// panic records here on the worker; a main-thread panic records here on main.
    /// [`take_last_panic_location`] reads-and-clears so a stale location can never
    /// leak onto a later, location-less message on the same (reused) thread.
    static LAST_PANIC_LOCATION: Cell<Option<(String, u32)>> = const { Cell::new(None) };
}

/// Record a panic's source location into the current thread's take-once slot.
///
/// Called unconditionally from the hook (before the `MINIEXTENDR_BACKTRACE`
/// env check) so the location is available to the panic-stringification sites
/// regardless of whether the stderr traceback is enabled.
fn record_panic_location(info: &std::panic::PanicHookInfo<'_>) {
    let loc = info.location().map(|l| (l.file().to_string(), l.line()));
    LAST_PANIC_LOCATION.with(|cell| cell.set(loc));
}

/// Take (read + clear) the last panic location recorded on the current thread.
///
/// Returns `None` when no panic hook fired on this thread since the last take
/// (e.g. the hook was never installed, as in some unit tests / the engine).
/// Clearing on read means a location from a panic that was caught-and-diverted
/// (e.g. the internal `RErrorMarker` on the R-longjmp path) never bleeds onto an
/// unrelated later message.
pub(crate) fn take_last_panic_location() -> Option<(String, u32)> {
    LAST_PANIC_LOCATION.with(|cell| cell.take())
}

/// Register the miniextendr panic hook.
///
/// If `MINIEXTENDR_BACKTRACE` is truthy (`yes`/`true`/`1`/`on`, per
/// `crate::env_flag::parse_bool`), the default Rust panic hook runs (full
/// traceback printed to stderr); otherwise the hook swallows the panic output
/// silently so the R error (emitted by `panic_message_to_r_error`) is what
/// users see. Unrecognized values default to off.
///
/// Idempotent within a DLL instance: the first call installs, subsequent
/// calls are no-ops. If the DLL is unloaded and loaded again, the new
/// instance has its own `INSTALLED` flag and installs afresh.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn miniextendr_panic_hook() {
    if INSTALLED
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |x| {
        // Capture the panic location for the R-facing message BEFORE the env
        // check, so `(at file:line)` is surfaced whether or not the stderr
        // traceback is enabled.
        record_panic_location(x);
        let show_traceback = std::env::var("MINIEXTENDR_BACKTRACE")
            .ok()
            .and_then(|v| crate::env_flag::parse_bool(&v))
            .unwrap_or(false);
        if show_traceback {
            default_hook(x)
        }
    }));
}

/// Remove the miniextendr panic hook and revert to Rust's default.
///
/// Called from `miniextendr_runtime_shutdown` (which runs in
/// `R_unload_<pkg>`). Must run before the DLL's code pages are unmapped —
/// otherwise the next panic, anywhere in the process, executes freed
/// memory. See #277.
///
/// Idempotent: safe to call even if the hook wasn't installed.
pub(crate) fn miniextendr_panic_hook_uninstall() {
    if !INSTALLED.swap(false, Ordering::AcqRel) {
        return;
    }
    // Take and drop our hook. `take_hook` returns the current hook and
    // resets the process slot to Rust's default hook. Dropping our
    // `Box<dyn Fn>` also drops the captured `default_hook`, which is fine:
    // we're intentionally reverting to the process default, not to
    // whatever hook existed before install.
    let _our_hook = std::panic::take_hook();
}
