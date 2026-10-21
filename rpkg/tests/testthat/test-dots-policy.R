# `#[miniextendr(dots = empty | empty(allow = ...) | forward(allow = ...) |
# data)]` (#2157): the wrapper refuses what `...` doesn't take before it
# evaluates anything, as one argument error naming every refused argument with
# the formal or allowed name it most likely meant. Fixtures:
# src/rust/dots_policy_tests.rs (an `mx_dots_fit` is a classed list built
# here).

mx_dots_fit <- function(estimate = 2) {
  structure(list(estimate = estimate), class = "mx_dots_fit")
}

caught <- function(expr) tryCatch(expr, error = identity)

test_that("dots = empty refuses a misspelt argument with a suggestion", {
  fit <- mx_dots_fit()
  expect_equal(format(fit), "estimate 2.000")
  expect_equal(format(fit, digits = 1L), "estimate 2.0")

  e <- caught(format(fit, digts = 1L))
  expect_s3_class(e, "miniextendr_error_unused_argument")
  expect_s3_class(e, "rust_error")
  expect_identical(
    conditionMessage(e),
    "unused argument 'digts' (did you mean 'digits'?)"
  )
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "...")
  expect_identical(e$argument, "digts")
  expect_identical(e$suggestion, "digits")
  # An S3 method reports the generic's call.
  expect_identical(conditionCall(e), quote(format(fit, digts = 1L)))
})

test_that("every refused argument is in one condition, unnamed ones by position", {
  e <- caught(mx_dots_predict(mx_dots_fit(), 1, 3, nwedata = 2, zzz = 4))
  expect_identical(e$argument, c("..1", "nwedata", "zzz"))
  expect_identical(e$suggestion, c(NA, "newdata", NA))
  expect_identical(
    conditionMessage(e),
    "unused arguments '..1' (unnamed), 'nwedata' (did you mean 'newdata'?), 'zzz'"
  )
})

test_that("the check forces nothing in `...`", {
  e <- caught(format(mx_dots_fit(), digts = stop("forced")))
  expect_s3_class(e, "miniextendr_error_unused_argument")
  expect_identical(e$argument, "digts")
})

test_that("a misspelt required argument is named, not reported missing", {
  fit <- mx_dots_fit(3)
  expect_equal(mx_dots_predict(fit, newdata = c(1, 2)), c(3, 6))
  e <- caught(mx_dots_predict(fit, nwedata = c(1, 2)))
  expect_s3_class(e, "miniextendr_error_unused_argument")
  expect_identical(e$suggestion, "newdata")
  expect_identical(conditionCall(e), quote(mx_dots_predict(fit, nwedata = c(1, 2))))
})

test_that("empty(allow = ...) takes the names base R passes", {
  fit <- mx_dots_fit(4)
  # data.frame() calls as.data.frame(x, optional = TRUE).
  expect_identical(data.frame(fit), data.frame(estimate = 4))
  expect_identical(
    as.data.frame(fit, row.names = NULL, optional = TRUE, stringsAsFactors = FALSE),
    data.frame(estimate = 4)
  )
  e <- caught(as.data.frame(fit, optinal = TRUE))
  expect_s3_class(e, "miniextendr_error_unused_argument")
  expect_identical(e$suggestion, "optional")
  expect_identical(conditionCall(e), quote(as.data.frame(fit, optinal = TRUE)))
})

test_that("empty(allow = ...) takes the generic's arguments given by position", {
  # The method's only formal is `x`, so the generic's `row.names` and
  # `optional` given by position reach `...` unnamed. Nothing reads them, so
  # only their number counts: one for each allowed name not given by name
  # (four: row.names, optional, stringsAsFactors, validRN).
  fit <- mx_dots_fit(4)
  expect_identical(as.data.frame(fit, NULL, TRUE), data.frame(estimate = 4))
  expect_identical(as.data.frame(fit, NULL, TRUE, FALSE, FALSE), data.frame(estimate = 4))
  e <- caught(as.data.frame(fit, NULL, TRUE, FALSE, FALSE, 1))
  expect_s3_class(e, "miniextendr_error_unused_argument")
  expect_identical(e$argument, "..5")
  expect_identical(e$suggestion, NA_character_)
  expect_identical(conditionMessage(e), "unused argument '..5' (unnamed)")
  # An allowed name given by name leaves one place fewer.
  e <- caught(as.data.frame(fit, NULL, TRUE, FALSE, FALSE, optional = TRUE))
  expect_identical(e$argument, "..4")
  # With no allowed names, every unnamed argument is refused.
  expect_identical(caught(mx_dots_predict(fit, 1, 2))$argument, "..1")
})

