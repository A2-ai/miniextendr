use super::*;
use std::thread;

/// Acquire the test serialization lock, recovering from a previous test
/// panic that left the lock poisoned.
fn acquire_test_lock() -> std::sync::MutexGuard<'static, ()> {
    LOG_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

// Convenience: install logger if not already installed, set level to Trace.
fn init_trace() {
    install_r_logger();
    set_log_level("trace");
}

// Clear the queue and reset dropped counter between tests.
fn reset_state() {
    // Recover from poisoned mutex (prior test may have panicked holding the lock).
    let mut q = queue().lock().unwrap_or_else(|e| e.into_inner());
    q.clear();
    drop(q);
    DROPPED.store(0, Ordering::Relaxed);
    take_rendered();
    set_fake_main_thread(None);
}

// Enqueue a record as if from a worker thread (bypasses level filter).
fn enqueue(level: Level, msg: &str) {
    let mut q = queue().lock().unwrap_or_else(|e| e.into_inner());
    if q.len() >= QUEUE_CAPACITY {
        q.pop_front();
        DROPPED.fetch_add(1, Ordering::Relaxed);
    }
    q.push_back((level, msg.to_string()));
}

// region: default_level_is_off

/// After `install_r_logger`, the max level is Off so info!/error! produce
/// nothing (no queue entries, no rendered output).
#[test]
fn default_level_is_off() {
    let _guard = acquire_test_lock();
    reset_state();
    install_r_logger(); // resets to Off
    set_fake_main_thread(Some(true));

    // These should be filtered out before reaching queue or render.
    log::info!("should be suppressed");
    log::error!("also suppressed");

    assert_eq!(
        queue().lock().unwrap_or_else(|e| e.into_inner()).len(),
        0,
        "queue must be empty"
    );
    assert!(take_rendered().is_empty(), "nothing should be rendered");
}
// endregion

// region: worker_buffering

/// A record logged from a non-main thread lands in QUEUE and is not
/// rendered immediately.
#[test]
fn worker_buffering() {
    let _guard = acquire_test_lock();
    reset_state();
    init_trace();

    // Pretend we are NOT on the main thread.
    set_fake_main_thread(Some(false));
    log::info!("buffered from worker");

    let q = queue().lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(q.len(), 1, "record must be in queue");
    assert_eq!(q[0].1, "buffered from worker");
    drop(q);

    assert!(
        take_rendered().is_empty(),
        "worker records must not be rendered immediately"
    );
}
// endregion

// region: fifo_drain

/// Records enqueued in order are drained in the same order (FIFO).
#[test]
fn fifo_drain() {
    let _guard = acquire_test_lock();
    reset_state();

    // Enqueue directly (bypasses level/thread check).
    for i in 0..5 {
        enqueue(Level::Info, &format!("msg-{i}"));
    }

    // Drain on the "main thread".
    set_fake_main_thread(Some(true));
    drain_log_queue();

    let rendered = take_rendered();
    assert_eq!(rendered.len(), 5);
    for (i, r) in rendered.iter().enumerate() {
        assert_eq!(r, &format!("msg-{i}"), "FIFO order violated at index {i}");
    }
}
// endregion

// region: overflow_accounting

/// Enqueueing capacity+50 records results in exactly 50 dropped and the
/// queue stays at QUEUE_CAPACITY.
#[test]
fn overflow_accounting() {
    let _guard = acquire_test_lock();
    reset_state();

    let extra = 50usize;
    for i in 0..(QUEUE_CAPACITY + extra) {
        enqueue(Level::Info, &format!("msg-{i}"));
    }

    assert_eq!(
        DROPPED.load(Ordering::Relaxed),
        extra,
        "expected {extra} dropped records"
    );
    assert_eq!(
        queue().lock().unwrap_or_else(|e| e.into_inner()).len(),
        QUEUE_CAPACITY,
        "queue must be capped at {QUEUE_CAPACITY}"
    );
}
// endregion

// region: overflow_warning_emitted_and_reset

