# An `Either` argument's native-borrow check follows the arm its conversion
# takes (#1504): an arm that borrows is refused when it aliases another
# argument, and an owned arm is not refused because the other arm would have
# borrowed. Fixtures: src/rust/either_borrow_tests.rs.
#
# Inputs that a fixture may write to are built with `c()`, which allocates:
# a bare literal could be a constant shared with the test's own code.

ns <- asNamespace("miniextendr")
skip_if_not(exists("either_borrow_mut_left", envir = ns), "either feature not compiled in")

expect_alias <- function(expr, a = "a", b = "b") {
  err <- tryCatch(expr, error = identity)
  expect_s3_class(err, "error")
  expect_match(
    conditionMessage(err),
    sprintf("parameters `%s` and `%s` borrow the same non-empty R object", a, b),
    fixed = TRUE
  )
  invisible(err)
}

test_that("the issue's cases: a borrowing arm is refused, an owned arm is not", {
  x <- c(7L)
  # Left converts and would borrow `x` mutably while `b` reads it.
  expect_alias(ns$either_borrow_mut_left(x, x))
  # Left fails, Right converts and would borrow it.
  expect_alias(ns$either_borrow_mut_right(x, x))
  expect_identical(x, 7L)
  # Left copies the scalar: the Right arm, which would borrow, is unused.
  expect_identical(ns$either_borrow_owned_left(x, x), 7L)
  expect_identical(x, 8L)
})

test_that("distinct objects convert on the arm the value selects", {
  x <- c(7L)
  y <- c(2L)
  expect_identical(ns$either_borrow_mut_left(x, y), 9L)
  expect_identical(x, 9L)
  expect_identical(ns$either_borrow_mut_left("abc", x), 3L)
  expect_identical(ns$either_borrow_mut_right(x, y), 11L)
  expect_identical(x, 11L)
  expect_identical(ns$either_borrow_mut_right("ab", x), 2L)
})

test_that("slices follow the selected arm", {
  x <- c(1, 2, 3)
  expect_alias(ns$either_borrow_slice(x, x))
  expect_identical(x, c(1, 2, 3))
  expect_identical(ns$either_borrow_slice(1L, x), 7)
  y <- c(1, 1)
  expect_identical(ns$either_borrow_slice(y, x), 14)
  expect_identical(y, c(7, 7))
  # An empty vector borrows nothing.
  e <- numeric()
  expect_identical(ns$either_borrow_slice(e, e), 0)
})

test_that("a nested Either keeps the arm of each level", {
  d <- c(2.5)
  expect_alias(ns$either_borrow_nested(d, d))
  expect_identical(d, 2.5)
  expect_identical(ns$either_borrow_nested(3L, d), 5.5)
  expect_identical(ns$either_borrow_nested("ab", d), 4.5)
  e <- c(1.5)
  expect_identical(ns$either_borrow_nested(e, d), 4)
  expect_identical(e, 4)
})

test_that("Option, Missing and a derived newtype keep the arm", {
  x <- c(7L)
  y <- c(1L)
  expect_identical(ns$either_borrow_option(NULL, x), -1L)
  expect_alias(ns$either_borrow_option(x, x))
  expect_identical(ns$either_borrow_option("abc", x), 3L)
  expect_identical(ns$either_borrow_option(y, x), 8L)
  expect_identical(ns$either_borrow_missing(b = x), -1L)
  expect_alias(ns$either_borrow_missing(x, x))
  expect_identical(ns$either_borrow_missing("ab", x), 2L)
  expect_identical(x, 7L)

  # The derive forwards the preparation: the owned Left arm is accepted.
  expect_identical(ns$either_borrow_newtype_owned(x, x), 7L)
  expect_identical(x, 8L)
  expect_alias(ns$either_borrow_newtype_right(x, x))
  expect_identical(ns$either_borrow_newtype_right("abc", x), 3L)
  expect_identical(x, 8L)
})

test_that("a custom Left conversion decides the arm", {
  p <- c(5L)
  expect_identical(ns$either_borrow_positive(p, p), 5L)
  n <- c(-5L)
  expect_alias(ns$either_borrow_positive(n, n))
  expect_identical(n, -5L)
  expect_identical(ns$either_borrow_positive(n, c(2L)), -3L)
})

test_that("the checked arm is the arm the conversion took", {
  x <- c(7L)
  ns$either_borrow_flip_reset()
  # Call 1 succeeds: Left, converted once and kept.
  expect_identical(ns$either_borrow_flip(x, x), 0L)
  expect_identical(ns$either_borrow_flip_calls(), 1L)
  # Call 2 fails: the Right arm would borrow, so the call is refused. No
  # second Left conversion runs that could succeed after the check.
  expect_alias(ns$either_borrow_flip(x, x))
  expect_identical(ns$either_borrow_flip_calls(), 2L)
  expect_identical(x, 7L)
  # Without a conflict nothing is prepared: one conversion per call.
  expect_identical(ns$either_borrow_flip(x, c(1L)), 0L)
  expect_identical(ns$either_borrow_flip_calls(), 3L)
})

