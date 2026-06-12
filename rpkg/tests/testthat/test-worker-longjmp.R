# Pin down the R-longjmp-inside-with_r_thread-from-a-worker-job path (#733).
#
# Follow-up to #731. The `worker_channel_stress.rs` cargo suite cannot safely
# cover an R error raised inside a `with_r_thread` closure driven from a
# `run_on_worker` job: the cargo-test R embedding has no top-level handler, so
# the `R_ContinueUnwind` in `worker.rs`'s `cleanup_handler` would resume an
# unwind with nowhere to land (likely segfault). Under testthat, R's normal
# top-level error handler is in place, so the resumed unwind lands cleanly.
#
# CHARACTERISATION (answers the #733 open question on class layering):
# the surfaced condition is a BARE `simpleError`, NOT a `rust_error`. The
# `cleanup_handler` calls `R_ContinueUnwind(token)`, which resumes the
# *original* R `Rf_error` longjmp directly to R's top level on the main thread.
# That bypasses the macro's tagged-condition transport (`make_rust_condition_
# value` → `rust_error`) entirely — the `run_on_worker` call in the fixture
# never returns, so the fixture's downstream `panic!` is dead code. This is
# the same shape as `test-worker.R`'s existing raw-`C_*` `Rf_error` cases,
# which only asserted on message text; here we additionally assert the class.
#
# Skipped on Windows, matching the sibling R-error worker tests in
# `test-worker.R`: R longjmp through worker-thread boundaries on Windows
# bypasses testthat's `expect_error` handler.

test_that("R error inside with_r_thread from a worker job surfaces (as a bare simpleError)", {
  skip_on_os("windows")

  # The call must error.
  expect_error(miniextendr:::gc_stress_with_r_thread_stop())

  err <- tryCatch(
    miniextendr:::gc_stress_with_r_thread_stop(),
    error = function(e) e
  )

  # Characterised behaviour: the resumed R longjmp lands at top level as the
  # original `Rf_error`, so the condition is a plain simpleError — it does NOT
  # carry the framework's `rust_error` class (the macro transport is bypassed).
  expect_s3_class(err, "simpleError")
  expect_true(inherits(err, "error"))
  expect_false(inherits(err, "rust_error"))

  # The original `Rf_error` message threads straight through to top level
  # (not the `cleanup_handler`'s generic fallback): the condition is non-empty
  # and carries the message raised inside the `with_r_thread` closure.
  expect_true(nzchar(conditionMessage(err)))
  expect_match(
    conditionMessage(err),
    "R error inside with_r_thread from a worker job"
  )
})

# Worker re-usability after an R longjmp (#931, follow-up to #733).
#
# After `R_ContinueUnwind` tears the original `Rf_error` longjmp through a worker
# job mid-`with_r_thread` straight to R's top level, is the worker thread still
# usable? The worker's `recv()` loop (`worker_loop` in `worker.rs`) must have
# re-armed, its per-job thread-local channels cleared, and its rendezvous channel
# left un-wedged. We can't inspect that state directly; the honest proof is to
# dispatch a *second*, normal `run_on_worker` job and assert it returns correctly.
#
# `gc_stress_worker_roundtrip()` is that second job: it does
# `run_on_worker(|| with_r_thread(|| 1000) + with_r_thread(|| .. + 234))`, so a
# correct round-trip returns 1000 + 1234 = 2234. A poisoned worker would hang
# (test timeout), panic, or return a wrong value.
test_that("worker is reusable after an R longjmp tears through a job (#931)", {
  skip_on_os("windows")

  # Sanity: the round-trip works on a fresh worker.
  expect_identical(miniextendr:::gc_stress_worker_roundtrip(), 2234L)

  # Trigger the longjmp through a worker job; it resumes to top level and we
  # catch it here, exactly as a user's tryCatch would.
  err <- tryCatch(
    miniextendr:::gc_stress_with_r_thread_stop(),
    error = function(e) e
  )
  expect_s3_class(err, "simpleError")

  # The worker must still be usable for a normal job afterward. If the recv loop
  # were dead or the channel wedged, this would not return 2234L (it would hang
  # or error). Run it twice to confirm the worker keeps serving jobs.
  expect_identical(miniextendr:::gc_stress_worker_roundtrip(), 2234L)
  expect_identical(miniextendr:::gc_stress_worker_roundtrip(), 2234L)
})

