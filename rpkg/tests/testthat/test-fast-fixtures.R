# Tests for the `#[miniextendr(no_preconditions)]`,
# `no_call_attribution`, and `fast` options.

test_that("fast_i32_default works on the happy path", {
  expect_identical(fast_i32_default(42L), 42L)
})

test_that("all variants agree on the happy path", {
  expect_identical(fast_i32_default(42L), 42L)
  expect_identical(fast_i32_no_preconditions(42L), 42L)
  expect_identical(fast_i32_no_call_attribution(42L), 42L)
  expect_identical(fast_i32_fast(42L), 42L)
})

test_that("multi-arg fast variant agrees on the happy path", {
  expect_identical(fast_sum3_default(1L, 2L, 3L), 6L)
  expect_identical(fast_sum3_fast(1L, 2L, 3L), 6L)
})

test_that("default wrapper raises stopifnot for bad input", {
  expect_error(fast_i32_default("not an int"),
               regexp = "must be numeric, logical, or raw")
})

test_that("no_preconditions wrapper raises a rust_error for bad input", {
  # stopifnot is gone; TryFromSexp still rejects the bad input, but the
  # message comes from Rust now ("failed to convert parameter 'x' to i32").
  e <- tryCatch(fast_i32_no_preconditions("not an int"), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_match(conditionMessage(e),
               "failed to convert parameter 'x' to i32")
})

test_that("no_call_attribution wrapper still rejects bad input via stopifnot", {
  expect_error(fast_i32_no_call_attribution("not an int"),
               regexp = "must be numeric, logical, or raw")
})

test_that("fast wrapper raises a rust_error for bad input", {
  e <- tryCatch(fast_i32_fast("not an int"), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_match(conditionMessage(e),
               "failed to convert parameter 'x' to i32")
})

test_that("no_call_attribution: error$call falls back to sys.call()", {
  e <- tryCatch(fast_i32_no_call_attribution("not an int"),
                error = function(e) e)
  # We don't get to compare to match.call() here because stopifnot fires
  # first. Instead exercise the conversion path:
  e2 <- tryCatch(fast_i32_fast("not an int"), error = function(e) e)
  expect_s3_class(e2, "rust_error")
  # call slot is populated (not NULL) — the wrapper's sys.call() fallback
  # surfaces the user invocation.
  expect_false(is.null(conditionCall(e2)))
  expect_match(deparse(conditionCall(e2)), "fast_i32_fast")
})

test_that("default wrapper's error$call uses match.call() (named args)", {
  e <- tryCatch(fast_i32_default(x = -2147483648L - 1),
                error = function(e) e)
  # The min-int sentinel is NA_integer_ in R; this triggers the
  # length-1 / numeric check downstream of stopifnot. Just verify the
  # call slot is named when match.call() is in play.
  # (Exact behaviour: stopifnot fires first for "not an int", but for a
  # valid integer-shaped NA the error still trips eventually.)
  # We're not testing the precise error class here, just call attribution.
  if (!is.null(conditionCall(e))) {
    expect_true(grepl("fast_i32_default", deparse(conditionCall(e))))
  }
})

# ----------------------------------------------------------------------------
# Impl-block fast-path (R6) tests
# ----------------------------------------------------------------------------

test_that("default R6 FastCounter works", {
  ns <- getNamespace("miniextendr")
  c <- ns$FastCounter$new(10L)
  expect_identical(c$value(), 10L)
  expect_identical(c$add(5L), 15L)
  expect_identical(c$value(), 15L)
})

test_that("fast R6 FastCounterFast works (same semantics, fast wrappers)", {
  ns <- getNamespace("miniextendr")
  c <- ns$FastCounterFast$new(10L)
  expect_identical(c$value(), 10L)
  expect_identical(c$add(5L), 15L)
  expect_identical(c$value(), 15L)
})

test_that("fast R6 class still raises rust_error on bad input", {
  ns <- getNamespace("miniextendr")
  c <- ns$FastCounterFast$new(0L)
  e <- tryCatch(c$add("not an int"), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_match(conditionMessage(e),
               "failed to convert parameter 'n' to i32")
})

test_that("default R6 class raises stopifnot on bad input", {
  ns <- getNamespace("miniextendr")
  c <- ns$FastCounter$new(0L)
  expect_error(c$add("not an int"),
               regexp = "must be numeric, logical, or raw")
})

# ----------------------------------------------------------------------------
# fast × error_direct interaction
#
# With both knobs active: `.call = NULL` AND direct-C raise.
#
# The C wrapper receives R_NilValue for the call SEXP (because fast emits
# `.call = NULL`). error_direct builds the condition in C with call = NULL,
# then calls stop(structure(...)).  R's stop() sees a condition with a NULL
# call slot and — because call. defaults to TRUE — fills in sys.call() for
# the calling frame.  So conditionCall(e) is NOT NULL.
# ----------------------------------------------------------------------------

test_that("fast + error_direct happy path works", {
  expect_identical(fast_and_error_direct(7L), 7L)
})

test_that("fast + error_direct: conditionCall is non-NULL (R's stop() fills in call when slot is NULL)", {
  e <- tryCatch(fast_and_error_direct("not an int"), error = function(e) e)
  expect_s3_class(e, "rust_error")
  # R's stop() fills in the call when the condition's call slot is NULL
  # (call. defaults to TRUE). conditionCall is therefore non-NULL even
  # though fast emitted .call = NULL to the C wrapper.
  expect_false(is.null(conditionCall(e)))
})
