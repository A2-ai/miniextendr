//! The hook slot on the test's own thread (no R runtime).

use super::*;
use std::cell::Cell;

/// A hook that records every message; `Rc` state, so `!Send`.
fn recorder() -> (Rc<RefCell<Vec<String>>>, impl Fn(&PanicReport) + 'static) {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let log = Rc::clone(&seen);
    let record = move |report: &PanicReport| log.borrow_mut().push(report.message.to_string());
    (seen, record)
}

#[test]
fn fire_calls_the_hook_with_message_and_source() {
    let last = Rc::new(Cell::new(None));
    let seen = Rc::clone(&last);
    set_panic_telemetry_hook(move |report| seen.set(Some((report.source, report.message.len()))));
    fire("boom", PanicSource::Connection);
    assert_eq!(last.get(), Some((PanicSource::Connection, 4)));
    clear_panic_telemetry_hook();
    fire("after clear", PanicSource::Altrep);
    assert_eq!(last.get(), Some((PanicSource::Connection, 4)));
}

#[test]
fn a_hook_can_set_the_next_hook() {
    let (seen, record) = recorder();
    let record = RefCell::new(Some(record));
    set_panic_telemetry_hook(move |_| {
        if let Some(record) = record.borrow_mut().take() {
            set_panic_telemetry_hook(record);
        }
    });
    fire("first", PanicSource::Altrep);
    assert!(seen.borrow().is_empty());
    fire("second", PanicSource::Altrep);
    assert_eq!(*seen.borrow(), ["second"]);
    clear_panic_telemetry_hook();
}

#[test]
fn a_hook_can_clear_itself() {
    let (seen, record) = recorder();
    set_panic_telemetry_hook(move |report| {
        record(report);
        clear_panic_telemetry_hook();
    });
    fire("first", PanicSource::UnwindProtect);
    fire("second", PanicSource::UnwindProtect);
    assert_eq!(*seen.borrow(), ["first"]);
}

#[test]
fn a_panicking_hook_is_suppressed() {
    set_panic_telemetry_hook(|_| panic!("hook failed"));
    fire("boom", PanicSource::Altrep);
    clear_panic_telemetry_hook();
}

/// The slot belongs to the thread that set it.
#[test]
fn another_thread_has_its_own_slot() {
    let (seen, record) = recorder();
    set_panic_telemetry_hook(record);
    let fired_elsewhere = std::thread::spawn(|| {
        let called = Rc::new(Cell::new(0));
        let count = Rc::clone(&called);
        fire("before a hook is set here", PanicSource::Altrep);
        set_panic_telemetry_hook(move |_| count.set(count.get() + 1));
        fire("elsewhere", PanicSource::Altrep);
        called.get()
    })
    .join()
    .unwrap();
    assert_eq!(fired_elsewhere, 1);
    assert!(seen.borrow().is_empty());
    clear_panic_telemetry_hook();
}
