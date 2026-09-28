# Tests for the `#[miniextendr(no_preconditions)]` option (fixtures in
# src/rust/fast_fixtures.rs).

test_that("fast_i32_default works on the happy path", {
  expect_identical(fast_i32_default(42L), 42L)
})

test_that("all variants agree on the happy path", {
  expect_identical(fast_i32_default(42L), 42L)
  expect_identical(fast_i32_no_preconditions(42L), 42L)
})

test_that("multi-arg no_preconditions variant agrees on the happy path", {
  expect_identical(fast_sum3_default(1L, 2L, 3L), 6L)
  expect_identical(fast_sum3_no_preconditions(1L, 2L, 3L), 6L)
})

test_that("default wrapper raises the R-side check for bad input", {
  expect_error(fast_i32_default("not an int"),
               regexp = "must be integer")
})

test_that("no_preconditions wrapper raises a rust_error for bad input", {
  # The R-side check is gone; TryFromSexp still rejects the bad input, with
  # the same argument-error condition worded by the conversion (#1591).
  e <- tryCatch(fast_i32_no_preconditions("not an int"), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_identical(conditionMessage(e), "'x' must be a single integer: got character")
  expect_identical(e$rust_type, "i32")
})

test_that("no_preconditions passes the call as written", {
  # No guard, but the same `.call = sys.call()` slot as every wrapper.
  body_text <- paste(deparse(body(fast_i32_no_preconditions)), collapse = "\n")
  expect_match(body_text, ".call = sys.call()", fixed = TRUE)
  expect_false(grepl("miniextendr_arg_error", body_text, fixed = TRUE))
  e <- tryCatch(fast_i32_no_preconditions("not an int"), error = function(e) e)
  expect_identical(conditionCall(e), quote(fast_i32_no_preconditions("not an int")))
})

test_that("default wrapper's Rust conversion error reports the call as written", {
  # `NA_integer_` passes the R-side type and length checks; the Rust `i32`
  # conversion refuses it.
  e <- tryCatch(fast_i32_default(NA_integer_), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_identical(conditionCall(e), quote(fast_i32_default(NA_integer_)))
})

# region: impl-block no_preconditions (R6)

test_that("default R6 FastCounter works", {
  ns <- getNamespace("miniextendr")
  c <- ns$FastCounter$new(10L)
  expect_identical(c$value(), 10L)
  expect_identical(c$add(5L), 15L)
  expect_identical(c$value(), 15L)
})

test_that("R6 FastCounterNoPreconditions works (same semantics, unchecked wrappers)", {
  ns <- getNamespace("miniextendr")
  c <- ns$FastCounterNoPreconditions$new(10L)
  expect_identical(c$value(), 10L)
  expect_identical(c$add(5L), 15L)
  expect_identical(c$value(), 15L)
})

test_that("no_preconditions R6 class still raises rust_error on bad input", {
  ns <- getNamespace("miniextendr")
  c <- ns$FastCounterNoPreconditions$new(0L)
  e <- tryCatch(c$add("not an int"), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_identical(conditionMessage(e), "'n' must be a single integer: got character")
  expect_identical(conditionCall(e), quote(c$add("not an int")))
})

test_that("default R6 class raises the R-side check on bad input", {
  ns <- getNamespace("miniextendr")
  c <- ns$FastCounter$new(0L)
  expect_error(c$add("not an int"),
               regexp = "must be integer")
})

# endregion
