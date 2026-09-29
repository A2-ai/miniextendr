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

# region: trait impls and the R6 active setter under no_preconditions

test_that("trait-impl methods follow the impl's no_preconditions", {
  ns <- getNamespace("miniextendr")
  checked <- ns$FastCounter$new(4L)
  unchecked <- ns$FastCounterNoPreconditions$new(4L)
  expect_identical(ns$FastCounter$FastScale$scaled(checked, 2.5), 10)
  expect_identical(ns$FastCounterNoPreconditions$FastScale$scaled(unchecked, 2.5), 10)

  expect_error(ns$FastCounter$FastScale$scaled(checked, "x"), "'k' must be double",
               fixed = TRUE)
  # The R-side check is gone; the conversion refuses the value instead.
  scaled <- ns$FastCounterNoPreconditions$FastScale$scaled
  e <- tryCatch(scaled(unchecked, "x"), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_identical(conditionMessage(e), "'k' must be a single double: got character")
  expect_identical(conditionCall(e), quote(scaled(unchecked, "x")))
})

test_that("the R6 active-binding setter follows the impl's no_preconditions", {
  ns <- getNamespace("miniextendr")
  c <- ns$FastCounterNoPreconditions$new(1L)
  c$count <- 7L
  expect_identical(c$count, 7L)

  # No R-side check in the binding either: the conversion refuses the value,
  # as it does for the standalone setter under the same flag.
  e <- tryCatch(c$count <- "x", error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_identical(conditionMessage(e), "'value' must be a single integer: got character")
  expect_identical(c$count, 7L)
  e_method <- tryCatch(c$set_count("x"), error = function(e) e)
  expect_identical(conditionMessage(e_method), conditionMessage(e))
})

test_that("an empty-body (TPIE) trait impl follows the impl's no_preconditions", {
  ns <- getNamespace("miniextendr")
  from_str <- ns$FastCounterNoPreconditions$RFromStr$from_str
  # The `Option<Self>` return comes back as the R6 class.
  c <- from_str("12")
  expect_true(R6::is.R6(c))
  expect_s3_class(c, "FastCounterNoPreconditions")
  expect_identical(c$value(), 12L)

  # The same TPIE trait keeps the R-side check without the flag.
  expect_error(Point$RFromStr$from_str(12L), "'s' must be character", fixed = TRUE)
  e <- tryCatch(from_str(12L), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_identical(conditionMessage(e), "'s' must be a single string: got integer")
  expect_identical(conditionCall(e), quote(from_str(12L)))
})

# endregion
