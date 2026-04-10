# Windows R CMD check hang: DataFusion Tokio threads keep Rterm alive

## What was attempted

Fix the Windows CI hang where R CMD check hangs indefinitely at
`* checking tests ...` even though all tests pass.

## What went wrong

R CMD check on Windows uses `system2(stdout=TRUE)` to run test scripts via
pipes. When any thread in the test Rterm process keeps running after the
main thread finishes, the stdout pipe handle stays open, and `system2()`
waits forever.

## Root cause

**DataFusion's Tokio async runtime** spawns background threads that outlive
the R test execution. `tokio::runtime::Runtime::new()` creates a multi-threaded
runtime with one worker thread per CPU core. These threads keep the Rterm
process alive after tests complete. Binary search across all 88 test files
confirmed `test-datafusion.R` is the sole trigger.

### Red herrings investigated

1. **callr/processx orphan Rterm processes** — Only relevant when callr tests
   run (already skipped on Windows via pre-existing `skip_on_os`). Not the
   cause of the main hang.

2. **Worker thread (`worker-thread` feature)** — Investigated extensively
   (Mutex-based sender drop, atexit handler, recv_timeout, reg.finalizer).
   The worker thread CAN keep a process alive via blocking `recv()`, but
   it is NOT the trigger for the observed hang — DataFusion is.

3. **Rust panic under pipe redirection** — Worker panic tests cause
   `fatal runtime error: failed to initiate panic, error 5` (ACCESS_DENIED)
   when stdout is a pipe. This crashes (aborts) the process rather than
   hanging it — a separate issue.

## Fix

Switch the Tokio runtime from `Runtime::new()` (multi-threaded, spawns N
worker threads) to `Builder::new_current_thread().enable_all().build()` (zero
background threads — IO/time/scheduler all run inline during `block_on()`).

Verified via Tokio source audit (v1.50.0): `current_thread` with `enable_all()`
spawns no threads. The blocking pool only spawns threads on explicit
`spawn_blocking()` (DataFusion doesn't use it). `Drop` completes immediately.

Safety net: `_R_CHECK_TESTS_ELAPSED_TIMEOUT_=300` in CI kills the test process
after 5 minutes if something else causes a hang in the future.

## Future work

- Investigate the Rust panic "error 5" on Windows under pipe redirection.
  May need a custom panic hook that avoids writing to stderr when the handle
  is a pipe.