# Characterise the ~8-byte longjmp leak (#931, follow-up to #733).
#
# `with_r_unwind_protect` (and the worker.rs main-thread loop's own
# `R_UnwindProtect`) leaks ~8 bytes (an `RErrorMarker` + `Box` header) on each
# `R_ContinueUnwind` path, because the cleanup handler can't reclaim them via
# `Box::from_raw` before R resumes the longjmp (see `miniextendr-api/CLAUDE.md`
# gotchas / MXL300 rationale). Each `gc_stress_with_r_thread_stop()` cycle drives
# exactly one such unwind.
#
# This is a CHARACTERISATION test, NOT a byte-bound assertion. We deliberately do
# NOT assert on process RSS: it is dominated by noise that swamps an 8-byte leak
# by three-plus orders of magnitude. Empirically, 5000 longjmp-through-tryCatch
# cycles grow RSS by ~50 MB (~11 KB/cycle) — but that growth is R-side garbage
# (condition objects, captured calls, restart frames) plus glibc/jemalloc arenas
# the allocator keeps mapped after free, none of which `gc()` returns to the OS.
# The true Rust-side leak (~8 bytes/cycle ≈ 40 KB over 5000) is invisible at page
# granularity. A hard RSS ceiling here would be flaky (it tripped a 32 MB bound
# on the very first run) — worse than a documented bound, per the #931 plan.
#
# So we drive N cycles and assert only the two facts that ARE reliable:
#   (a) the process survives all N unwinds (no segfault / abort), and
#   (b) the worker is still usable for a normal job afterward.
# The RSS delta is recorded via `message()` (visible in CI logs / with the
# "location" reporter) purely for documentation; the leak rate itself is pinned
# in `miniextendr-api/CLAUDE.md`. We use N = 2000 to keep the run fast while
# still accumulating any plausible gross regression.
test_that("repeated worker R longjmps don't crash and leave the worker usable (#931)", {
  skip_on_os("windows")
  skip_on_cran()

  proc_rss_kb <- function() {
    pid <- Sys.getpid()
    out <- tryCatch(
      suppressWarnings(system2("ps", c("-o", "rss=", "-p", pid), stdout = TRUE)),
      error = function(e) character(0)
    )
    out <- trimws(out[nzchar(trimws(out))])
    if (length(out) != 1L) {
      return(NA_real_)
    }
    suppressWarnings(as.numeric(out))
  }

  n <- 2000L

  gc()
  rss_before <- proc_rss_kb()

  for (i in seq_len(n)) {
    # Each call longjmps to top level; tryCatch contains it. This is the same
    # mechanism PR #930 used, repeated to accumulate any per-cycle leak.
    tryCatch(
      miniextendr:::gc_stress_with_r_thread_stop(),
      error = function(e) NULL
    )
  }

  gc()
  rss_after <- proc_rss_kb()

  # Record the RSS delta for documentation only — NOT asserted (see comment).
  if (!is.na(rss_before) && !is.na(rss_after)) {
    delta_kb <- rss_after - rss_before
    message(sprintf(
      "[#931 leak char] %d cycles: RSS %.0f -> %.0f KB (delta %.0f KB, ~%.1f bytes/cycle incl. R-side garbage + allocator arenas)",
      n, rss_before, rss_after, delta_kb, (delta_kb * 1024) / n
    ))
  } else {
    message("[#931 leak char] RSS not measurable on this platform")
  }

  # (a) + (b): reaching here proves no crash; a clean round-trip proves the
  # worker is still usable after N unwinds. These are the assertable facts.
  expect_identical(miniextendr:::gc_stress_worker_roundtrip(), 2234L)
})