/// After overflow, the first drain emits a warning. A subsequent drain
/// does NOT re-emit it.
#[test]
fn overflow_warning_emitted_and_reset() {
    let _guard = acquire_test_lock();
    reset_state();

    // Force overflow by enqueueing capacity+1 records.
    for i in 0..=(QUEUE_CAPACITY) {
        enqueue(Level::Info, &format!("msg-{i}"));
    }

    set_fake_main_thread(Some(true));

    // First drain — overflow warning + QUEUE_CAPACITY records.
    drain_log_queue();
    let first = take_rendered();
    assert!(
        first
            .iter()
            .any(|m| m.contains("log records dropped due to queue overflow")),
        "expected overflow warning in first drain; got: {first:?}"
    );

    // Second drain — nothing left, no overflow warning.
    drain_log_queue();
    let second = take_rendered();
    assert!(
        second.iter().all(|m| !m.contains("log records dropped")),
        "unexpected overflow warning re-emitted in second drain; got: {second:?}"
    );
}
// endregion

// region: drain_on_error_path

/// A record buffered by a "worker" is visible in the rendered sink after
/// the drain that the trampoline would call on error exit.
///
/// This test exercises the drain path directly, which is what the
/// `drain_log_queue_if_available()` call in `unwind_protect.rs` fires on
/// every FFI exit path (normal, panic, R longjmp).
#[test]
fn drain_on_error_path() {
    let _guard = acquire_test_lock();
    reset_state();

    // Step 1: buffer a record as if from a worker thread.
    enqueue(Level::Warn, "worker warning before error");

    // Step 2: simulate trampoline error-path drain — called from main thread.
    set_fake_main_thread(Some(true));
    drain_log_queue();

    // Step 3: the record must appear in the rendered sink.
    let rendered = take_rendered();
    assert!(
        rendered
            .iter()
            .any(|m| m.contains("worker warning before error")),
        "buffered record must be drained even on error path; got: {rendered:?}"
    );
}
// endregion

// region: std::thread spawn buffering (real thread, no fake override)

/// A record from a real `std::thread::spawn` thread is buffered (not
/// rendered), and appears after drain on the "main thread".
#[test]
fn real_thread_buffering() {
    let _guard = acquire_test_lock();
    reset_state();
    init_trace();

    // Spawn a thread that will NOT be the R main thread (R_MAIN_THREAD_ID
    // is either unset or set to the test thread, neither matches the spawned
    // thread ID).
    let handle = thread::spawn(|| {
        log::info!("from spawned thread");
    });
    handle.join().unwrap();

    // The record must be in the queue (not rendered yet).
    assert_eq!(queue().lock().unwrap_or_else(|e| e.into_inner()).len(), 1);
    assert!(take_rendered().is_empty());

    // Drain on the fake-main thread.
    set_fake_main_thread(Some(true));
    drain_log_queue();
    let rendered = take_rendered();
    assert!(
        rendered.iter().any(|m| m.contains("from spawned thread")),
        "got: {rendered:?}"
    );
}
// endregion

// region: MatchArg for LevelFilter

/// Every entry in CHOICES round-trips through `from_choice` + `to_choice`,
/// and `CHOICES.len()` matches `LevelFilter::iter().count()` — this pins the
/// "in-sync with the log crate" invariant at test time.
#[test]
fn match_arg_choices_match_levelfilter_iter() {
    use crate::MatchArg;
    use log::LevelFilter;
    use std::str::FromStr;

    let choices = <LevelFilter as MatchArg>::CHOICES;
    assert_eq!(
        choices.len(),
        LevelFilter::iter().count(),
        "CHOICES.len() must equal LevelFilter::iter().count()"
    );
    for &c in choices {
        let filter = LevelFilter::from_str(c)
            .unwrap_or_else(|_| panic!("from_str failed for choice {:?}", c));
        assert_eq!(
            filter.to_choice(),
            c,
            "to_choice did not round-trip for {:?}",
            c
        );
    }
}

/// An unrecognised string returns `None`.
#[test]
fn match_arg_from_choice_unknown_returns_none() {
    use crate::MatchArg;
    use log::LevelFilter;

    assert_eq!(<LevelFilter as MatchArg>::from_choice("nope"), None);
}

/// For every variant produced by `LevelFilter::iter()`, converting to a
/// choice string and back yields the original variant.
#[test]
fn match_arg_to_choice_round_trip() {
    use crate::MatchArg;
    use log::LevelFilter;

    for v in LevelFilter::iter() {
        let s = v.to_choice();
        assert_eq!(
            <LevelFilter as MatchArg>::from_choice(s),
            Some(v),
            "round-trip failed for {:?}",
            v
        );
    }
}
// endregion
