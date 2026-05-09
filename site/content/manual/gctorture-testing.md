+++
title = "GC-torture testing — surfacing latent SEXP-protection bugs"
weight = 50
description = "R provides gctorture(TRUE) and gctorture2(step, wait, inhibit_release) to make every allocation trigger a full GC. Any SEXP that's reachable through Rust state but not rooted in R's protect mechanism gets collected on the next allocation, surfacing use-after-free that would otherwise hide for thousands of test runs (or only manifest under stricter allocators like glibc R 4.6 release on CI)."
+++

R provides `gctorture(TRUE)` and `gctorture2(step, wait, inhibit_release)` to make every
allocation trigger a full GC. Any SEXP that's reachable through Rust state but not rooted
in R's protect mechanism gets collected on the next allocation, surfacing use-after-free
that would otherwise hide for thousands of test runs (or only manifest under stricter
allocators like glibc R 4.6 release on CI).

This page documents the harness pattern that works in this codebase, common pitfalls,
and a recipe for running it across the full `rpkg/` test suite.

## When to use

- A new `unsafe` block reads or writes an SEXP cached in Rust state.
- A new entry-point allocates SEXPs that linger in `Vec<SEXP>` / `Box<dyn Any>` / similar
  containers across `.Call` boundaries, ALTREP callbacks, or background threads.
- CI fails with `malloc(): unsorted double linked list corrupted`, `*** caught segfault ***`,
  or `address 0x... 'invalid permissions'`, particularly on Linux R release but not on
  devel/oldrel — that's the strict-allocator signature for a UAF that other runtimes
  tolerate.
- Before merging anything that touches `serde/columnar.rs`, ALTREP code, ExternalPtr
  storage, or the worker-thread serializer.

## The two pitfalls

### 1. Don't enable gctorture before `library(miniextendr)`

`gctorture(TRUE)` is so aggressive that ordinary package-load paths (`initMethodDispatch`,
S4 method registration, `loadNamespace` hooks) hit unprotected SEXPs in unrelated CRAN
packages and crash before your tests start. The crash signature is the giveaway —
traceback ends in `library` / `loadNamespace` / `runHook(".onLoad", ...)`.

**Always** load the package first, then flip gctorture on:

```r
library(miniextendr)
library(testthat)
gctorture(TRUE)
# ... call your code ...
gctorture(FALSE)
```

### 2. `test_dir` loads more packages

`testthat::test_dir` may attach extra packages (rlang, withr, dplyr, …) the first time
it runs. Each `.onLoad` hook is a gctorture-amplified hazard you don't want to debug.
For targeted testing, **call exported functions directly via `get(name)()`** rather than
going through `test_dir`. Keep `test_dir` for the final whole-suite sweep, after you've
loaded everything you need.

## Recipe — per-function sweep

```r
library(miniextendr)
gctorture(TRUE)

funs <- ls("package:miniextendr", pattern = "^test_columnar_")  # or any prefix
ok <- 0L
fail <- character(0)
for (f in funs) {
  res <- tryCatch({ get(f)(); "ok" }, error = function(e) conditionMessage(e))
  if (identical(res, "ok")) ok <- ok + 1L
  else fail <- c(fail, sprintf("%s: %s", f, res))
}
gctorture(FALSE)
cat(sprintf("%d/%d ok\n", ok, length(funs)))
if (length(fail)) cat(fail, sep = "\n")
```

This is the cheapest mode: ~1 minute per simple function under `gctorture(TRUE)`.
Catches every `Rf_allocVector` → unprotected-SEXP-read sequence inside the call.

Crash interpretation:
- **`exit=0` with truncated output** — the script SIGSEGV'd silently. The last printed
  function header is the one that crashed. Re-run that single function in isolation to
  confirm.
- **Explicit `*** caught segfault ***` with traceback** — R's signal handler caught it.
  Read the traceback for the immediate cause; the *root* cause is usually one frame up
  (the allocation that triggered GC, not the one that segfaulted reading freed memory).
- **`Rscript` exits with backtrace and address `0x...` low** — wild pointer from a freed
  SEXP, often from `Rf_eval` or `R_NilValue`-adjacent reads.

## Recipe — full `rpkg/` sweep (slow)

For the full suite, use `gctorture2(step = N)` with `N > 1` to keep the run tractable.
`step = 100` is ~100× slower than baseline (vs. ~1000× for `gctorture(TRUE)`) and still
surfaces the vast majority of UAF bugs.

```r
library(miniextendr)
library(testthat)
setTimeLimit(Inf, Inf, transient = FALSE)
options(timeout = Inf)

gctorture2(step = 100, wait = 0, inhibit_release = FALSE)
res <- test_dir(
  "rpkg/tests/testthat",
  reporter = ProgressReporter,
  stop_on_failure = FALSE
)
gctorture2(step = 0)  # disable
print(res)
```

Run it as `Rscript /path/to/script.R 2>&1 > /tmp/gctorture-full.log`, in the background,
and watch the log. Expect hours, not minutes — the full suite at `step = 100` typically
takes 30–90 minutes locally and substantially longer in CI.

For a tighter feedback loop while bisecting:
- `step = 10` covers most bugs in <30s/test.
- `step = 1` matches `gctorture(TRUE)` and only makes sense for single-function bisects.

## What gctorture cannot find

- **Stack-discipline violations on the protect stack** — `Rf_protect` / `Rf_unprotect`
  imbalances surface as panics from `R_PPStackTop` overflow, not gctorture firings.
- **Cross-`.Call` lifetime bugs** that depend on R-side caching (e.g., an ExternalPtr
  that survives one call only because R's namespace cached the wrapping closure).
- **Thread-affine bugs** — gctorture is single-threaded and won't reproduce data races
  across the worker thread.
- **ALTREP materialization order issues** — gctorture stresses GC, not lazy-eval ordering.

## Project-specific reference

- Root cause example for this technique: PR #424 (issue #307) — `ColumnBuffer::Generic`
  held raw `Vec<Option<SEXP>>` across allocations. The fix was a `ProtectScope` opened
  in `from_rows` and threaded through `ColumnFiller`. CI surfaced the bug only on Linux
  R 4.6 release; gctorture surfaced it deterministically in a 30-second local run.
- Helpers in `miniextendr-api/src/gc_protect.rs` — `OwnedProtect`, `ProtectScope`,
  `ProtectIndex`, `ReprotectSlot`, `Root`. See module docs for trade-offs.
- Pool variant for any-order release: `miniextendr-api/src/protect_pool.rs`
  (`ProtectPool` — VECSXP-backed, generational keys).

## Adding to CI

A nightly `gctorture-nightly` job invoking the full-suite recipe at `step = 100` would
catch this class of bug before it reaches a release. **Not yet wired up** — the Linux
release runner needs ~2× current timeout. Tracked at TODO (file an issue when adding).
