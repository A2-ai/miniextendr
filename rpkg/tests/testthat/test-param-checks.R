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
