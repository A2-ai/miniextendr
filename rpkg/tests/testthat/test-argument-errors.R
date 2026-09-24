# One argument-error condition whichever side catches a bad argument (#1591):
# src/rust/argument_error_tests.rs. An R-side check (type / length, `no_na`,
# `inherits`, choices) and a failed Rust conversion raise the same classes,
# `kind = "conversion"` and `e$param`; the conversion adds `e$rust_type`.
# rpkg sets no `conversion_error_class`, so the classes are the plain
# `rust_error` layering here; the configured case is in
# tests/cross-package/producer.pkg/tests/testthat/test-conversion-error-class.R.

layers <- c("rust_error", "simpleError", "error", "condition")
caught <- function(expr) tryCatch(expr, error = identity)

# region: the issue's two paths

test_that("the R-side check and the Rust conversion raise the same condition", {
  # A scalar AsNumeric given two values: caught by the R-side length check.
  e1 <- caught(miniextendr:::arg_error_ratio(c(1, 2), 3))
  # An AsNumericVec given a non-number: caught by the Rust conversion.
  e2 <- caught(miniextendr:::arg_error_peak(c("1", "BLQ")))

  expect_identical(class(e1), layers)
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
  # The calls are as before: the R-side check names the wrapper's call as
  # written (what stopifnot() reported), the conversion its matched call.
  expect_equal(conditionCall(e1), quote(miniextendr:::arg_error_ratio(c(1, 2), 3)))
  expect_equal(conditionCall(e2), quote(miniextendr:::arg_error_peak(dv = c("1", "BLQ"))))
})

test_that("one handler catches both paths", {
  param_of <- function(expr) tryCatch(expr, rust_error = function(e) e$param)
  expect_identical(param_of(miniextendr:::arg_error_ratio(c(1, 2), 3)), "num")
  expect_identical(param_of(miniextendr:::arg_error_peak(c("1", "BLQ"))), "dv")
  # The base classes stay, so a simpleError handler still sees both.
  expect_s3_class(caught(miniextendr:::arg_error_ratio(1, list())), "simpleError")
  expect_identical(caught(miniextendr:::arg_error_ratio(1, list()))$param, "den")
})

test_that("a scalar AsNumeric that fails its conversion reads in R terms", {
  e <- caught(miniextendr:::arg_error_ratio("BLQ", 3))
  expect_identical(class(e), layers)
  expect_identical(e$param, "num")
  expect_identical(e$rust_type, "AsNumeric")
  expect_identical(
    conditionMessage(e),
    "'num' must be a single number: non-numeric value(s): \"BLQ\" (element 1)"
  )
  expect_identical(miniextendr:::arg_error_ratio("3", 2L), 1.5)
  expect_identical(miniextendr:::arg_error_peak(c("1", "4")), 4)
})

test_that("under call = caller both paths name the caller's call", {
  e1 <- caught(miniextendr:::arg_error_ratio_caller(c(1, 2), 3))
  e2 <- caught(miniextendr:::arg_error_peak_caller(c("1", "BLQ")))
  expect_identical(class(e1), layers)
  expect_identical(class(e2), layers)
  expect_identical(e1$param, "num")
  expect_identical(e2$param, "dv")
  expect_equal(conditionCall(e1), quote(miniextendr:::arg_error_ratio_caller(num = c(1, 2), den = 3)))
  expect_equal(conditionCall(e2), quote(miniextendr:::arg_error_peak_caller(dv = c("1", "BLQ"))))
})

# endregion

# region: R wording of the conversion errors

