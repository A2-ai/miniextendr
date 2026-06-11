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

test_that("worker re-arms after an R longjmp tears through a job (#931)", {
  skip_on_os("windows")
  expect_error(miniextendr:::gc_stress_with_r_thread_stop())
  expect_identical(miniextendr:::gc_stress_worker_roundtrip_after_longjmp(), 42L)
  for (i in 1:5) try(miniextendr:::gc_stress_with_r_thread_stop(), silent = TRUE)
  expect_identical(miniextendr:::gc_stress_worker_roundtrip_after_longjmp(), 42L)
})

test_that("R-longjmp-through-worker leak does not compound across unwinds (#931)", {
  skip_on_cran()
  skip_on_os("windows")
  # The leak on the `R_ContinueUnwind` path is *fixed-per-unwind* and does NOT
  # compound (see `worker.rs::dispatch_to_worker` leak note): each unwind skips
  # the destructors for a constant set of channel/box state, so steady-state
  # RSS growth per batch is roughly constant rather than accelerating.
  #
  # We assert that *shape* (non-compounding) rather than an absolute KB/unwind
  # threshold. An absolute byte bound is not portable: RSS counts whole
  # resident pages and allocator arenas, not leaked bytes, so the per-unwind
  # RSS figure swings widely across glibc/allocator/R versions (R 4.6 measured
  # ~1.8 KB/unwind; R-devel's allocator can show ~19 KB/unwind for the *same*
  # bounded leak — see #931 CI). A compounding leak, by contrast, shows each
  # successive equal-sized batch growing RSS by a strictly increasing amount.
  rss_kb <- function() as.numeric(system2("ps", c("-o", "rss=", "-p", Sys.getpid()), stdout = TRUE))
  batch <- function(n) {
    for (i in seq_len(n)) try(miniextendr:::gc_stress_with_r_thread_stop(), silent = TRUE)
    gc()
    rss_kb()
  }
  n <- 2000
  # Warmup: the first few thousand unwinds reserve allocator arenas / resident
  # pages (a large *one-time* RSS bump unrelated to the per-unwind leak).
  batch(n)
  r0 <- rss_kb()
  r1 <- batch(n)  # growth over batch 1
  r2 <- batch(n)  # growth over batch 2
  g1 <- r1 - r0
  g2 <- r2 - r1
  # Non-compounding: batch 2's RSS growth must not balloon past batch 1's. A
  # genuine compounding leak grows super-linearly, so g2 would dwarf g1; a
  # fixed-per-unwind leak (plus page granularity) keeps g2 comparable to g1.
  # Allow generous slack (3x + 4 MB floor) for page-granularity jitter and the
  # occasional arena bump, while still failing a leak that doubles every batch.
  expect_lt(g2, max(3 * g1, 0) + 4096)
})
