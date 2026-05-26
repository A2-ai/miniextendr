# 2026-05-26 — Worker-channel contention audit

User flagged that the *channel back to R API on main thread* (the
worker → main routing in `miniextendr-api/src/worker.rs`) was originally
written to experiment with progress bars and similar features, and was
never put under contention. This note is the audit I did before writing
the stress test that lives alongside it.

## What the mechanism actually is

`worker.rs::worker_channel` (lines 280–680). Three channels, two thread-locals,
one mutex.

```
                       MAIN THREAD                           WORKER THREAD
                       ===========                           =============

  R_init_<pkg>:
    init_worker():
      mpsc::sync_channel::<WorkerMsg>(0)    ──▶ rx held by worker thread
      WorkerState { tx, JoinHandle } ──▶ in `static WORKER: Mutex<Option<_>>`

  run_on_worker(f):                                          rx.recv() ──┐
    sync_channel::<WorkerMessage<T>>(1)         worker_tx           ┌────┘
    sync_channel::<MainThreadResponse>(1)       response_rx         │
    AnyJob wrapping (f, worker_tx, response_rx)                     │
    WORKER.lock() clone tx                                          │
    tx.send(Job(job))    ─── rendezvous (cap 0) ────────────────▶   │
                                                                    │
                                                          job() runs:
                                                            TLS:
                                                              WORKER_TO_MAIN_TX
                                                                = Some(worker_tx)
                                                              MAIN_RESPONSE_RX
                                                                = Some(response_rx)
                                                            catch_unwind(f)
                                                              f may call with_r_thread:
                                                                worker_tx.send(WorkRequest)
                                                                response_rx.recv() ──┐
                                                                                     │
    loop {                                                                           │
      worker_rx.recv():                                                              │
        WorkRequest(work) ──◀──────────────────────────────────────────────          │
          R_UnwindProtect( trampoline=catch_unwind(work), cleanup=send Err ):        │
            ── trampoline ok ──▶ response_tx.send(Ok(boxed))   ───────────▶          │
            ── trampoline rust panic ──▶ response_tx.send(Err)  ──────────▶          │
            ── R longjmp ──▶ cleanup sends Err, panic_any(RErrorMarker)              │
                              outer catch_unwind catches → R_ContinueUnwind          │
                                                                                     │
                                                            f returns r              │
                                                            TLS cleared              │
                                                            worker_tx.send(Done)──▶  │
        Done(result) ──▶ return                                                      │
    }
```

Key properties of the design that I want to pin down with tests:

1. **Single worker** — capacity-0 rendezvous on the main→worker channel
   means at most one job is in flight; `dispatch_to_worker` blocks in `send`
   until the worker has picked it up via `recv`.
2. **Per-job channel pair** — `(worker_tx, response_rx)` is created fresh
   inside `dispatch_to_worker`, never shared between jobs.
3. **Thread-local routing context** — `WORKER_TO_MAIN_TX` /
   `MAIN_RESPONSE_RX` are set only on the worker thread. Threads that the
   user spawns *from inside* a worker job do **not** inherit it, so
   `with_r_thread` from them must panic with the documented
   "called outside of `run_on_worker` context" message.
4. **Bounded protocol** — capacity-1 buffers in each direction; one
   round-trip outstanding at a time per job. Pumping N round-trips per
   job is supposed to work, regardless of N.
5. **Re-entry detection** — already covered (`run_on_worker_reentry_panics_not_deadlocks`).
6. **Cleanup on R longjmp** — already covered structurally in
   worker.rs lines 548–574; running an end-to-end test of it is risky
   (see below) so it stays out of scope.

## What's covered today

- `worker::tests::sendable_is_send` — Send impl.
- `worker::tests::with_r_thread_panics_before_init` — guard rail.
- `worker::tests::has_worker_context_false_outside_worker` — TLS sanity.
- `worker::tests::worker_tests::run_on_worker_reentry_panics_not_deadlocks` —
  re-entry detection (not a deadlock).
- `worker::tests::stub_tests::*` — no-worker-thread feature stubs.
- `tests/worker_shutdown.rs` — `miniextendr_runtime_shutdown` blocks
  until the worker has actually joined.
- `tests/altrep_thread.rs::altrep_via_with_r_thread_from_worker` — one
  round-trip from inside a `run_on_worker` job.
- Indirect: `tests/rayon.rs` exercises `with_r_thread` from the worker
  on every allocation, but always at most one round-trip and never
  under contention.

## What's NOT covered (the user's intuition was right)

