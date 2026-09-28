# Parameters named like a base function the generated wrapper calls
# (src/rust/shadowed_formal_tests.rs). The wrapper writes those calls
# `base::name(...)`, so omitting such an argument, or passing a function there,
# affects only that argument.

caught <- function(expr) tryCatch(expr, error = identity)
missing_length <- 'argument "length" is missing, with no default'

test_that("a formal named length leaves the other guards alone", {
  expect_identical(miniextendr:::shadowed_length(length = 2), 2)
  expect_identical(miniextendr:::shadowed_length(TRUE, 3), 3)
  # Omitted: the ordinary missing-argument error, not one from `overwrite`'s
  # length check.
  e <- caught(miniextendr:::shadowed_length())
  expect_identical(conditionMessage(e), missing_length)
  expect_identical(conditionCall(e), quote(miniextendr:::shadowed_length()))
  # A function: refused by `length`'s own type guard, not by `overwrite`'s
  # length check (`mean(FALSE)` is `0`).
  e <- caught(miniextendr:::shadowed_length(length = mean))
  expect_identical(conditionMessage(e), "'length' must be double")
  expect_identical(e$param, "length")
  # `mean(TRUE)` is `1`, which used to pass `overwrite`'s check by accident.
  e <- caught(miniextendr:::shadowed_length(TRUE, mean))
  expect_identical(e$param, "length")
})

test_that("a formal named c leaves the choice formal's default alone", {
  expect_identical(miniextendr:::shadowed_c(), "fast")
  expect_identical(miniextendr:::shadowed_c("slow"), "slow")
  # The choices come from base::c, not from the caller's function.
  expect_identical(miniextendr:::shadowed_c(c = function(...) "caller"), "fast")
  expect_identical(miniextendr:::shadowed_c("slow", c = 3), "slow")
  expect_identical(formals(miniextendr:::shadowed_c)$mode, quote(base::c("fast", "slow")))
  e <- caught(miniextendr:::shadowed_c("medium", c = mean))
  expect_identical(e$param, "mode")
})

test_that("formals named missing and quote leave the forwarding alone", {
  expect_false(miniextendr:::shadowed_missing())
  expect_true(miniextendr:::shadowed_missing(1))
  expect_false(miniextendr:::shadowed_missing(missing = mean, quote = mean))
  expect_true(miniextendr:::shadowed_missing(1, missing = mean, quote = mean))
  expect_true(miniextendr:::shadowed_missing(1, missing = 3, quote = 4))
  # The `no_na` check on `x` still runs.
  e <- caught(miniextendr:::shadowed_missing(NA_real_, missing = mean))
  expect_identical(conditionMessage(e), "'x' must not be NA")
  expect_identical(e$param, "x")
})

test_that("a formal named attr leaves the check after .Call() alone", {
  expect_identical(miniextendr:::shadowed_attr(FALSE), 1)
  expect_identical(miniextendr:::shadowed_attr(FALSE, attr = mean), 1)
  # The Rust error is raised, whatever `attr` is.
  expect_error(miniextendr:::shadowed_attr(TRUE), "shadowed_attr failed", class = "rust_error")
  expect_error(
    miniextendr:::shadowed_attr(TRUE, attr = mean),
    "shadowed_attr failed",
    class = "rust_error"
  )
  expect_error(miniextendr:::shadowed_attr(TRUE, attr = 3), "shadowed_attr failed", class = "rust_error")
})

test_that("a formal named list leaves the dots forwarding alone", {
  expect_identical(miniextendr:::shadowed_list(), 0L)
  expect_identical(miniextendr:::shadowed_list(list = mean, 1, 2, 3), 3L)
  expect_identical(miniextendr:::shadowed_list(mean, 1, 2), 2L)
  expect_identical(miniextendr:::shadowed_list(list = 3, 1), 1L)
})

test_that("an R6 method's formal named length leaves its guards alone", {
  holder <- miniextendr:::ShadowedHolder$new()
  expect_identical(holder$resize(length = 2), 2)
  e <- caught(holder$resize())
  expect_identical(conditionMessage(e), missing_length)
  e <- caught(holder$resize(length = mean))
  expect_identical(conditionMessage(e), "'length' must be double")
  expect_identical(e$param, "length")
})

# region: every wrapper

# Calls, anywhere in `expr`, to a bare function name that is a formal in scope
# and also a base function. A function's formal defaults are evaluated in its
# own frame, so they see its formals too.
shadowed_calls <- function(expr, scope = character(), where = "<top level>") {
  if (!is.call(expr)) {
    return(character())
  }
  head <- expr[[1L]]
  if (identical(head, quote(`function`))) {
    formals <- as.list(expr[[2L]])
    scope <- union(scope, names(formals))
    parts <- c(formals, list(expr[[3L]]))
  } else {
    parts <- as.list(expr)
  }
  found <- character()
  if (is.symbol(head)) {
    name <- as.character(head)
    if (name %in% scope && is.function(get0(name, envir = baseenv(), inherits = FALSE))) {
      found <- sprintf("%s: %s()", where, name)
    }
  }
  # By index: a formal without a default is the empty symbol, which a loop
  # variable cannot hold.
  for (i in seq_along(parts)) {
    found <- c(found, shadowed_calls(parts[[i]], scope, where))
  }
  found
}

test_that("no generated wrapper calls a base function that one of its formals shadows", {
  path <- test_path("../../R/miniextendr-wrappers.R")
  # Generated at install time; a tarball check runs without it.
  skip_if_not(file.exists(path))
  exprs <- parse(path, keep.source = FALSE)
  found <- character()
  for (expr in exprs) {
    where <- if (is.call(expr) && length(expr) >= 2L) deparse1(expr[[2L]]) else "<top level>"
    found <- c(found, shadowed_calls(expr, where = where))
  }
  expect_identical(found, character())
})

# endregion
