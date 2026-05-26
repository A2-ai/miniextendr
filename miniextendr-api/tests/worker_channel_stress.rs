//! Contention tests for the worker → main routing channel.
//!
//! `with_r_thread` / `run_on_worker` (`miniextendr-api/src/worker.rs`) is
//! how Rust code calls R API entry points from a non-main thread.
//! Existing coverage exercises a *single* round-trip
//! (`altrep_thread::altrep_via_with_r_thread_from_worker`), shutdown
//! (`worker_shutdown.rs`), and re-entry detection
//! (`worker::tests::run_on_worker_reentry_panics_not_deadlocks`). This
//! binary covers the gaps: many round-trips, barrier-storms of
//! descendant threads, error-propagation, and the routing-context
//! boundary.
//!
//! Single integration binary by design — a wedged channel here must
//! not take other test binaries down. Every cross-thread `recv` is
//! `recv_timeout`-bounded so a regression that deadlocks fails loudly
//! rather than hanging CI.
//!
//! Run with:
//!
//!   MINIEXTENDR_BACKTRACE=1 \
//!     cargo test -p miniextendr-api --test worker_channel_stress \
//!                --features worker-thread -- --nocapture
//!
//! `MINIEXTENDR_BACKTRACE=1` is needed only when *debugging* a failure
//! — without it, `miniextendr_panic_hook` silently swallows panic
//! tracebacks (the framework's normal behaviour, since panics turn into
//! tagged R errors). The test failure itself still reports as
//! `FAILED`; only the panic detail is suppressed.

#![cfg(feature = "worker-thread")]

mod r_test_utils;

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::mpsc;
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use miniextendr_api::unwind_protect::panic_payload_to_string;
use miniextendr_api::with_r_thread;
use miniextendr_api::worker::{is_r_main_thread, run_on_worker};

/// Deadline for every cross-thread `recv` in this file. Long enough to
/// absorb slow-CI jitter, short enough that a real deadlock surfaces
/// well under a minute.
const RECV_DEADLINE: Duration = Duration::from_secs(30);

#[test]
fn worker_channel_stress_suite() {
    r_test_utils::with_r_thread(|| {
        // The r-test-main thread is the one that ran
        // `miniextendr_runtime_init()`, so it IS `is_r_main_thread()`.
        // Every `run_on_worker` in this file is dispatched from here —
        // that's the documented invariant: only the main thread runs
        // the main-thread event loop inside `dispatch_to_worker`.
        assert!(
            is_r_main_thread(),
            "test harness should run sub-cases on R main thread"
        );

        case_main_thread_short_circuit();
        case_recursive_with_r_thread_on_main();
        case_many_round_trips_one_job();
        case_descendant_thread_storm();
        case_main_thread_keeps_working_after_descendant_panics();
        case_rust_panic_in_main_closure();
        case_rust_panic_in_worker_body();
        case_with_r_thread_outside_run_on_worker_context();
    });
}

// region: Main-thread fast paths

/// `with_r_thread` from the main thread runs inline; no channel
/// touched.
fn case_main_thread_short_circuit() {
    let result = with_r_thread(|| {
        assert!(
            is_r_main_thread(),
            "closure must observe main-thread fast path"
        );
        7i32
    });
    assert_eq!(result, 7);
}

/// Nested `with_r_thread` from main — both calls short-circuit.
fn case_recursive_with_r_thread_on_main() {
    let outer = with_r_thread(|| {
        let inner = with_r_thread(|| {
            assert!(is_r_main_thread());
            5i32
        });
        inner * 2
    });
    assert_eq!(outer, 10);
}

// endregion

// region: Sustained round-trip protocol

/// One worker job that pumps N round-trips through the capacity-1
/// protocol. Verifies neither direction wedges and every message is
/// delivered in order (an ordered sum catches drops and reorderings
/// cheaply).
fn case_many_round_trips_one_job() {
    const N: i64 = 512;

    let result = run_on_worker(move || {
        assert!(
            !is_r_main_thread(),
            "worker job should run off the main thread"
        );

        let mut acc = 0i64;
        for i in 0..N {
            let v = with_r_thread(move || {
                assert!(
                    is_r_main_thread(),
                    "with_r_thread closure must land on R main thread"
                );
                i
            });
            acc += v;
        }
        acc
    });

    let expected: i64 = (0..N).sum();
    assert_eq!(result, Ok(expected), "round-trip sum mismatch");
}

// endregion

// region: Descendant thread storm

