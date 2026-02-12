//! Configurable panic hook for miniextendr-based R packages.

/// This function registers a configurable print panic hook, for use in miniextendr-based R-packages.
/// If the environment variable `MINIEXTENDR_BACKTRACE` is set to either `true` or `1`,
/// then it displays the entire Rust panic traceback (default hook), otherwise it omits the panic backtrace.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn miniextendr_panic_hook() {
    static RUN_ONCE: std::sync::Once = std::sync::Once::new();
    RUN_ONCE.call_once_force(|x| {
        // On poisoned retry, perform full initialization instead of
        // returning early. The previous attempt panicked before
        // completing, so the panic hook may not be installed.
        if x.is_poisoned() {
            eprintln!(
                "warning: miniextendr panic hook is retrying after a previous failed attempt"
            );
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
    });
}
