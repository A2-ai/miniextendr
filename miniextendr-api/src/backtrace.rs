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

use std::sync::atomic::{AtomicBool, Ordering};

/// True iff this DLL instance has installed the panic hook.
///
/// Per-DLL: each dyn.load of the compiled artifact gets a fresh static, so
/// the install/uninstall lifecycle is scoped to one load.
static INSTALLED: AtomicBool = AtomicBool::new(false);

/// Register the miniextendr panic hook.
///
/// If `MINIEXTENDR_BACKTRACE` is set to `true` or `1`, the default Rust
/// panic hook runs (full traceback printed to stderr); otherwise the hook
/// swallows the panic output silently so the R error (emitted by
/// `panic_message_to_r_error`) is what users see.
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
        let show_traceback = std::env::var("MINIEXTENDR_BACKTRACE")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
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