| # | Pattern | Why it matters |
|---|---|---|
| P1 | One job with **N inner `with_r_thread` calls** (N = 1k+) | Verifies the capacity-1 round-trip protocol doesn't lose messages or wedge under high churn. Progress-bar style "tick on main every iteration" usage. |
| P2 | **M concurrent threads** each call `run_on_worker` | Worker is a single bottleneck; we want all M jobs to complete, no deadlock, no message mix-up. |
| P3 | M threads using `std::sync::Barrier` to **race into `run_on_worker` simultaneously** | Forces maximum contention on `WORKER.lock()` and the rendezvous send. |
| P4 | **Mixed long + short jobs** — one job that sleeps inside `with_r_thread`, plus a flock of short jobs | Liveness check: short jobs must queue and finish; the long one doesn't starve anyone permanently. |
| P5 | **`with_r_thread` re-called from main thread** (short-circuit branch) | Cheap to verify but documents the main-thread fast-path. |
| P6 | **`with_r_thread` from a thread that is not the worker, with `worker-thread` feature on** | Documented to panic; should be exercised, not just claimed. |
| P7 | **Rust panic in the `with_r_thread` closure on main** | Trampoline catch_unwind → `data.panic_payload` → Err to worker → re-raise → outer catch_unwind in worker job → `Done(Err)`. |
| P8 | **`std::thread` spawned inside a `run_on_worker` job tries `with_r_thread`** | TLS is set on the worker thread only; the descendant has no routing context. Must panic, not deadlock. This is what stops rayon parallel iters from accidentally calling R from worker pool threads. |

Out of scope:
- **R longjmp from inside the closure on main** — `Rf_error` from a
  `with_r_thread` body would exercise the `cleanup_handler` path
  (worker.rs:548). The path is correct on inspection, but the test thread
  (spawned by `r_test_utils::with_r_thread`) has no outer R top-level
  handler, so `R_ContinueUnwind(token)` resumes an unwind that has
  nowhere to land. Triggering this in `cargo test` is more likely to
  crash the test process than exercise the code. Better to cover it from
  rpkg's testthat suite where R's own top-level handler is in place.
- **`miniextendr_runtime_shutdown` mid-job** — out of scope here
  (`worker_shutdown.rs` covers the idle case; mid-job shutdown is the
  package-unload-with-active-worker case and overlaps with #277).

## Test design

One integration binary `tests/worker_channel_stress.rs` so a wedged
channel doesn't take other tests down. Single top-level `#[test]`
function inside `r_test_utils::with_r_thread(|| { … })` running
sub-cases sequentially, matching the existing rayon/allocator pattern
(R can only initialise once per process).

Each sub-case has its own deadline via `recv_timeout` — a regression
that wedges the channel must surface as a test failure, never a hang.

Counts kept modest (N ≈ 200 round-trips, M ≈ 8 threads) so CI cost
stays low. They can be turned up locally for smoke runs.

## Finding: `run_on_worker` has an undocumented "main-thread caller" precondition

The first cut of this test tried to model contention by spawning M std
threads, each calling `run_on_worker` independently from inside the
test-util's R-main-thread closure. They all failed with

```
panic in `with_r_thread`: assertion failed: is_r_main_thread()
```

inside the closure passed to `with_r_thread`.

Tracing the implementation: `dispatch_to_worker` (worker.rs:419) runs
the main-thread event loop **on whatever thread called it**, and
`route_to_main_thread` (worker.rs:385) sends `WorkRequest` to that
caller's `worker_rx`. So `with_r_thread` from the worker job routes
back to the *caller of `run_on_worker`*, not to whichever thread holds
`R_MAIN_THREAD_ID`. If the caller isn't main, R API work lands on the
wrong thread silently.

This is reasonable for the design's intended use — `#[miniextendr]`
entry points are called via `.Call` which is always on R's main thread,
so in practice the caller IS the main thread. But it's an *implicit*
contract that isn't asserted anywhere and isn't called out in the
rustdoc.

Three options to make the contract explicit:

1. Add a `debug_assert!(is_r_main_thread())` at the top of
   `dispatch_to_worker`. Cheap, catches misuse in test builds, doesn't
   change release behaviour.
2. Promote it to a real `assert!` (release-too). Slightly more
   defensive but adds a per-dispatch atomic load.
3. Document the precondition in the `run_on_worker` rustdoc and on
   `with_r_thread`. Lowest effort, highest blast radius if someone
   misses it.

(1) or (1)+(3) is probably right. Out of scope for this PR; tracked
in #730 with the failing-test snippet as the repro.

## What I'm NOT changing

- No changes to `worker.rs`. This is a coverage exercise.
- No new public API.
- The precondition finding above will be filed as a separate issue,
  not addressed in this PR.
