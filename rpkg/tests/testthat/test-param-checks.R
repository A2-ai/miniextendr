# Per-parameter `inherits` / `no_na` checks (src/rust/param_check_tests.rs).

mx_obj <- function() structure(list(a = 1, b = 2), class = "mx_obj")

test_that("inherits = \"cls\" accepts the class and refuses anything else", {
  expect_identical(miniextendr:::param_inherits_one(mx_obj()), 2L)
  expect_error(
    miniextendr:::param_inherits_one(list(a = 1)),
    "'x' must inherit from 'mx_obj'",
    fixed = TRUE
  )
  # The type check runs first.
  expect_error(miniextendr:::param_inherits_one(1), "'x' must be a list", fixed = TRUE)
})

test_that("inherits(\"a\", \"b\") accepts any of the classes", {
  expect_true(miniextendr:::param_inherits_any(structure(1, class = "mx_a")))
  expect_true(miniextendr:::param_inherits_any(structure(1, class = c("sub", "mx_b"))))
  expect_error(
    miniextendr:::param_inherits_any(1),
    "'x' must inherit from 'mx_a' or 'mx_b'",
    fixed = TRUE
  )
})

test_that("inherits on an Option<List> lets NULL through", {
  expect_false(miniextendr:::param_inherits_optional())
  expect_false(miniextendr:::param_inherits_optional(NULL))
  expect_true(miniextendr:::param_inherits_optional(mx_obj()))
  expect_error(
    miniextendr:::param_inherits_optional(list()),
    "'x' must inherit from 'mx_obj'",
    fixed = TRUE
  )
})

test_that("no_na refuses NA and NaN on a double scalar", {
  expect_identical(miniextendr:::param_no_na_scalar(1.5), 1.5)
  expect_error(miniextendr:::param_no_na_scalar(NA_real_), "'x' must not be NA", fixed = TRUE)
  expect_error(miniextendr:::param_no_na_scalar(NaN), "'x' must not be NA", fixed = TRUE)
  # Without `no_na` the same type accepts NA.
  expect_identical(miniextendr:::test_f64_identity(NA_real_), NA_real_)
})

test_that("no_na refuses a vector containing NA", {
  expect_identical(miniextendr:::param_no_na_vec(c(1, 2)), 3)
  expect_identical(miniextendr:::param_no_na_vec(double()), 0)
  expect_error(
    miniextendr:::param_no_na_vec(c(1, NA)),
    "'x' must not contain NA",
    fixed = TRUE
  )
})

test_that("no_na on a Missing<f64> lets an omitted argument through", {
  expect_false(miniextendr:::param_no_na_missing())
  expect_true(miniextendr:::param_no_na_missing(1))
  expect_error(miniextendr:::param_no_na_missing(NA_real_), "'x' must not be NA", fixed = TRUE)
})

test_that("both checks on one parameter run NA first, then the class", {
  x <- structure(c(1, 2), class = "mx_num")
  expect_identical(miniextendr:::param_checks_both(x), 3)
  expect_error(
    miniextendr:::param_checks_both(structure(c(1, NA), class = "mx_num")),
    "'x' must not contain NA",
    fixed = TRUE
  )
  expect_error(miniextendr:::param_checks_both(c(1, 2)), "'x' must inherit from 'mx_num'", fixed = TRUE)
})

test_that("fast keeps no_na but drops the type checks", {
  expect_identical(miniextendr:::param_no_na_fast(2), 2)
  expect_error(miniextendr:::param_no_na_fast(NA_real_), "'x' must not be NA", fixed = TRUE)
  # "a" reaches Rust: the conversion error, not an R-side type check.
  e <- tryCatch(miniextendr:::param_no_na_fast("a"), error = identity)
  expect_s3_class(e, "rust_error")
  expect_false(grepl("must be double", conditionMessage(e), fixed = TRUE))
})

test_that("under call = caller the checks are attributed to the caller", {
  obj <- mx_obj()
  expect_identical(miniextendr:::param_checks_caller(obj, 1), 1)
  e <- tryCatch(miniextendr:::param_checks_caller(list(), 1), error = identity)
  expect_identical(conditionMessage(e), "'x' must inherit from 'mx_obj'")
  expect_equal(conditionCall(e), quote(miniextendr:::param_checks_caller(x = list(), y = 1)))
  e <- tryCatch(miniextendr:::param_checks_caller(obj, NA_real_), error = identity)
  expect_identical(conditionMessage(e), "'y' must not be NA")
  expect_equal(conditionCall(e), quote(miniextendr:::param_checks_caller(x = obj, y = NA_real_)))
})

