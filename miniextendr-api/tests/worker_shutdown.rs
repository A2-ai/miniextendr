//! Integration test for the worker shutdown path.
//!
//! `miniextendr_runtime_shutdown()` must block until the worker thread has
//! actually exited. Package unload (`R_unload_<pkg>` → shutdown) returns
//! right before `library.dynam.unload` unmaps the DLL's code pages — a
//! still-live worker that wakes up in freed memory corrupts SEH state and
//! produces "failed to initiate panic, error 5" across the process (#277).
//!
//! The old design polled an `AtomicBool` from a `recv_timeout(250ms)` loop
//! and joined via `JoinHandle::is_finished()` polling. It's been replaced
//! with a tagged `WorkerMsg::{Job, Shutdown}` channel and a blocking
//! `JoinHandle::join()`. This test exercises the new path.
//!
//! Lives in its own integration binary (each `tests/*.rs` file becomes one)
//! so taking the worker down here does not disrupt other test binaries that
//! rely on a live worker.

mod r_test_utils;

use std::time::{Duration, Instant};

#[test]
#[cfg(feature = "worker-thread")]
fn worker_exits_after_runtime_shutdown() {
    // Boot R + the miniextendr runtime (this spawns the worker thread under
    // the `worker-thread` feature). We don't need to actually dispatch a job:
    // the worker is parked on `recv()` from the moment init completes, which
    // is exactly the state where the DLL unload race would manifest.
    r_test_utils::with_r_thread(|| ());

    // `shutdown` sends `WorkerMsg::Shutdown`, drops the sender, and blocks
    // on `JoinHandle::join()`. With no jobs in flight the worker wakes from
    // its `recv()` immediately, matches `Shutdown`, and exits; the join
    // completes in microseconds.
    //
    // We still bound the call at a generous deadline so a regression that
    // makes shutdown wedge (e.g. a new thread-local cleanup path that
    // deadlocks) fails loudly instead of hanging the test suite.
    let start = Instant::now();
    miniextendr_api::worker::miniextendr_runtime_shutdown();
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_secs(2),
        "miniextendr_runtime_shutdown did not return within 2 s (took {elapsed:?}). \
         The shutdown path must block until the worker has joined — if it hangs, \
         R_unload_<pkg> will hang devtools::test / library.dynam.unload."
    );
}

#[test]
#[cfg(not(feature = "worker-thread"))]
fn shutdown_is_noop_without_worker_thread_feature() {
    // Without the feature there is no worker; shutdown should return
    // essentially instantly (it only uninstalls the panic hook).
    let start = Instant::now();
    miniextendr_api::worker::miniextendr_runtime_shutdown();
    assert!(start.elapsed() < Duration::from_millis(100));
}