test_that("built-in conversion errors say what the argument must be", {
  e <- caught(miniextendr:::arg_error_int_unchecked("a"))
  expect_identical(conditionMessage(e), "'x' must be a single integer: got character")
  expect_identical(e$rust_type, "i32")
  expect_identical(e$param, "x")
  expect_identical(class(e), layers)
  expect_identical(
    conditionMessage(caught(miniextendr:::arg_error_int_unchecked(1:2))),
    "'x' must be a single integer: got length 2"
  )
  expect_identical(
    conditionMessage(caught(miniextendr:::arg_error_int_unchecked(NA_integer_))),
    "'x' must be a single integer: NA is not allowed"
  )
  expect_identical(
    conditionMessage(caught(miniextendr:::arg_error_flag_unchecked(NA))),
    "'flag' must be TRUE or FALSE: NA is not allowed"
  )
  expect_identical(
    conditionMessage(caught(miniextendr:::arg_error_string_unchecked(1))),
    "'s' must be a single string: got numeric"
  )
  expect_identical(
    conditionMessage(caught(miniextendr:::arg_error_strings_unchecked(1:2))),
    "'xs' must be character: got integer"
  )
  # No SEXPTYPE names and no variant prefix reach the message.
  for (msg in c(
    conditionMessage(caught(miniextendr:::arg_error_int_unchecked("a"))),
    conditionMessage(caught(miniextendr:::arg_error_peak(c("1", "BLQ"))))
  )) {
    expect_no_match(msg, "SXP|invalid value|failed to convert")
  }
})

test_that("a selection of the wrong length for a fixed-size array is an argument error", {
  e <- caught(match_arg_multi_mode_array("Fast"))
  expect_identical(class(e), layers)
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "modes")
  expect_identical(conditionMessage(e), "'modes' must be of length 2: got length 1")
})

# endregion

# region: the checks the author names

test_that("no_na, inherits and choices failures are argument errors", {
  x <- structure(c(1, 2), class = "mx_num")
  expect_identical(miniextendr:::arg_error_named_checks(x, "slow"), "slow: 3")

  e <- caught(miniextendr:::arg_error_named_checks(structure(c(1, NA), class = "mx_num"), "fast"))
  expect_identical(class(e), layers)
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "x")
  expect_identical(conditionMessage(e), "'x' must not contain NA")

  e <- caught(miniextendr:::arg_error_named_checks(c(1, 2), "fast"))
  expect_identical(class(e), layers)
  expect_identical(e$param, "x")
  expect_identical(conditionMessage(e), "'x' must inherit from 'mx_num'")

  e <- caught(miniextendr:::arg_error_named_checks(x, "medium"))
  expect_identical(class(e), layers)
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "mode")
  expect_identical(conditionMessage(e), "'mode' should be one of \"fast\", \"slow\"")
  expect_equal(conditionCall(e), quote(miniextendr:::arg_error_named_checks(x, "medium")))
})

test_that("match_arg and several_ok failures are argument errors", {
  e <- caught(match_arg_multi_mode(c("Fast", "zzz")))
  expect_identical(class(e), layers)
  expect_identical(e$param, "modes")
  expect_match(conditionMessage(e), "^'modes' element 2 \\(\"zzz\"\\) should be one of")
  e <- caught(match_arg_multi_mode(1L))
  expect_identical(e$param, "modes")
  expect_identical(conditionMessage(e), "'modes' must be NULL or a character vector")
})

test_that("method and trait-method checks raise the same condition", {
  h <- ParamCheckHolder$new()
  e <- caught(h$add(list(), 1))
  expect_identical(class(e), layers)
  expect_identical(e$param, "x")
  e <- caught(h$add(structure(list(), class = "mx_obj"), NA_real_))
  expect_identical(class(e), layers)
  expect_identical(e$param, "y")
  expect_identical(conditionMessage(e), "'y' must not be NA")

  obj <- ScalerR6$new(2)
  e <- caught(ScalerR6$Scaler$scale(obj, x_factor = NA_real_))
  expect_identical(class(e), layers)
  expect_identical(e$param, "x_factor")
})

test_that("fast keeps the named check and the conversion error is the same kind", {
  e <- caught(miniextendr:::param_no_na_fast(NA_real_))
  expect_identical(class(e), layers)
  expect_identical(e$param, "x")
  e <- caught(miniextendr:::param_no_na_fast("a"))
  expect_identical(class(e), layers)
  expect_identical(e$param, "x")
  expect_identical(e$rust_type, "f64")
  expect_identical(conditionMessage(e), "'x' must be a single double: got character")
})

# endregion
