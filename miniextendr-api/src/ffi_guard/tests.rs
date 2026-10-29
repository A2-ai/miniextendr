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

    let _ = guarded_ffi_call_with_fallback(|| panic!("test panic"), 0i32, PanicSource::Connection);

    assert!(fired.load(Ordering::SeqCst), "telemetry hook should fire");
    crate::panic_telemetry::clear_panic_telemetry_hook();
}
