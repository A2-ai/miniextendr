//! Integration test for the cooperative worker shutdown path (#204).
//!
//! PR #199 (closes #103) added `recv_timeout(250ms)` + `WORKER_SHOULD_STOP:
//! AtomicBool` + `miniextendr_runtime_shutdown()`. The previous worker tests
//! exercised dispatch + re-entry guarding, but never the actual shutdown
//! transition. This test:
//!
//! 1. Initializes the runtime (spawns the worker).
//! 2. Sends `miniextendr_runtime_shutdown()`.
//! 3. Polls `miniextendr_runtime_join_for_test` until the worker exits or
//!    2 s elapse — roughly 8 shutdown polls at the 250 ms poll interval.
//!
//! Lives in its own integration binary (each `tests/*.rs` file becomes one)
//! so taking the worker down here does not disrupt other test binaries that
//! rely on a live worker.

mod r_test_utils;

use std::time::Duration;

#[test]
#[cfg(feature = "worker-thread")]
fn worker_exits_after_runtime_shutdown() {
    // Boot R + the miniextendr runtime (this spawns the worker thread under
    // the `worker-thread` feature). We don't need to actually dispatch a job:
    // the worker is parked on `recv_timeout` from the moment init completes,
    // which is exactly the state where #103's deadlock would manifest.
    r_test_utils::with_r_thread(|| ());

    miniextendr_api::worker::miniextendr_runtime_shutdown();

    let joined = miniextendr_api::worker::miniextendr_runtime_join_for_test(Duration::from_secs(2));

    assert!(
        joined,
        "worker did not exit within 2 s after miniextendr_runtime_shutdown; \
         the cooperative shutdown path from #103 appears broken"
    );
}

#[test]
#[cfg(not(feature = "worker-thread"))]
fn shutdown_is_noop_without_worker_thread_feature() {
    // Without the feature there is no worker; shutdown + join are both no-ops.
    miniextendr_api::worker::miniextendr_runtime_shutdown();
    assert!(miniextendr_api::worker::miniextendr_runtime_join_for_test(
        Duration::from_millis(10)
    ));
}
