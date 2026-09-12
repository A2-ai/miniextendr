# Deferred callback boundaries (#1518)

The issue proposed draining deferred conditions after `with_r_unwind_protect_sourced`.
Tracing the existing code exposed two additional requirements: ALTREP callbacks can
return a fresh unprotected SEXP, and an R-origin longjmp bypasses the normal return
path. Signalling allocates and invokes arbitrary R handlers, so it must root a SEXP
result and retain Rust outcomes inside the signalling unwind guard.

Queue marks also refine the original report: conditions queued outside every boundary
can remain stranded below the next entry point's mark. Conditions queued inside an
enclosing .Call but outside a callback-specific boundary are drained by that enclosing
call. Neither behavior gives the callback its own delivery point.

The implementation drains each guarded callback locally, preserves the Rust panic's
location before handlers can re-enter Rust, and truncates an abandoned queue before
`R_ContinueUnwind`. SEXP-returning ALTREP trampolines retain a protection guard through
signalling (C NULL fallback results are passed through). Generic low-level guards
require rooted handles for SEXP-bearing outcomes, documented on the API.

R source inspection (`background/r-svn/src/main/memory.c`, `RunFinalizers`) confirms
that finalization saves and clears `R_HandlerStack` and `R_RestartStack`, establishes
an isolated top-level context, and restores the caller afterward. External-pointer
finalization and connection close/destroy/Drop therefore suppress deferred conditions,
including nested guarded work. Connection I/O signals after releasing its mutable
Rust state borrow. Identical queue entries remain distinct and ordered.

The first runtime fixture build failed because `RCall::arg` takes a SEXP, not a Rust
string. The fixture now converts and roots the message before building the call.
No runtime implementation change was needed for that compiler diagnostic.

Suppression is thread-local: a worker can queue diagnostics concurrently with an
R-thread visit that triggers finalization. A process-wide suppression flag would
lose those unrelated worker entries. A unit test queues on another thread while
the current thread is suppressed and verifies only the worker entry survives.

Runtime verification passed the deferred-condition, ALTREP-condition, connection,
FFI-guard and worker-longjmp files, with GC stress enabled. This includes nested
R error recovery, condition-handler re-entry, exiting handlers on SEXP results,
`warn = 2`, repeated identical conditions, finalizer suppression, and the no-argument
GC fixture. The worker suite also survived 2,000 longjmp/reuse cycles.

Final review of `RCustomConnection::build` found another allocation window: it called
open before rooting the SEXP returned by `R_new_custom_connection`. R source
`src/main/connections.c` confirms that function unprotects the result before returning.
Since open can now signal, the builder protects its connection until it returns. A
regression forces GC from the open-condition handler; a no-argument fixture exercises
open and close under the automatic GC sweep. An additional ALTREP test covers a
queued-condition callback returning C NULL to request R's sum fallback.

The first full vendored tarball check (before the connection-open follow-up) passed
with 0 errors, 0 warnings and one CRAN incoming note: new submission and tarball size
(24,698,775 bytes). Its full R tests and examples passed; GC stress was run separately
in the targeted files. The final targeted run also passed the connection-open GC regression and the C NULL sum-fallback regression (124 deferred-condition expectations), plus the ALTREP, connection, FFI-guard and 2,000-cycle worker-longjmp suites.

The fixtures share a generated Rd page. Reusing `fail` for a numeric ALTREP mode and
for the nested guard's boolean flag overwrote the older Result fixture's argument
description. The new parameters are named `mode` and `panic_inner`; named-argument
calls in the R regressions verify their wrappers, and roxygen regenerates all three
argument descriptions independently.
