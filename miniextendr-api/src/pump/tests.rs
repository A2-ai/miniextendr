use super::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

// region: happy_path

/// Worker sends N messages, pump receives all, worker returns Ok.
#[test]
fn happy_path() {
    let received = Arc::new(Mutex::new(Vec::<i64>::new()));
    let received2 = Arc::clone(&received);

    let result = WorkerPump::new()
        .drain_logs_each_tick(false)
        .run(
            |tx| {
                for i in 0..5i64 {
                    tx.send(i).unwrap();
                }
                Ok(42i64)
            },
            |msg| {
                received2.lock().unwrap().push(msg);
            },
        )
        .expect("run failed");

    assert_eq!(result, 42);
    let got = received.lock().unwrap();
    assert_eq!(*got, vec![0, 1, 2, 3, 4]);
}
// endregion

// region: worker_returns_err

/// Worker returns Err early; pump exits cleanly; run returns the worker's Err.
#[test]
fn worker_returns_err() {
    let pump_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let pump_count2 = Arc::clone(&pump_count);

    let result: Result<(), WorkerError> = WorkerPump::new().drain_logs_each_tick(false).run(
        |_tx| {
            // drop tx immediately, send no messages
            Err("deliberate worker error".into())
        },
        |_msg: ()| {
            pump_count2.fetch_add(1, Ordering::Relaxed);
        },
    );

    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("deliberate worker error"),
        "unexpected error: {msg}"
    );
    assert_eq!(
        pump_count.load(Ordering::Relaxed),
        0,
        "pump must not run if worker sends nothing"
    );
}
// endregion

// region: pump_panics

/// Pump callback panics; worker is joined (no thread leak).
///
/// We instrument the worker with a `Drop` guard that flips an `AtomicBool`
/// to confirm it ran after the scope unwinds.
#[test]
fn pump_panics() {
    // Instrument: flipped to true when the worker's guard is dropped.
    let worker_dropped = Arc::new(AtomicBool::new(false));
    let worker_dropped2 = Arc::clone(&worker_dropped);

    struct DropGuard(Arc<AtomicBool>);
    impl Drop for DropGuard {
        fn drop(&mut self) {
            self.0.store(true, Ordering::Relaxed);
        }
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        WorkerPump::<u8>::new().drain_logs_each_tick(false).run(
            move |tx| {
                let _guard = DropGuard(worker_dropped2);
                // Send one message to ensure pump fires at least once,
                // then block until rx is dropped (pump panic).
                let _ = tx.send(1u8);
                // The second send will return Err when rx drops on pump panic.
                let _ = tx.send(2u8);
                Ok(())
            },
            |_msg: u8| {
                panic!("pump panic");
            },
        )
    }));

    // The outer catch_unwind should see the pump panic propagate.
    assert!(result.is_err(), "expected panic to propagate");
    // The worker's DropGuard must have run — confirms worker was joined.
    assert!(
        worker_dropped.load(Ordering::Relaxed),
        "worker Drop did not run — possible thread leak"
    );
}
// endregion

// region: bounded_channel_no_deadlock_on_scope_drop

/// Fill the bounded channel; break out of pump early; assert worker exits.
///
/// The worker tries to send more messages than the channel can hold.  The
/// pump breaks after the first message (simulating an early exit), which
/// drops `rx`.  The worker's blocked `tx.send` must return `Err` (not
/// deadlock), and the worker must finish.
///
/// We verify no deadlock by asserting `run` completes before the test
/// times out (Rust's test harness kills hanging tests).
#[test]
fn bounded_channel_no_deadlock_on_scope_drop() {
    // Use a channel capacity of 1 so the worker blocks quickly.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        WorkerPump::<u8>::new()
            .channel_capacity(1)
            .drain_logs_each_tick(false)
            .run(
                |tx| {
                    // Try to send many messages — will block after slot 1 fills.
                    for i in 0u8..=10 {
                        if tx.send(i).is_err() {
                            // rx was dropped (pump broke out), exit cleanly.
                            break;
                        }
                    }
                    Ok(())
                },
                |_msg: u8| {
                    // Simulate early pump exit by panicking after first message.
                    panic!("early pump exit");
                },
            )
    }));

    // We get a panic from the pump, but no deadlock (test would hang otherwise).
    assert!(result.is_err(), "expected pump panic");
}
// endregion

// region: drain_logs_each_tick_default_on

/// With the `log` feature enabled: a log record from the worker is flushed
/// to the render sink by the time the pump processes the next message.
///
/// Without the `log` feature this test reduces to a basic send/recv check.
#[test]
fn drain_logs_each_tick_default_on() {
    #[cfg(feature = "log")]
    {
        use crate::optionals::log_impl::{
            LOG_TEST_LOCK, install_r_logger, set_fake_main_thread, set_log_level, take_rendered,
        };

        // Acquire the same lock that log_impl tests use so we don't race on
        // the shared QUEUE / DROPPED / TEST_SINK globals.
        let _guard = LOG_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        install_r_logger();
        set_log_level("trace");
        // Clear residual state from prior tests.
        take_rendered();

        let rendered_before_pump: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let rendered_before_pump2 = Arc::clone(&rendered_before_pump);

        // The pump loop (and drain_log_queue inside it) runs on *this* thread.
        // Mark this thread as main so drain_log_queue() actually drains.
        // The worker is a real scoped thread whose FAKE_IS_MAIN is None,
        // so is_main() → is_r_main_thread() → false → records go to queue.
        set_fake_main_thread(Some(true));

        WorkerPump::<u8>::new()
            .drain_logs_each_tick(true)
            .run(
                |tx| {
                    // Runs on a real background thread; is_main() == false
                    // → log record is buffered in QUEUE, not rendered yet.
                    log::info!("hello from worker");
                    tx.send(1u8).ok();
                    Ok(())
                },
                move |msg: u8| {
                    // WorkerPump calls drain_log_queue() BEFORE calling us.
                    // Since is_main() == true on the pump thread, the queue
                    // was already drained → record is in TEST_SINK.
                    let rendered = take_rendered();
                    rendered_before_pump2.lock().unwrap().extend(rendered);
                    let _ = msg;
                },
            )
            .expect("run failed");

        // Restore state.
        set_fake_main_thread(None);
        take_rendered();

        // The drain happened before pump(msg), so the record must have
        // appeared in the captured snapshot.
        let rendered = rendered_before_pump.lock().unwrap();
        assert!(
            rendered.iter().any(|m| m.contains("hello from worker")),
            "log record was not drained before pump tick; got: {rendered:?}"
        );
    }

    // When `log` feature is disabled: just verify run completes.
    #[cfg(not(feature = "log"))]
    {
        let result: Result<u8, WorkerError> = WorkerPump::new().drain_logs_each_tick(true).run(
            |tx| {
                tx.send(99u8).ok();
                Ok(99u8)
            },
            |_| {},
        );
        assert_eq!(result.unwrap(), 99u8);
    }
}
// endregion
