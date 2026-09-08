# #1425: structured worker conditions became generic panics

Attempted: signal error, warning, message, and custom conditions both directly
on the worker and inside with_r_thread callbacks. Preserve classes, nested and
NA-bearing vector data, message, call attribution, and worker reuse.

Failure: the regression produced 48 failures across eight conditions. Both
channels erased RCondition to String, losing kind/classes/data and reporting
intentional conditions as generic panics. Telemetry counted every condition.
The test's initial call expectation also needed named arguments to match
the wrapper's match.call() attribution.

Fix: carry owned WorkerError::Condition(RCondition) or WorkerError::Panic(String)
through both channels and the inline fallback. Fold generic panic locations on
the origin thread; resume relays without firing another panic hook. Only generic
panics emit Worker telemetry. Generated wrappers use the existing typed
with_r_unwind_protect conversion for conditions and the already-reported panic
message for generic failures. The raw R longjmp continuation is unchanged.

Regression coverage includes all four condition kinds on both R-facing paths,
exact metadata and payloads, warning/message muffling, one telemetry report per
generic panic, source locations, and worker reuse. Rust channel tests exercise
the actual worker/callback route, and inline tests preserve all four variants
without enabling the worker thread.

Review also found that generic panics during pre-dispatch input conversion
emitted no telemetry. Route the outer wrapper catch through
with_r_unwind_protect, which reports a fresh generic panic once and preserves
typed conditions. Keep returned WorkerError::Panic values on their already-
reported path. A custom-conversion fixture checks both cases.

The first fixed-runtime test exposed incorrect expectations in the new test:
the shared R helper gives messages a trailing newline and NULL call, and custom
conditions include simpleCondition. Match that existing R-facing contract.
Worker-default validation also requires no_worker on fixtures that manually
dispatch, directly materialize R condition data, or explicitly test main-thread
R errors; otherwise the feature changes what those fixtures exercise.

Validation: the condition/worker/panic/feature-default R selection passed 409
assertions in default mode and 410 with the CI-detected feature set plus
worker-default, with zero failures or warnings in both. The unavailable
growth-debug feature accounts for one explicit skip in each run. Full Rust
workspace/cross-package/UI tests, the worker-enabled channel and shutdown
integration binaries, and 520 macro unit tests passed.
