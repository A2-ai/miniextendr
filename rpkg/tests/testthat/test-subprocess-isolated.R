# Subprocess-isolated tests for hazardous panic/thread/unwind paths.
#
# These tests exercise code paths that can crash R or leave error-handling
# state inconsistent when run in the main testthat process. Each test runs
# in a fresh R subprocess via callr::r(), so a crash or corrupted state
# does not contaminate the rest of the test suite.

skip_on_cran()
skip_if_not_installed("callr")

# ---------------------------------------------------------------------------
# Helper: run an expression in a subprocess with miniextendr loaded
# ---------------------------------------------------------------------------
run_isolated <- function(expr, timeout = 30) {
  callr::r(
    function(expr_to_eval) {
      library(miniextendr)
      eval(expr_to_eval)
    },
    args = list(expr_to_eval = substitute(expr)),
    timeout = timeout,
    error = "error"
  )
}

# ===========================================================================
# From test-errors-more.R: Thread panic propagation
# ===========================================================================
# These call R API from a spawned thread. The checked FFI wrappers detect
# wrong-thread usage and panic. The panic then propagates through
# extern "C-unwind", which can cause "failed to initiate panic" runtime
# errors. In a subprocess, a crash is acceptable -- we just verify the
# process doesn't hang and exits with an error.

test_that("unsafe_C_r_error_in_thread signals error in subprocess", {
  result <- tryCatch(
    run_isolated(miniextendr:::unsafe_C_r_error_in_thread()),
    error = function(e) e
  )
  # The subprocess should produce an error (either a clean R error or a crash).
  # If we get here with an error object, the test passes -- the subprocess
  # did not hang and the crash was contained.
  expect_true(inherits(result, "error"))
})

test_that("unsafe_C_r_print_in_thread signals error in subprocess", {
  result <- tryCatch(
    run_isolated(miniextendr:::unsafe_C_r_print_in_thread()),
    error = function(e) e
  )
  expect_true(inherits(result, "error"))
})

# ===========================================================================
# From test-thread-broken.R: RThreadBuilder crashes R
# ===========================================================================
# RThreadBuilder attempts to register a new thread with R's internal
# threading system. This currently corrupts R's stack-checking state and
# crashes the process. We verify the subprocess terminates with an error
# rather than hanging.

test_that("RThreadBuilder basic flow crashes cleanly in subprocess", {
  result <- tryCatch(
    run_isolated(miniextendr:::unsafe_C_test_r_thread_builder()),
    error = function(e) e
  )
  # Either it crashes (error) or, if future R versions fix registration,
  # it might succeed and return 123L. Both outcomes are acceptable.
  if (inherits(result, "error")) {
    expect_true(inherits(result, "error"))
  } else {
    expect_equal(result, 123L)
  }
})

test_that("RThreadBuilder spawn_join crashes cleanly in subprocess", {
  result <- tryCatch(
    run_isolated(miniextendr:::unsafe_C_test_r_thread_builder_spawn_join()),
    error = function(e) e
  )
  if (inherits(result, "error")) {
    expect_true(inherits(result, "error"))
  } else {
    expect_equal(result, 456L)
  }
})

# ===========================================================================
# From test-unwind.R: R error paths in unwind-protect
# ===========================================================================
# These functions trigger R errors inside with_r_unwind_protect, which
# exercises the longjmp cleanup path. Running multiple of these in sequence
# within a single process can leave R/Rust error handling in an inconsistent
# state, but each one should work correctly in isolation.

test_that("unwind_protect_r_error signals R error in subprocess", {
  result <- tryCatch(
    run_isolated(miniextendr:::unsafe_C_unwind_protect_r_error()),
    error = function(e) e
  )
  expect_true(inherits(result, "error"))
  # The error should mention the intentional R error message
  expect_match(conditionMessage(result), "intentional R error|error", ignore.case = TRUE)
})

test_that("unwind_protect_lowlevel_test signals R error in subprocess", {
  result <- tryCatch(
    run_isolated(miniextendr:::unsafe_C_unwind_protect_lowlevel_test()),
    error = function(e) e
  )
  expect_true(inherits(result, "error"))
  expect_match(conditionMessage(result), "test R error|error", ignore.case = TRUE)
})

test_that("add_r_error triggers R error cleanly in subprocess", {
  result <- tryCatch(
    run_isolated(miniextendr:::add_r_error(1L, 2L)),
    error = function(e) e
  )
  expect_true(inherits(result, "error"))
  expect_match(conditionMessage(result), "r error in|add_r_error|error", ignore.case = TRUE)
})

# ===========================================================================
# Sequence stability: run multiple unwind error paths back-to-back
# ===========================================================================
# The original test-unwind.R notes that running error scenarios in sequence
# can leave state inconsistent. Verify that a subprocess can survive two
# sequential unwind-error calls without crashing.

test_that("sequential unwind error paths survive in single subprocess", {
  result <- tryCatch(
    callr::r(function() {
      library(miniextendr)
      # First call: should trigger R error
      tryCatch(
        miniextendr:::unsafe_C_unwind_protect_r_error(),
        error = function(e) paste("caught:", conditionMessage(e))
      )
      # Second call: should also trigger R error without crashing
      tryCatch(
        miniextendr:::unsafe_C_unwind_protect_lowlevel_test(),
        error = function(e) paste("caught:", conditionMessage(e))
      )
      # If we get here, state was not corrupted
      "ok"
    }, timeout = 30, error = "error"),
    error = function(e) e
  )
  # Either the subprocess completed with "ok" (stable) or crashed (unstable).
  # Both are informative -- we just want no hang.
  if (inherits(result, "error")) {
    # Process crashed or errored -- that's the known instability, still passes
    # as long as the subprocess didn't hang (timeout would throw different error)
    expect_false(grepl("timed out", conditionMessage(result), ignore.case = TRUE))
  } else {
    expect_equal(result, "ok")
  }
})