test_that("impl methods take inherits(...) / no_na(...) at method level", {
  h <- ParamCheckHolder$new()
  expect_identical(h$add(mx_obj(), 2), 2)
  expect_identical(h$add(structure(list(), class = "mx_other"), 3), 5)
  expect_error(h$add(list(), 1), "'x' must inherit from 'mx_obj' or 'mx_other'", fixed = TRUE)
  expect_error(h$add(mx_obj(), NA_real_), "'y' must not be NA", fixed = TRUE)
})

test_that("trait methods take no_na(...) at method level", {
  obj <- ScalerR6$new(2)
  expect_equal(ScalerR6$Scaler$scale(obj, x_factor = 3), 6)
  expect_error(ScalerR6$Scaler$scale(obj, x_factor = NA_real_), "'x_factor' must not be NA", fixed = TRUE)
})

# region: no_na on the reading markers
#
# `AsNumeric*` / `AsCharacter*` read inputs as missing that `anyNA()` passes
# (the text "NA", blank strings, a factor `NA` level). The R guard refuses R's
# `NA` first; the C wrapper checks the converted value after the conversion
# and raises the same condition.

caught_msg <- function(expr) conditionMessage(tryCatch(expr, error = identity))

test_that("no_na on AsNumeric accepts numbers in every form", {
  for (x in list(2.5, "2.5", " 2.5 ", factor("2.5"))) {
    expect_identical(miniextendr:::param_no_na_number(x), 2.5)
  }
  expect_identical(miniextendr:::param_no_na_number(Inf), Inf)
})

test_that("no_na on AsNumeric refuses R's NA and NaN in the R guard", {
  for (x in list(NA_real_, NA_character_, factor(NA), NaN, NA)) {
    expect_identical(caught_msg(miniextendr:::param_no_na_number(x)), "'x' must not be NA")
  }
})

test_that("no_na on AsNumeric refuses what the marker reads as missing", {
  for (x in list("NA", " NA ", "", "   ", "　")) {
    expect_identical(caught_msg(miniextendr:::param_no_na_number(x)), "'x' must not be NA")
  }
  for (x in list(factor("NA"), factor(""), factor(NA, exclude = NULL))) {
    expect_identical(caught_msg(miniextendr:::param_no_na_number(x)), "'x' must not be NA")
  }
})

test_that("no_na on AsNumeric refuses NaN read from text, as numeric NaN is", {
  expect_identical(caught_msg(miniextendr:::param_no_na_number("NaN")), "'x' must not be NA")
  expect_identical(caught_msg(miniextendr:::param_no_na_number(factor("NaN"))), "'x' must not be NA")
})

test_that("a value that is not a number stays the conversion error", {
  expect_identical(
    caught_msg(miniextendr:::param_no_na_number("n/a")),
    "'x' must be a single number: non-numeric value(s): \"n/a\" (element 1)"
  )
  expect_match(
    caught_msg(miniextendr:::param_no_na_numbers(c("NA", "n/a"))),
    "non-numeric value(s): \"n/a\" (element 2)",
    fixed = TRUE
  )
})

test_that("no_na on AsNumericVec says 'must not contain NA'", {
  expect_identical(miniextendr:::param_no_na_numbers(c("1", " 2 ", "0x10")), 0L)
  expect_identical(miniextendr:::param_no_na_numbers(character(0)), 0L)
  for (x in list(c("1", "NA", "", " "), factor(c("1", "")), c("1", "NaN"), c(1, NA))) {
    expect_identical(caught_msg(miniextendr:::param_no_na_numbers(x)), "'x' must not contain NA")
  }
})

test_that("no_na on Option<AsNumeric> lets NULL through as not given", {
  expect_identical(miniextendr:::param_no_na_number_opt(), "NULL")
  expect_identical(miniextendr:::param_no_na_number_opt(NULL), "NULL")
  expect_identical(miniextendr:::param_no_na_number_opt("3"), "3")
  for (x in list("NA", "", NA)) {
    expect_identical(caught_msg(miniextendr:::param_no_na_number_opt(x)), "'x' must not be NA")
  }
})

