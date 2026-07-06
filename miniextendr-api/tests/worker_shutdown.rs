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
//!
//! # Why this is ONE test function
//!
//! `miniextendr_runtime_init` is `Once`-guarded, so the worker spawns at
//! most once per process; the first `miniextendr_runtime_shutdown()` takes
//! it down for good (respawn requires re-`dyn.load`ing the DLL — fresh
//! statics — which a cargo test process can't do). Two `#[test]` fns
//! sharing that global therefore race under cargo's parallel test threads
//! (whichever shuts down first strands the other), and even serialized,
//! only the first one to run exercises a live worker. So the in-flight
//! scenario, the post-job wake-from-`recv()` join, and shutdown idempotency
//! are asserted in sequence in a single test.

mod r_test_utils;

use std::time::{Duration, Instant};

/// `miniextendr_runtime_shutdown` while a **pure-Rust** job is in flight,
/// followed by idempotent-shutdown assertions.
///
/// # What this characterises (issue #734)
///
/// #734 asks what happens when shutdown fires mid-job. The honest answer
/// depends entirely on *which thread* drives the in-flight job, and the
/// channel topology makes only one shape reachable in production:
///
/// - The main→worker channel is `sync_channel::<WorkerMsg>(0)` — a
///   **rendezvous** channel. A `send` blocks until the worker is actively
///   in `recv()`.
/// - While a job runs, the worker is inside `job()`, *not* in `recv()`.
///   So `shutdown()`'s `tx.send(WorkerMsg::Shutdown)` **blocks** until the
///   job finishes and the worker loops back to `recv()`.
/// - `JoinHandle::join()` then completes once the worker breaks out.
///
/// **Net behaviour for a pure-Rust job: `shutdown()` blocks until the
/// in-flight job completes, then joins.** It does NOT abort the job and
/// does NOT return early — so `R_unload_<pkg>` cannot unmap the DLL while
/// a pure-Rust worker job is still executing. No use-after-unmap window
/// for this shape.
///
/// The idle-worker join path (worker parked on `recv()`, `Shutdown` wakes
/// it) is exercised by the same sequence: the pending rendezvous `Shutdown`
/// lands exactly when the worker finishes the job and re-enters `recv()`.
///
/// # Why the issue's deadlock scenario is unreachable in production
///
/// The issue worries about `R_unload_<pkg>` firing mid-`with_r_thread`.
/// That can't happen on the same thread: R is single-threaded, and
/// `R_unload_<pkg>` only runs once every `.Call` has returned — i.e. once
/// every `run_on_worker` on the main thread has already received `Done`
/// and the worker is back parked on `recv()`. `dispatch_to_worker`
/// drives its event loop on the calling thread; the main thread cannot be
/// blocked in that loop *and* be executing `R_unload_<pkg>` at the same
/// time.
///
/// A genuine deadlock requires `shutdown()` on a *different* thread than
/// the one driving a `with_r_thread`-blocked job — which in turn requires
/// `run_on_worker` to have been dispatched from a non-main thread, already
/// a `debug_assert` violation (#730). So we deliberately do NOT exercise
/// that path here: it would hang (correctly — the comment in `shutdown()`
/// explains the no-timeout choice surfaces such hangs rather than masking
/// them), and a `with_r_thread`-from-shutdown round-trip belongs in an
/// rpkg testthat fixture with R's top-level handler in place, not a raw
/// cargo harness (#733).
///
/// This test drives the *reachable* shape: a pure-Rust job in flight,
/// shutdown from a concurrent thread, asserting shutdown waits for the job.
#[test]
#[cfg(feature = "worker-thread")]
fn shutdown_waits_for_in_flight_job_then_is_idempotent() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use miniextendr_api::worker::{is_r_main_thread, run_on_worker};

    // Signals the job sets when it has actually started running on the
    // worker, and a flag it sets right before it returns.
    let started = Arc::new(AtomicBool::new(false));
    let finished = Arc::new(AtomicBool::new(false));
    // Lets the test thread observe, after the fact, whether shutdown
    // returned before the job had marked itself finished.
    let job_done_before_shutdown_returned = Arc::new(AtomicBool::new(false));

    // The job must run via `run_on_worker` dispatched from the R main
    // thread (the #730 invariant). `dispatch_to_worker` then drives its
    // event loop on that same thread and blocks until `Done`. So we run
    // the dispatch on the R-test-main thread and fire `shutdown()` from a
    // separate observer thread once the job is confirmed running.
    let started_for_job = Arc::clone(&started);
    let finished_for_job = Arc::clone(&finished);

    // Spawn the observer that will call shutdown once the job is running.
    let started_obs = Arc::clone(&started);
    let finished_obs = Arc::clone(&finished);
    let done_before = Arc::clone(&job_done_before_shutdown_returned);
    let observer = std::thread::spawn(move || {
        // Wait until the job signals it is executing on the worker.
        let deadline = Instant::now() + Duration::from_secs(5);
        while !started_obs.load(Ordering::Acquire) {
            assert!(
                Instant::now() < deadline,
                "in-flight job never started within 5 s"
            );
            std::thread::yield_now();
        }

        // Job is now running on the worker (mid pure-Rust work). Call
        // shutdown from THIS thread, concurrent with the live job. The
        // rendezvous `tx.send(Shutdown)` must block until the worker
        // finishes the job and loops back to `recv()`, then `join()`.
        let start = Instant::now();
        miniextendr_api::worker::miniextendr_runtime_shutdown();
        let elapsed = start.elapsed();

        // Core characterisation: by the time shutdown returned, the job
        // must have finished — shutdown waited for the in-flight job
        // rather than racing the DLL unmap against a live worker.
        done_before.store(finished_obs.load(Ordering::Acquire), Ordering::Release);

        // Generous upper bound: the job sleeps ~300 ms; shutdown should
        // return shortly after that, well under the deadline. A regression
        // that wedges shutdown forever fails here instead of hanging CI.
        assert!(
            elapsed < Duration::from_secs(10),
            "shutdown did not return within 10 s with a pure-Rust job in flight \
             (took {elapsed:?}) — possible deadlock regression"
        );
    });

    // Drive the dispatch on the R main thread.
    r_test_utils::with_r_thread(move || {
        assert!(
            is_r_main_thread(),
            "dispatch must happen on the R main thread (#730 invariant)"
        );

        let r = run_on_worker(move || {
            // We are now ON the worker thread, inside an in-flight job.
            started_for_job.store(true, Ordering::Release);
            // Pure-Rust work only — NO `with_r_thread`. A bounded sleep
            // stands in for "computation still running" so the observer
            // thread has time to fire shutdown while we're live.
            std::thread::sleep(Duration::from_millis(300));
            finished_for_job.store(true, Ordering::Release);
            99i32
        });

        // The job completes normally even though shutdown was racing it:
        // shutdown's rendezvous `send` only lands once we return here and
        // the worker loops back to `recv()`.
        assert_eq!(r, Ok(99), "in-flight job result lost across shutdown race");
    });

    observer.join().expect("observer thread panicked");

    assert!(
        finished.load(Ordering::Acquire),
        "in-flight job never marked finished"
    );
    assert!(
        job_done_before_shutdown_returned.load(Ordering::Acquire),
        "shutdown returned BEFORE the in-flight pure-Rust job finished — \
         this is the #277 / #734 use-after-unmap window: the DLL could be \
         unmapped while the worker is still executing job code"
    );

    // Second shutdown on an already-shut-down runtime: `WORKER` is `None`,
    // so this must be a fast no-op (idempotency contract documented on
    // `worker_channel::shutdown`). Bounded so a regression that re-wedges
    // (e.g. a new cleanup path that blocks) fails loudly instead of hanging.
    let start = Instant::now();
    miniextendr_api::worker::miniextendr_runtime_shutdown();
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(2),
        "repeat miniextendr_runtime_shutdown did not return within 2 s \
         (took {elapsed:?}); it must be an idempotent no-op after the first call"
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