/// Worker job spawns M descendant threads, holds them at a Barrier,
/// then releases them all at once. Each descendant tries
/// `with_r_thread` — none has the routing TLS, so each must panic with
/// the documented "outside of `run_on_worker` context" message. The
/// worker's own thread is unaffected and the job completes.
///
/// This is the highest-contention shape that real workloads can
/// produce (rayon parallel iter inside a `#[miniextendr]` function
/// being the canonical example). The job ends up serialising any
/// R access either by (a) running it *outside* the parallel section
/// or (b) routing through `with_r_thread` from the worker's own
/// thread only.
fn case_descendant_thread_storm() {
    const M: usize = 16;

    let r = run_on_worker(|| {
        let barrier = Arc::new(Barrier::new(M));
        let (tx, rx) = mpsc::sync_channel::<bool>(M);

        for _ in 0..M {
            let b = barrier.clone();
            let tx = tx.clone();
            std::thread::spawn(move || {
                b.wait();
                let outcome = catch_unwind(AssertUnwindSafe(|| with_r_thread(|| 0i32)));
                let detected = match outcome {
                    Ok(_) => false,
                    Err(payload) => panic_payload_to_string(payload.as_ref())
                        .contains("outside of `run_on_worker` context"),
                };
                let _ = tx.send(detected);
            });
        }
        drop(tx);

        let deadline = Instant::now() + RECV_DEADLINE;
        let mut detected_all = true;
        for _ in 0..M {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match rx.recv_timeout(remaining) {
                Ok(b) => detected_all &= b,
                Err(_) => return Err("descendant storm wedged"),
            }
        }
        if !detected_all {
            return Err("at least one descendant did NOT panic with the documented message");
        }
        Ok(M)
    });

    assert_eq!(
        r,
        Ok(Ok(M)),
        "descendant thread storm did not behave as documented"
    );
}

/// After a wave of descendant panics, the worker's own thread can
/// still call `with_r_thread` successfully. Catches a regression where
/// a descendant panic somehow poisons the routing context (it
/// shouldn't — TLS lives on the worker thread, descendants never had
/// access — but exercise it explicitly).
fn case_main_thread_keeps_working_after_descendant_panics() {
    const M: usize = 8;

    let r = run_on_worker(|| {
        // First, spawn M descendants that try with_r_thread and panic.
        let (tx, rx) = mpsc::sync_channel::<()>(M);
        for _ in 0..M {
            let tx = tx.clone();
            std::thread::spawn(move || {
                let _ = catch_unwind(AssertUnwindSafe(|| with_r_thread(|| 0i32)));
                let _ = tx.send(());
            });
        }
        drop(tx);
        let deadline = Instant::now() + RECV_DEADLINE;
        for _ in 0..M {
            let remaining = deadline.saturating_duration_since(Instant::now());
            rx.recv_timeout(remaining).expect("descendant lost");
        }

        // Then verify the worker's own thread can still round-trip.
        let mut acc = 0i64;
        for i in 0..32 {
            acc += with_r_thread(move || {
                assert!(is_r_main_thread());
                i
            });
        }
        acc
    });

    let expected: i64 = (0..32).sum();
    assert_eq!(
        r,
        Ok(expected),
        "routing channel poisoned by descendant panics"
    );
}

// endregion

// region: Error propagation

/// Rust panic in the main-side closure: trampoline catches → response
/// sends Err → worker re-raises → outer catch_unwind in worker job →
/// `Done(Err)` → `run_on_worker` returns Err with the panic message.
fn case_rust_panic_in_main_closure() {
    let r = run_on_worker(|| with_r_thread(|| panic!("worker-channel-stress: boom-on-main")));
    let msg = r.expect_err("panic in `with_r_thread` must become Err");
    assert!(
        msg.contains("boom-on-main"),
        "panic message did not survive: {msg}"
    );
}

/// Panic directly in the worker job body, outside any `with_r_thread`.
/// The worker's `catch_unwind` catches it and `Done(Err)` returns.
fn case_rust_panic_in_worker_body() {
    let r = run_on_worker::<_, ()>(|| {
        panic!("worker-channel-stress: panic in worker body");
    });
    let msg = r.expect_err("panic in worker body must become Err");
    assert!(
        msg.contains("panic in worker body"),
        "worker-body panic message did not survive: {msg}"
    );
}

// endregion

// region: Routing-context boundary

/// A thread that is neither main nor a worker-job context calling
/// `with_r_thread` must panic with the documented message — never
/// silently wait, never run inline.
fn case_with_r_thread_outside_run_on_worker_context() {
    let handle = std::thread::spawn(|| catch_unwind(AssertUnwindSafe(|| with_r_thread(|| 0i32))));
    let r = handle.join().expect("spawned thread crashed unrecoverably");
    let payload =
        r.expect_err("with_r_thread from a non-main, non-worker thread must panic, not run inline");
    let msg = panic_payload_to_string(payload.as_ref());
    assert!(
        msg.contains("outside of `run_on_worker` context"),
        "wrong panic message: {msg}"
    );
}

// endregion
