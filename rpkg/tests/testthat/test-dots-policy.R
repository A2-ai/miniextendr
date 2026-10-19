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
  expect_identical(caught(as.data.frame(fit, TRUE))$argument, "..1")
})

test_that("forward(allow = ...) passes the listed names and refuses the rest", {
  expect_equal(mx_dots_forward(2), 2)
  expect_equal(mx_dots_forward(2, scale = 3, shift = 1), 7)
  e <- caught(mx_dots_forward(2, sclae = 3))
  expect_s3_class(e, c("mx_dots_unused", "miniextendr_error_unused_argument"))
  expect_identical(class(e)[1:2], c("mx_dots_unused", "miniextendr_error_unused_argument"))
  expect_identical(e$suggestion, "scale")
  # A unique prefix of a listed name is suggested.
  expect_identical(caught(mx_dots_forward(2, sh = 1))$suggestion, "shift")
  # An unnamed argument is refused: the list holds names.
  expect_identical(caught(mx_dots_forward(2, 3))$argument, "..1")
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

test_that("an R6 method with a `...` parameter forwards the listed names", {
  tally <- MxDotsTally$new()
  expect_equal(tally$add(), 1)
  expect_equal(tally$add(by = 2, times = 3), 7)
  e <- caught(tally$add(bye = 2))
  expect_s3_class(e, "miniextendr_error_unused_argument")
  expect_identical(e$suggestion, "by")
  # The refused call changed nothing.
  expect_equal(tally$add(by = 0), 7)
})

test_that("the S3 methods keep their generics' formals", {
  expect_length(tools::checkS3methods(package = "miniextendr"), 0L)
})
