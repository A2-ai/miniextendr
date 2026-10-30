use super::*;
use std::thread;

// Convenience: install logger if not already installed, set level to Trace.
fn init_trace() {
    install_r_logger();
    set_log_level("trace");
}

// Send a record as a thread other than the main thread does (bypasses the
// level filter).
fn send(level: Level, msg: &str) {
    send_to_main(level, msg.to_string());
}

// region: default_level_is_off

/// After `install_r_logger`, the max level is Off so info!/error! produce
/// nothing, on the main thread or off it.
#[test]
fn default_level_is_off() {
    on_main_thread(|| {
        install_r_logger(); // resets to Off

        // These should be filtered out before reaching the channel or render.
        log::info!("should be suppressed");
        set_fake_main_thread(Some(false));
        log::error!("also suppressed");
        set_fake_main_thread(Some(true));
        drain_log_queue();

        assert!(take_rendered().is_empty(), "nothing should be rendered");
    });
}
// endregion

// region: worker_buffering

/// A record logged off the main thread waits in the channel: it is not
/// rendered until the main thread drains.
#[test]
fn worker_buffering() {
    on_main_thread(|| {
        init_trace();

        // Pretend we are NOT on the main thread.
        set_fake_main_thread(Some(false));
        log::info!("buffered from worker");
        // Draining off the main thread does nothing.
        drain_log_queue();
        assert!(
            take_rendered().is_empty(),
            "worker records must not be rendered immediately"
        );

        set_fake_main_thread(Some(true));
        drain_log_queue();
        assert_eq!(take_rendered(), ["buffered from worker"]);
    });
}
// endregion

// region: fifo_drain

/// Records sent in order are drained in the same order (FIFO).
#[test]
fn fifo_drain() {
    on_main_thread(|| {
        for i in 0..5 {
            send(Level::Info, &format!("msg-{i}"));
        }

        drain_log_queue();

        let rendered = take_rendered();
        assert_eq!(rendered.len(), 5);
        for (i, r) in rendered.iter().enumerate() {
            assert_eq!(r, &format!("msg-{i}"), "FIFO order violated at index {i}");
        }
    });
}
// endregion

// region: main_thread_record_after_waiting_records

/// A record logged on the main thread prints after the records other threads
/// sent before it.
#[test]
fn main_thread_record_after_waiting_records() {
    on_main_thread(|| {
        init_trace();

        thread::spawn(|| log::info!("from spawned thread"))
            .join()
            .unwrap();
        log::info!("from main thread");

        assert_eq!(take_rendered(), ["from spawned thread", "from main thread"]);
    });
}
// endregion

// region: overflow_accounting

/// Sending capacity+50 records from a spawned thread drops the 50 newest: the
/// drain prints the first QUEUE_CAPACITY records in order, then the overflow
/// warning.
#[test]
fn overflow_accounting() {
    on_main_thread(|| {
        init_trace();

        let extra = 50usize;
        thread::spawn(move || {
            for i in 0..(QUEUE_CAPACITY + extra) {
                log::info!("msg-{i}");
            }
        })
        .join()
        .unwrap();

        assert_eq!(
            DROPPED.load(Ordering::Relaxed),
            extra,
            "expected {extra} dropped records"
        );

        drain_log_queue();
        let rendered = take_rendered();
        assert_eq!(rendered.len(), QUEUE_CAPACITY + 1);
        for (i, r) in rendered[..QUEUE_CAPACITY].iter().enumerate() {
            assert_eq!(r, &format!("msg-{i}"), "kept records out of order at {i}");
        }
        assert_eq!(
            rendered[QUEUE_CAPACITY],
            format!("{extra} log records dropped due to queue overflow")
        );
    });
}
// endregion

// region: overflow_warning_emitted_and_reset

/// After overflow, the first drain emits a warning. A subsequent drain
/// does NOT re-emit it.
#[test]
fn overflow_warning_emitted_and_reset() {
    on_main_thread(|| {
        // Force overflow by sending capacity+1 records.
        for i in 0..=(QUEUE_CAPACITY) {
            send(Level::Info, &format!("msg-{i}"));
        }

        // First drain — QUEUE_CAPACITY records + overflow warning.
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
            second.is_empty(),
            "nothing should be left after the first drain; got: {second:?}"
        );
    });
}
// endregion

// region: drain_on_error_path

/// A record sent by a "worker" is visible in the rendered sink after
/// the drain that the trampoline would call on error exit.
///
/// This test exercises the drain path directly, which is what the
/// `drain_log_queue_if_available()` call in `unwind_protect.rs` fires on
/// every FFI exit path (normal, panic, R longjmp).
#[test]
fn drain_on_error_path() {
    on_main_thread(|| {
        // Step 1: send a record as if from a worker thread.
        send(Level::Warn, "worker warning before error");

        // Step 2: simulate trampoline error-path drain — called from main thread.
        drain_log_queue();

        // Step 3: the record must appear in the rendered sink.
        let rendered = take_rendered();
        assert!(
            rendered
                .iter()
                .any(|m| m.contains("worker warning before error")),
            "sent record must be drained even on error path; got: {rendered:?}"
        );
    });
}
// endregion

// region: std::thread spawn buffering (real thread, no fake override)

/// A record from a real `std::thread::spawn` thread is not rendered when
/// logged, and the next drain on the "main thread" prints it.
#[test]
fn real_thread_buffering() {
    on_main_thread(|| {
        init_trace();

        // The spawned thread has no fake override and is not R's main thread
        // (R_MAIN_THREAD_ID is unset or another thread), so it sends.
        thread::spawn(|| {
            log::info!("from spawned thread");
        })
        .join()
        .unwrap();

        assert!(take_rendered().is_empty());

        drain_log_queue();
        assert_eq!(take_rendered(), ["from spawned thread"]);
    });
}
// endregion

// region: per-thread order across threads

/// Records from several threads interleave, but each thread's records print in
/// the order that thread sent them.
#[test]
fn per_thread_order() {
    on_main_thread(|| {
        init_trace();

        thread::scope(|s| {
            for t in 0..4 {
                s.spawn(move || {
                    for i in 0..100 {
                        log::info!("t{t}-{i}");
                    }
                });
            }
        });

        drain_log_queue();
        let rendered = take_rendered();
        assert_eq!(rendered.len(), 400);
        for t in 0..4 {
            let prefix = format!("t{t}-");
            let seen: Vec<&str> = rendered
                .iter()
                .filter_map(|m| m.strip_prefix(&prefix))
                .collect();
            let expected: Vec<String> = (0..100).map(|i| i.to_string()).collect();
            assert_eq!(seen, expected, "thread {t} records out of order");
        }
    });
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