test_that("no_na on Missing<AsNumeric> lets an omitted argument through", {
  expect_identical(miniextendr:::param_no_na_number_missing(), "absent")
  expect_identical(miniextendr:::param_no_na_number_missing(" 2 "), "2")
  expect_identical(caught_msg(miniextendr:::param_no_na_number_missing("")), "'x' must not be NA")
})

test_that("fast drops the type checks but keeps both no_na checks", {
  expect_identical(miniextendr:::param_no_na_numbers_fast(c("1", "2")), 0L)
  expect_identical(
    caught_msg(miniextendr:::param_no_na_numbers_fast(c(1, NA))),
    "'x' must not contain NA"
  )
  expect_identical(
    caught_msg(miniextendr:::param_no_na_numbers_fast(c("1", "NA"))),
    "'x' must not contain NA"
  )
})

test_that("no_na on AsCharacter refuses an NA the conversion produces", {
  expect_identical(miniextendr:::param_no_na_label("NA"), "NA")
  expect_identical(miniextendr:::param_no_na_label(factor("a")), "a")
  expect_identical(
    caught_msg(miniextendr:::param_no_na_label(factor(NA, exclude = NULL))),
    "'x' must not be NA"
  )
})

test_that("a custom no_na message is the same text on both sides", {
  from_r <- caught_msg(miniextendr:::param_no_na_number_custom(NA))
  from_rust <- caught_msg(miniextendr:::param_no_na_number_custom("NA"))
  expect_identical(from_rust, from_r)
  expect_identical(from_rust, caught_msg(miniextendr:::param_no_na_custom(NA_real_)))
})

test_that("on the worker path the check runs before dispatch", {
  skip_if_not(miniextendr_has_feature("worker-thread"), "worker-thread feature off")
  expect_identical(miniextendr:::param_no_na_numbers_worker(c("1", "2")), 0L)
  expect_identical(
    caught_msg(miniextendr:::param_no_na_numbers_worker(c("1", ""))),
    "'x' must not contain NA"
  )
})

test_that("under call = caller the Rust check names the caller", {
  expect_identical(miniextendr:::param_no_na_number_caller(" 4 "), 4)
  e <- tryCatch(miniextendr:::param_no_na_number_caller("NA"), error = identity)
  expect_identical(conditionMessage(e), "'x' must not be NA")
  expect_equal(conditionCall(e), quote(miniextendr:::param_no_na_number_caller(x = "NA")))
  e <- tryCatch(miniextendr:::param_no_na_number_caller(NA), error = identity)
  expect_equal(conditionCall(e), quote(miniextendr:::param_no_na_number_caller(x = NA)))
})

test_that("no_na follows the type through an alias and a derived newtype", {
  expect_identical(miniextendr:::param_no_na_alias("1.5"), 1.5)
  expect_identical(caught_msg(miniextendr:::param_no_na_alias("NA")), "'x' must not be NA")
  expect_identical(miniextendr:::param_no_na_newtype(factor("7")), 7)
  expect_identical(caught_msg(miniextendr:::param_no_na_newtype("")), "'x' must not be NA")
  expect_identical(miniextendr:::param_no_na_newtype_opt(), "NULL")
  expect_identical(miniextendr:::param_no_na_newtype_opt("2"), "2")
  expect_identical(
    caught_msg(miniextendr:::param_no_na_newtype_opt("NaN")),
    "'x' must not be NA"
  )
})

test_that("no_na on a map of markers reads each top-level value", {
  expect_identical(miniextendr:::param_no_na_map(list(a = "1", b = 2)), 0L)
  expect_identical(
    caught_msg(miniextendr:::param_no_na_map(list(a = "NA", b = "2"))),
    "'x' must not contain NA"
  )
})

test_that("impl methods check the converted value of a no_na marker", {
  h <- ParamCheckHolder$new()
  expect_identical(h$add_dose(" 2 "), 2)
  expect_identical(caught_msg(h$add_dose(" NA ")), "'dose' must not be NA")
  expect_identical(h$add_doses(c("1", "2")), 5)
  expect_identical(caught_msg(h$add_doses(c("1", ""))), "every dose must be a number")
  expect_identical(caught_msg(h$add_doses(c(1, NA))), "every dose must be a number")
})

# endregion
