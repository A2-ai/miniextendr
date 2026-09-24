# Crate-level `conversion_error_class` default
#
# producer.pkg's Cargo.toml sets `[package.metadata.miniextendr]
# conversion_error_class = ["producer_error_argument", "producer_error"]`.
# Every argument-conversion condition of the crate carries those classes: after
# the error type's own `RConditionError` classes (duplicates skipped), and on
# their own for a plain `SexpError`. `kind` stays "conversion". The R-side
# argument checks (type / length, `no_na`, `inherits`, choices) raise the same
# condition (#1591), so one handler catches every argument error.

layers <- c("rust_error", "simpleError", "error", "condition")
crate_classes <- c("producer_error_argument", "producer_error", layers)

test_that("a plain conversion error gets the crate default classes", {
  e <- tryCatch(producer_int_twice("a"), error = function(e) e)
  expect_equal(class(e), crate_classes)
  expect_equal(e$kind, "conversion")
  expect_equal(e$param, "x")
  expect_equal(e$rust_type, "i32")
  expect_equal(conditionMessage(e), "'x' must be a single integer: got character")
  expect_equal(tryCatch(producer_int_twice(NA_integer_), producer_error = function(e) e$param), "x")
  expect_identical(producer_int_twice(2L), 4L)
})

test_that("the error type's own classes come first, without duplicates", {
  e <- tryCatch(producer_count_twice(-1L), error = function(e) e)
  expect_equal(
    class(e),
    c("producer_error_not_positive", "producer_error", "producer_error_argument", layers)
  )
  expect_equal(e$kind, "conversion")
  expect_equal(e$param, "n")
  expect_equal(e$rust_type, "ProducerCount")
  expect_equal(e$value, -1L)
  # No R-facing expectation for a custom type: R's `invalid '<p>' argument`.
  expect_equal(conditionMessage(e), "invalid 'n' argument: expected a positive count, got -1")

  e <- tryCatch(producer_count_twice("a"), error = function(e) e)
  expect_equal(
    class(e),
    c("producer_error_not_integer", "producer_error", "producer_error_argument", layers)
  )
  expect_identical(producer_count_twice(3L), 6L)
})

test_that("the R-side length check and the Rust conversion raise one condition (#1591)", {
  # A scalar AsNumeric given two values: caught by the R-side length check.
  e1 <- tryCatch(producer.pkg:::producer_ratio_caller(c(1, 2), 3), error = function(e) e)
  # An AsNumericVec given a non-number: caught by the Rust conversion.
  e2 <- tryCatch(producer.pkg:::producer_peak_caller(c("1", "BLQ")), error = function(e) e)
  expect_identical(class(e1), crate_classes)
  expect_identical(class(e2), class(e1))
  expect_identical(e1$kind, "conversion")
  expect_identical(e2$kind, "conversion")
  expect_identical(e1$param, "num")
  expect_identical(e2$param, "dv")
  expect_identical(conditionMessage(e1), "'num' must have length 1")
  expect_identical(
    conditionMessage(e2),
    "'dv' must be numeric: non-numeric value(s): \"BLQ\" (element 2)"
  )
  # The Rust type is for the package author, kept out of the message; the
  # R-side check has none.
  expect_null(e1$rust_type)
  expect_identical(e2$rust_type, "AsNumericVec")
  # The crate default `call = caller` names the delegate on both paths.
  expect_equal(conditionCall(e1), quote(producer.pkg:::producer_ratio_caller(num = c(1, 2), den = 3)))
  expect_equal(conditionCall(e2), quote(producer.pkg:::producer_peak_caller(dv = c("1", "BLQ"))))
  # One handler for the package's argument errors catches both.
  catch_param <- function(expr) tryCatch(expr, producer_error_argument = function(e) e$param)
  expect_identical(catch_param(producer.pkg:::producer_ratio_caller(c(1, 2), 3)), "num")
  expect_identical(catch_param(producer.pkg:::producer_peak_caller(c("1", "BLQ"))), "dv")
  expect_identical(producer.pkg:::producer_ratio_caller("3", 2), 1.5)
  expect_identical(producer.pkg:::producer_peak_caller(c("1", "4")), 4)
})

test_that("no_na, inherits and choices failures carry the crate classes", {
  x <- structure(c(1, 2), class = "producer_num")
  expect_identical(producer.pkg:::producer_named_checks_impl(x, "slow"), "slow: 3")

  e <- tryCatch(
    producer.pkg:::producer_named_checks_impl(structure(c(1, NA), class = "producer_num"), "fast"),
    error = function(e) e
  )
  expect_identical(class(e), crate_classes)
  expect_identical(e$param, "x")
  expect_identical(conditionMessage(e), "'x' must not contain NA")

  e <- tryCatch(producer.pkg:::producer_named_checks_impl(c(1, 2), "fast"), error = function(e) e)
  expect_identical(class(e), crate_classes)
  expect_identical(e$param, "x")
  expect_identical(conditionMessage(e), "'x' must inherit from 'producer_num'")

  e <- tryCatch(producer.pkg:::producer_named_checks_impl(x, "medium"), error = function(e) e)
  expect_identical(class(e), crate_classes)
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "mode")
  expect_identical(conditionMessage(e), "'mode' should be one of \"fast\", \"slow\"")
  # `call = wrapper`: the wrapper's own call, as the R-side checks always reported it.
  expect_equal(conditionCall(e), quote(producer.pkg:::producer_named_checks_impl(x, "medium")))
})

test_that("a check with the author's message keeps the crate classes", {
  x <- structure(c(1, 2), class = "producer_num")
  expect_identical(producer.pkg:::producer_named_checks_msg_impl(x), 3)

  e <- tryCatch(
    producer.pkg:::producer_named_checks_msg_impl(structure(c(1, NA), class = "producer_num")),
    error = function(e) e
  )
  expect_identical(class(e), crate_classes)
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "x")
  expect_identical(conditionMessage(e), "`x` must not contain NA")

  e <- tryCatch(producer.pkg:::producer_named_checks_msg_impl(c(1, 2)), error = function(e) e)
  expect_identical(class(e), crate_classes)
  expect_identical(e$param, "x")
  expect_identical(conditionMessage(e), "`x` must be a `producer_num`")
  expect_equal(conditionCall(e), quote(producer.pkg:::producer_named_checks_msg_impl(c(1, 2))))
})
