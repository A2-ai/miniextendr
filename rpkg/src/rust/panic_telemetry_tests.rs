//! Test fixtures for panic_telemetry.

use miniextendr_api::panic_telemetry;
use miniextendr_api::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};

static TELEMETRY_COUNT: AtomicU32 = AtomicU32::new(0);

/// Install a telemetry hook that counts panics.
#[miniextendr]
pub fn telemetry_install_counter() {
    TELEMETRY_COUNT.store(0, Ordering::SeqCst);
    panic_telemetry::set_panic_telemetry_hook(|_report| {
        TELEMETRY_COUNT.fetch_add(1, Ordering::SeqCst);
    });
}

/// Get the telemetry counter value.
#[miniextendr]
pub fn telemetry_get_count() -> i32 {
    TELEMETRY_COUNT.load(Ordering::SeqCst) as i32
}

/// Clear the telemetry hook.
#[miniextendr]
pub fn telemetry_clear_hook() {
    panic_telemetry::clear_panic_telemetry_hook();
}