test_that("a prepared conversion's warning is raised once", {
  x <- c(7L)
  warnings <- 0L
  result <- withCallingHandlers(
    ns$either_borrow_noisy(x, x),
    warning = function(w) {
      warnings <<- warnings + 1L
      invokeRestart("muffleWarning")
    }
  )
  expect_identical(result, 14L)
  expect_identical(warnings, 1L)
})

test_that("both arms failing keeps the conversion error and the error order", {
  big <- c(500L)
  # `first` fails before `a`: its error is the one reported, although the
  # check prepared `a` first.
  err <- tryCatch(ns$either_borrow_order(-1L, big, big), error = identity)
  expect_s3_class(err, "rust_error")
  expect_identical(err$param, "first")
  err <- tryCatch(ns$either_borrow_order(1L, big, big), error = identity)
  expect_s3_class(err, "rust_error")
  expect_identical(err$param, "a")
  expect_match(conditionMessage(err), "not below 100", fixed = TRUE)
  small <- c(5L)
  expect_alias(ns$either_borrow_order(1L, small, small))
  expect_identical(ns$either_borrow_order(1L, small, c(2L)), 8L)
  expect_identical(small, 7L)
})

test_that("a choice Either borrows only for input that is not a choice", {
  x <- c(1, 2)
  expect_alias(ns$either_borrow_choice(x, x), a = "level")
  expect_identical(x, c(1, 2))
  expect_identical(ns$either_borrow_choice("low", x), 6)
  y <- c(1, 1)
  expect_identical(ns$either_borrow_choice(y, x), 8)
  expect_identical(y, c(4, 4))
})

test_that("an Either inside a tuple is checked against both arms", {
  x <- c(7L)
  err <- expect_alias(ns$either_borrow_in_tuple(list(x, "s"), x))
  expect_match(conditionMessage(err), "checked against both arms", fixed = TRUE)
  expect_identical(ns$either_borrow_in_tuple(list(c(1L), "s"), x), 8L)
})

test_that("trait methods check the arm, through the wrapper and the shims", {
  host <- ns$EitherBorrowHost$new()
  p <- c(5L)
  n <- c(-5L)
  expect_identical(host$EitherBorrowPick$pick(p, p), 5L)
  expect_alias(host$EitherBorrowPick$pick(n, n))
  expect_identical(host$EitherBorrowPickAs$pick_as(p, p), 5L)
  expect_alias(host$EitherBorrowPickAs$pick_as(n, n))
  expect_identical(n, -5L)
  for (generic in c(FALSE, TRUE)) {
    expect_identical(ns$either_borrow_shim(p, p, generic, FALSE), 5L)
    expect_alias(ns$either_borrow_shim(n, n, generic, FALSE))
    expect_identical(n, -5L)
    m <- c(-5L)
    expect_identical(ns$either_borrow_shim(m, c(2L), generic, FALSE), -3L)
    expect_identical(m, -3L)
  }
})

test_that("a default method the impl leaves out checks the arm too", {
  # Its wrapper and the generic trait's concrete shim convert through the
  # types inferred from the copied signature (#2123).
  host <- ns$EitherBorrowHost$new()
  p <- c(5L)
  n <- c(-5L)
  expect_identical(host$EitherBorrowPick$pick_default(p, p), 5L)
  expect_alias(host$EitherBorrowPick$pick_default(n, n))
  expect_identical(host$EitherBorrowPickAs$pick_as_default(p, p), 5L)
  expect_alias(host$EitherBorrowPickAs$pick_as_default(n, n))
  expect_identical(n, -5L)
  m <- c(-5L)
  expect_identical(host$EitherBorrowPick$pick_default(m, c(2L)), -3L)
  expect_identical(m, -3L)
  expect_identical(host$EitherBorrowPickAs$pick_as_default(m, c(2L)), 2L)
  for (generic in c(FALSE, TRUE)) {
    expect_identical(ns$either_borrow_shim(p, p, generic, TRUE), 5L)
    expect_alias(ns$either_borrow_shim(n, n, generic, TRUE))
    expect_identical(n, -5L)
  }
})

test_that("the gctorture fixture runs", {
  expect_identical(ns$gc_stress_either_borrow(), 3L)
})

test_that("the prepared arms keep their roots under gctorture", {
  skip_gc_stress_if_disabled()
  result <- tryCatch({
    gctorture(TRUE)
    ns$gc_stress_either_borrow()
  }, finally = gctorture(FALSE))
  expect_identical(result, 3L)
})
