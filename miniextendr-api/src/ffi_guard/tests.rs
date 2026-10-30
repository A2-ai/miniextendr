use super::*;

#[test]
fn catch_unwind_returns_value_on_success() {
    let result = guarded_ffi_call(|| 42, GuardMode::CatchUnwind, PanicSource::Altrep);
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

/// The hook records; the test asserts outside it, where a failure reaches the
/// test runner (`fire` swallows a panicking hook). The slot is per thread, so
/// no other test's panic reaches this hook (#1492).
#[test]
fn fallback_fires_telemetry_on_panic() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let reports = Rc::new(RefCell::new(Vec::new()));
    let seen = Rc::clone(&reports);
    crate::panic_telemetry::set_panic_telemetry_hook(move |report| {
        seen.borrow_mut()
            .push((report.source, report.message.to_string()));
    });

    let _ = guarded_ffi_call_with_fallback(|| panic!("test panic"), 0i32, PanicSource::Connection);
    crate::panic_telemetry::clear_panic_telemetry_hook();

    assert_eq!(
        *reports.borrow(),
        [(PanicSource::Connection, "test panic".to_string())]
    );
}
