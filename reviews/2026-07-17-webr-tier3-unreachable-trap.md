# webR tier-3 testthat pass: `unreachable` trap (main-push red since 2026-07-11)

## What was attempted

Every main-push run of `webr.yml`'s tier-3 informational testthat pass
(#1255/#1258) died with:

```
[tier3][testthat] FAIL: harness error before counts: unreachable
```

`unreachable` is a WebAssembly trap: the R/wasm runtime aborted mid-`test_dir`,
the R worker died, and the harness (correctly) turned the missing counts line
into a red gate. The failure was first attributed to the most recent merge
(#1363's streaming-ALTREP `get_region` forward), but the run history showed
the step had **never** been green: the first failure is the merge commit of
#1258 itself (2026-07-11T19:34Z, run 29165513537), the PR that added the
testthat pass. `webr.yml`'s testthat leg only activates on main-push /
workflow_dispatch, so the PR could not pre-verify it.

## What went wrong

Two independent framework bugs, both invisible on native targets, both fatal
under webR because Emscripten implements `longjmp` and unwinding as **wasm
exceptions** (webR builds R with `-fwasm-exceptions -s SUPPORT_LONGJMP=wasm`;
Rust's `wasm32-unknown-emscripten` target defaults to `panic=unwind` over
native wasm EH):

1. **`miniextendr_assert_utf8_locale` was `extern "C"` (non-unwind) but
   raises via `Rf_error`.** Under wasm the longjmp is an exception that
   unwinds through the frame; rustc's abort guard on non-unwind ABIs turns it
   into an `unreachable` trap. Natively, `longjmp` never touches landing
   pads, so the bug was undetectable. First alphabetical hit:
   `test-encoding.R`'s mid-session locale-flip tests calling
   `assert_utf8_locale_now()` — this was the trap that killed every main-push
   run. (`miniextendr_encoding_init`, its sibling, was already `"C-unwind"`.)

2. **`run_on_worker`'s inline branch (wasm builds, or `worker-thread` off)
   ran the closure bare** — `Ok(f())` with no catch — while the worker path
   catches panics at the channel and returns `Err(msg)`. A panicking closure
   therefore unwound out of raw `extern "C-unwind"` `.Call` fixtures that
   rely on the `Err` contract, escaped R entirely (R has no handler for the
   Rust exception tag), and reached Node as an uncaught
   `WebAssembly.Exception` ("wasm exception"). Hit by `test-worker.R`
   (`unsafe_C_test_worker_panic_simple`) and `test-errors-more.R`
   (`unsafe_C_worker_drop_on_panic`).

## How it was diagnosed

Local Docker repro was infeasible (daemon down; Docker VM at 28 GB with 11 GB
host disk free — the documented disk-exhaustion crash mode). Instead, a
throwaway diagnostic branch instrumented `smoke.mjs` and was driven via
`workflow_dispatch` (which enables `SMOKE_TESTTHAT=1`):

- **Error-path probes in isolation**: plain R error, conversion error, pure
  Rust panic, heap-drop panic, direct `Rf_error` longjmp through
  `with_r_unwind_protect`. All five passed, proving the panic → tagged
  condition transport and R-longjmp-through-Rust-frames both work under wasm
  EH (the `__c_longjmp` tag passes through Rust's tag-filtered `catch_unwind`
  pads without being caught, matching native bypass semantics).
- **Per-file sweep with session reboot**: ran all 137 test files one at a
  time via `test_dir(filter = ...)`, rebooting the webR session after each
  trap. Result: 134/137 files clean; only `test-encoding.R` (trap:
  `unreachable`), `test-errors-more.R` and `test-worker.R` (trap: `wasm
  exception`). Diagnostic runs: 29535733978 (localization),
  29540422832 (fix verification + per-call probes).

## Root cause

Non-unwind `extern "C"` ABI on a longjmp-ing export, and a panic-contract gap
on `run_on_worker`'s inline path. Both are unwinding-boundary bugs that only
materialize when longjmp/unwind is exception-based (wasm EH); native longjmp
skips frames and native panics never cross those particular boundaries in
tested configurations.

## Fix

`miniextendr-api/src/encoding.rs`: `miniextendr_assert_utf8_locale` →
`extern "C-unwind"`. `miniextendr-api/src/worker.rs`: inline `run_on_worker`
branch now `catch_unwind`s and returns `Err(message)` with the same
stringification rules as `worker_channel::fold_panic_message` (RCondition
verbatim, generic panics with `(at file:line)`), plus panic telemetry parity.

Verified in dispatch run 29540422832 (same instrumented harness, rebuilt
from the fix branch): all 16 error-path probes report `caught: ...` —
including `encoding-locale-flip` (previously the `unreachable` trap, now
"caught: miniextendr requires a UTF-8 locale") and every worker-panic
probe (previously uncaught `WebAssembly.Exception`s) — and the per-file
sweep reports **0/137 files trapped** (pre-fix: 3/137). Post-fix suite
aggregate under wasm: passed=6846, failed=5, skipped=41, errors=100
(ordinary informational failures: callr/subprocess-class
incompatibilities, not traps).

## Lessons

- A wasm trap message tells you the failure class: `unreachable` = rustc
  abort guard / `core::intrinsics::abort`; "wasm exception" = an uncaught
  exception reaching the JS host. They are different bugs.
- `extern "C"` vs `extern "C-unwind"` is load-bearing on wasm even for code
  that "only longjmps" — audit non-unwind boundaries for `Rf_error`/panic
  reachability when touching webR paths.
- `workflow_dispatch` on a diagnostic branch is the cheap empirical loop for
  main-push-only CI legs: per-call probes plus a per-file sweep with session
  reboots localized three traps in one run.