test_that("forward(allow = ...) passes the listed names and unnamed arguments on", {
  expect_identical(mx_dots_forward(), list())
  expect_identical(mx_dots_forward(digits = 3, nsmall = 1), list(digits = 3, nsmall = 1))
  # An unnamed argument goes on as it is; the function the dots are passed to
  # matches it by position.
  expect_identical(mx_dots_forward(3, nsmall = 1), list(3, nsmall = 1))
  expect_identical(mx_dots_forward(3, 1, 2), list(3, 1, 2))
  e <- caught(mx_dots_forward(3, digts = 3))
  expect_identical(class(e)[1:2], c("mx_dots_unused", "miniextendr_error_unused_argument"))
  expect_identical(e$argument, "digts")
  expect_identical(e$suggestion, "digits")
  # A unique prefix of a listed name is suggested.
  expect_identical(caught(mx_dots_forward(ns = 1))$suggestion, "nsmall")
})

test_that("a class in both the refusal's and the crate's classes appears once", {
  # rpkg sets no `conversion_error_class`; bind one where the helpers look it
  # up, as a crate with `conversion_error_class = ["pkg_error_argument",
  # "pkg_error"]` and a `field_error(class = ...)` sharing "pkg_error" has it.
  with_crate_classes <- function(f) {
    environment(f) <- list2env(
      list(.miniextendr_conversion_error_class = c("pkg_error_argument", "pkg_error")),
      parent = asNamespace("miniextendr")
    )
    f
  }
  check_dots <- with_crate_classes(miniextendr:::.miniextendr_check_dots)
  e <- caught(check_dots(
    "foo", 1L, c("x", "i", "exact"),
    class = c("pkg_error_config", "pkg_error"), call = quote(x[["a", foo = 1]])
  ))
  expect_identical(class(e), c(
    "pkg_error_config", "pkg_error", "miniextendr_error_unused_argument",
    "pkg_error_argument", "rust_error", "simpleError", "error", "condition"
  ))
  expect_identical(e$argument, "foo")

  # The same for a condition raised from Rust: `arg_error!` sends a marker
  # where the crate's classes go, after the error's own.
  raise <- with_crate_classes(miniextendr:::.miniextendr_raise_condition)
  value <- list(
    error = "'mode' must be one of \"a\"", kind = "conversion",
    class = c("pkg_error", ".miniextendr_conversion_error_class"), call = NULL
  )
  e <- caught(raise(value, quote(f(mode = "b"))))
  expect_identical(
    class(e),
    c("pkg_error", "pkg_error_argument", "rust_error", "simpleError", "error", "condition")
  )
})

test_that("dots = data checks nothing", {
  expect_identical(mx_dots_data(), 0L)
  expect_identical(mx_dots_data(1, b = 2, zzz = 3), 3L)
})

test_that("an impl-block S3 method checks the generic's dispatch `...`", {
  counter <- new_mxdotscounter(7L)
  expect_identical(format(counter, width = 3L), "  7")
  # The misspelt name is reported, not the missing `width`.
  e <- caught(format(counter, widht = 3L))
  expect_s3_class(e, "miniextendr_error_unused_argument")
  expect_identical(e$suggestion, "width")
  expect_identical(conditionCall(e), quote(format(counter, widht = 3L)))
})

test_that("an R6 method with a `...` parameter passes the listed names on", {
  tally <- MxDotsTally$new()
  expect_equal(tally$add(), 1)
  expect_equal(tally$add(by = 2, times = 3), 7)
  expect_equal(tally$add(2, 3), 13)
  expect_identical(tally$format_args(2, nsmall = 1), list(2, nsmall = 1))
  e <- caught(tally$format_args(digts = 2))
  expect_s3_class(e, "miniextendr_error_unused_argument")
  expect_identical(e$suggestion, "digits")
})

test_that("the S3 methods keep their generics' formals", {
  expect_length(tools::checkS3methods(package = "miniextendr"), 0L)
})
