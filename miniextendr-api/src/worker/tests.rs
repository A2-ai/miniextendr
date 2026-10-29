use super::*;

#[test]
fn sendable_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<Sendable<*const u8>>();
}

#[test]
fn with_r_thread_panics_before_init() {
    // If another test already called miniextendr_runtime_init (via Once),
    // we can't test the pre-init path. Verify at least panics from wrong thread.
    if R_MAIN_THREAD_ID.get().is_some() {
        let handle = std::thread::spawn(|| std::panic::catch_unwind(|| with_r_thread(|| 42)));
        let result = handle.join().expect("thread panicked outside catch_unwind");
        assert!(
            result.is_err(),
            "with_r_thread should panic from non-main thread"
        );
        return;
    }
    let result = std::panic::catch_unwind(|| {
        with_r_thread(|| 42);
    });
    assert!(result.is_err());
    let payload = result.unwrap_err();
    let msg = crate::unwind_protect::panic_payload_to_string(payload.as_ref());
    assert!(
        msg.contains("miniextendr_runtime_init"),
        "expected init error message, got: {msg}"
    );
}

#[test]
fn has_worker_context_false_outside_worker() {
    assert!(!has_worker_context());
}

// region: Feature-gated tests: worker-thread

#[cfg(feature = "worker-thread")]
mod worker_tests {
    use super::*;

    /// Calling `run_on_worker` from within worker code (re-entry) must be
    /// detected and panic, not deadlock.
    ///
    /// Dispatches from the current test thread rather than a spawned one.
    /// `run_on_worker` requires the R main thread (#730), and a spawned
    /// std::thread would trip the debug_assert before the re-entry check
    /// gets a chance to run. If this test happens to run on a thread that
    /// isn't `R_MAIN_THREAD_ID` (another test initialised first), skip —
    /// the reentry behaviour is checked elsewhere on each invocation of
    /// the suite.
    #[test]
    fn run_on_worker_reentry_panics_not_deadlocks() {
        miniextendr_runtime_init();
        if !is_r_main_thread() {
            return;
        }

        let result = run_on_worker(|| {
            // Re-entry: this runs on the worker thread.
            run_on_worker(|| 42).unwrap();
        });

        let msg = result.expect_err("re-entry should surface as Err");
        assert!(
            msg.contains("re-entr") || msg.contains("Re-entr"),
            "expected re-entry error, got: {msg}"
        );
    }

    /// `run_on_worker` from a non-main thread trips the debug-only
    /// precondition assert (#730). Spawning a fresh std::thread guarantees
    /// the caller isn't `R_MAIN_THREAD_ID`, regardless of which thread
    /// `miniextendr_runtime_init` first ran on.
    ///
    /// `cfg(debug_assertions)` only — the assert is compiled out in
    /// release, by design.
    #[cfg(debug_assertions)]
    #[test]
    fn run_on_worker_from_non_main_thread_asserts_in_debug() {
        miniextendr_runtime_init();

        let (tx, rx) = std::sync::mpsc::sync_channel::<Result<String, ()>>(1);
        std::thread::spawn(move || {
            let outcome =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_on_worker(|| 0i32)));
            let report = match outcome {
                Err(payload) => Ok(crate::unwind_protect::panic_payload_to_string(
                    payload.as_ref(),
                )
                .into_owned()),
                Ok(_) => Err(()),
            };
            let _ = tx.send(report);
        });

        match rx.recv_timeout(std::time::Duration::from_secs(5)) {
            Ok(Ok(msg)) => {
                assert!(
                    msg.contains("R main thread"),
                    "expected main-thread precondition assert, got: {msg}"
                );
            }
            Ok(Err(())) => {
                panic!("debug_assert did not fire when run_on_worker was called from non-main")
            }
            Err(_) => panic!("test deadlocked waiting for debug_assert panic"),
        }
    }
}
// endregion

// region: Feature-gated tests: no worker-thread (stubs)

#[cfg(not(feature = "worker-thread"))]
mod stub_tests {
    use super::*;

    #[test]
    fn stub_with_r_thread_inline() {
        miniextendr_runtime_init();
        // If another parallel test already set R_MAIN_THREAD_ID to a
        // different thread (OnceLock), we won't be "main" and with_r_thread
        // will rightfully panic. Skip in that case.
        if !is_r_main_thread() {
            return;
        }
        let result = with_r_thread(|| 42);
        assert_eq!(result, 42);
    }

    #[test]
    fn stub_run_on_worker_inline() {
        let result = run_on_worker(|| 123);
        assert_eq!(result, Ok(123));
    }

    /// Without `worker-thread`, `with_r_thread` must panic when called from
    /// a non-main thread.
    #[test]
    fn stub_with_r_thread_panics_on_wrong_thread() {
        miniextendr_runtime_init();

        let handle = std::thread::spawn(|| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| with_r_thread(|| 42)))
        });

        let result = handle.join().expect("thread panicked outside catch_unwind");
        assert!(
            result.is_err(),
            "with_r_thread should panic when called from a non-main thread \
                 without the worker-thread feature, but it ran inline silently"
        );
    }
}
// endregion
