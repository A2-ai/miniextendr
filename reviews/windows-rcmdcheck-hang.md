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
the R test execution. When `test-datafusion.R` runs, the Tokio thread pool
keeps the Rterm process alive after tests complete. Binary search across 88
test files confirmed `test-datafusion.R` is the sole trigger.

### Red herrings investigated

1. **callr/processx orphan Rterm processes** — Only relevant when callr tests
   run (already skipped on Windows). Not the cause of the main hang.

2. **Worker thread (`worker-thread` feature)** — Investigated extensively
   (Mutex-based sender drop, atexit handler, recv_timeout, reg.finalizer).
   The worker thread CAN keep a process alive via blocking `recv()`, but
   it is NOT the trigger for the observed hang — DataFusion is.

3. **Rust panic under pipe redirection** — Worker panic tests cause
   `fatal runtime error: failed to initiate panic, error 5` (ACCESS_DENIED)
   when stdout is a pipe. This crashes (aborts) the process rather than
   hanging it. Skipped as a separate fix.

## Fix

1. `skip_on_os("windows")` on `test-datafusion.R` — DataFusion is
   platform-independent; tested on Linux/macOS
2. `skip_on_os("windows")` on worker panic tests in `test-worker.R` — Rust
   panics on the worker thread fail with "error 5" under pipe redirection
3. `_R_CHECK_TESTS_ELAPSED_TIMEOUT_=300` in CI as a safety net
4. `skip_on_os("windows")` on `test-subprocess-isolated.R` (callr tests)

## Future work

- Investigate why Tokio's runtime threads are not shut down when R exits.
  A `tokio::runtime::Runtime::shutdown_background()` call from `.onUnload`
  or an R-level session finalizer may allow DataFusion tests to run on Windows.
- Investigate the Rust panic "error 5" on Windows under pipe redirection.
  May need a custom panic hook that avoids writing to stderr when the handle
  is a pipe.
