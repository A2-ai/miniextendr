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
