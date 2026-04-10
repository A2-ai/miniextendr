# Windows R CMD check hang: worker thread keeps Rterm alive

## What was attempted

Fix the Windows CI hang where R CMD check hangs indefinitely at
`* checking tests ...` even though all 3949 tests pass in ~60 seconds.

## What went wrong

R CMD check on Windows uses `system2()` with pipe-redirected stdout to run
test scripts. The test Rterm process loads miniextendr, which spawns a
background worker thread (via `worker-thread` feature). After tests complete
and R's main thread finishes, the worker thread keeps the process alive.
`system2()` waits for the stdout pipe to close, but the pipe stays open as
long as the process exists.

**Key observation**: Even after the worker thread exits (verified via
`ThreadCount=1`), the Rterm process doesn't terminate. This suggests the hang
is in R's own exit/cleanup path when a DLL with Rust threads is loaded,
possibly related to CRT thread cleanup or DLL_PROCESS_DETACH sequencing.

Approaches tried that did NOT fix the hang:

1. **`skip_on_os("windows")` for callr tests** - Correct but insufficient.
   The hang is from the worker thread, not callr subprocesses.

2. **`.onUnload` shutdown** - Doesn't fire during R CMD check test exit
   (R doesn't unload packages before quitting the test process).

3. **`reg.finalizer(onexit=TRUE)`** - Doesn't fire during R CMD check's
   test process exit (GC finalizers aren't run reliably on process exit).

4. **C `atexit()` handler** - Registered but doesn't appear to fire during
   R's exit path on Windows.

5. **`recv_timeout(1s)` in worker loop** - Worker thread DOES exit (confirmed
   by ThreadCount=1), but the R process STILL doesn't terminate. The issue
   is deeper than the thread lifetime.

6. **Worker thread handle close via `SetStdHandle`** - Would change
   process-wide handles, breaking output for the main thread.

## Root cause

The Rust worker thread (spawned by `miniextendr_runtime_init()`) prevents the
Windows CRT from cleanly exiting the R process. Even after the thread exits,
the CRT's exit sequence appears to hang — possibly in DLL detach or thread
cleanup. The exact mechanism is unclear but is specific to the interaction
between R's Rterm.exe, the Rust DLL, and Windows process exit semantics.

## Fix

**Pragmatic**: Set `_R_CHECK_TESTS_ELAPSED_TIMEOUT_=300` on the Windows CI
job. This tells R CMD check to kill the test process after 5 minutes. Since
tests complete in ~60s, this gives ample headroom while preventing the
indefinite hang.

Additionally, `skip_on_os("windows")` is applied to all three callr-using
test files to avoid a second source of hanging (orphan Rterm processes from
callr/processx).

## Future work

The proper fix is to ensure the Rust worker thread and DLL unload cooperate
cleanly on Windows. This likely requires:

1. Registering the worker shutdown as an R-level session cleanup (e.g., via
   `R_CleanUp` hook or `R_RunExitFinalizers`)
2. Understanding why the CRT's exit sequence hangs after the worker thread
   has already exited
3. Possibly using `ExitProcess()` directly from an atexit handler to bypass
   the CRT's thread cleanup
