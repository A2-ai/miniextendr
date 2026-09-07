//! This binary owns the process-global telemetry hook. Keep its lifecycle in
//! one test so ordinary parallel unit tests cannot fire or replace the hook.

use miniextendr_api::guarded_ffi_call_with_fallback;
use miniextendr_api::panic_telemetry::{
    PanicSource, clear_panic_telemetry_hook, set_panic_telemetry_hook,
};
use std::sync::mpsc;

#[test]
fn fallback_reports_panics_without_cross_test_hook_interference() {
    let (events, received) = mpsc::channel();
    set_panic_telemetry_hook(move |report| {
        // fire() suppresses hook panics, so assertions belong after the calls.
        let _ = events.send((report.message.to_owned(), report.source));
    });

    let results = std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            guarded_ffi_call_with_fallback(|| panic!("worker panic"), -1, PanicSource::Worker)
        });
        let connection = guarded_ffi_call_with_fallback(
            || panic!("connection panic"),
            -2,
            PanicSource::Connection,
        );
        (worker.join().unwrap(), connection)
    });
    let success = guarded_ffi_call_with_fallback(|| 42, -1, PanicSource::Altrep);
    clear_panic_telemetry_hook();
    let after_clear =
        guarded_ffi_call_with_fallback(|| panic!("after clear"), -3, PanicSource::UnwindProtect);

    let mut reports: Vec<_> = received.try_iter().collect();
    reports.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(results, (-1, -2));
    assert_eq!(success, 42);
    assert_eq!(after_clear, -3);
    assert_eq!(
        reports,
        vec![
            ("connection panic".to_owned(), PanicSource::Connection),
            ("worker panic".to_owned(), PanicSource::Worker),
        ],
    );
}
