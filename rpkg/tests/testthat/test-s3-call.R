# The call an S3 method's conditions report (#1851), and `call = none` on a
# function. Fixtures: src/rust/s3_call_tests.rs (an `mx_rec` is a classed list
# built here; `MxTally` is an impl-block class).
#
# Under `UseMethod()` dispatch R records the dispatched call in the method's
# frame, `summary.mx_rec(x)` for `summary(x)`, and a replacement method's call
# carries the new value, `` `$<-.mx_rec`(`*tmp*`, f, value = ...) ``. A
# generated method reports the generic's call instead, `summary(x)` and
# `x$f <- value`, whichever side raises the condition: the Rust body, a
# deferred warning, or the R-side argument checks. Called directly, by its
# own name, a method reports that call.

mx_rec <- function(a = 1) {
  structure(
    list(a = a, f = list(col = c(1, 2, 3)), locked = list(col = c(1, 2, 3))),
    class = "mx_rec"
  )
}

caught <- function(expr) tryCatch(expr, error = identity)
warned <- function(expr) tryCatch(expr, warning = identity)

# R CMD check runs these on the package: `checkS3methods()` compares each
# method's formals with its generic's, `checkReplaceFuns()` wants `value` as
# the last formal of every replacement function and method.
expect_s3_checks_clean <- function() {
  expect_length(tools::checkS3methods(package = "miniextendr"), 0L)
  expect_length(tools::checkReplaceFuns(package = "miniextendr"), 0L)
}

# region: a method on an ordinary generic

test_that("an error from a method names the generic's call", {
  x <- mx_rec(a = -1)
  e <- caught(summary(x))
  expect_s3_class(e, "rust_error")
  expect_match(conditionMessage(e), "`a` must be non-negative")
  expect_equal(conditionCall(e), quote(summary(x)))
  # The arguments stay as written.
  e <- caught(summary(x, extra = 1))
  expect_equal(conditionCall(e), quote(summary(x, extra = 1)))
  # Through `lapply()` the call is the one R made, under the generic's name.
  e <- caught(lapply(list(x), summary))
  expect_equal(conditionCall(e), quote(summary(X[[i]], ...)))
  # Called directly, the method reports its own call.
  e <- caught(summary.mx_rec(x))
  expect_equal(conditionCall(e), quote(summary.mx_rec(x)))
  expect_identical(summary(mx_rec(a = 2), 1, 2), 4)
})

test_that("a warning deferred from a method names the generic's call", {
  x0 <- mx_rec(a = 0)
  w <- warned(summary(x0))
  expect_s3_class(w, "rust_warning")
  expect_identical(conditionMessage(w), "`a` is zero")
  expect_equal(conditionCall(w), quote(summary(x0)))
  w <- warned(summary.mx_rec(x0))
  expect_equal(conditionCall(w), quote(summary.mx_rec(x0)))
  expect_identical(suppressWarnings(summary(x0)), 0)
})

test_that("a group generic method names the operator's call", {
  x <- mx_rec()
  e <- caught(x + 1)
  expect_s3_class(e, "rust_error")
  expect_equal(conditionCall(e), quote(x + 1))
  e <- caught(x == x)
  expect_equal(conditionCall(e), quote(x == x))
  e <- caught(Ops.mx_rec(x, 1))
  expect_equal(conditionCall(e), quote(Ops.mx_rec(x, 1)))
})

# endregion

# region: extraction generics

test_that("an extraction method names the generic's call", {
  x <- mx_rec()
  expect_identical(x[2], "f")
  expect_identical(x[c(1, 3)], c("a", "locked"))
  e <- caught(x[5])
  expect_equal(conditionCall(e), quote(x[5]))
  expect_identical(x$a, 1)
  e <- caught(x$nope)
  expect_equal(conditionCall(e), quote(x$nope))
  e <- caught(`$.mx_rec`(x, "nope"))
  expect_equal(conditionCall(e), quote(`$.mx_rec`(x, "nope")))
})

test_that("an argument error from a method names the generic's call", {
  x <- mx_rec()
  # `i` must be numeric: refused before the body runs.
  e <- caught(x["a"])
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "i")
  expect_equal(conditionCall(e), quote(x["a"]))
  e <- caught(`[.mx_rec`(x, "a"))
  expect_equal(conditionCall(e), quote(`[.mx_rec`(x, "a")))
})

# endregion

# region: replacement generics

test_that("a replacement method names a compact assignment call", {
  x <- mx_rec()
  e <- caught(x$locked <- 1)
  expect_s3_class(e, "rust_error")
  expect_match(conditionMessage(e), "can't be replaced")
  expect_equal(conditionCall(e), quote(x$locked <- value))
  e <- caught(x[["locked"]] <- 1)
  expect_equal(conditionCall(e), quote(x[["locked"]] <- value))
  e <- caught(x[[3]] <- 1)
  expect_equal(conditionCall(e), quote(x[[3]] <- value))
  e <- caught(x["locked"] <- list(1))
  expect_equal(conditionCall(e), quote(x["locked"] <- value))
  e <- caught(names(x) <- c("a", "b", "c"))
  expect_equal(conditionCall(e), quote(names(x) <- value))
  # A replacement that goes through lands.
  x$f <- 2
  expect_identical(x$f, 2)
  x[["a"]] <- 5
  expect_identical(x$a, 5)
  x["a"] <- list(6)
  expect_identical(x$a, 6)
  names(x) <- c("a", "g", "locked")
  expect_identical(names(x), c("a", "g", "locked"))
  expect_s3_checks_clean()
})

test_that("a replacement method called directly reports its own call", {
  x <- mx_rec()
  e <- caught(`$<-.mx_rec`(x, "locked", value = 1))
  expect_equal(conditionCall(e), quote(`$<-.mx_rec`(x, "locked", value = 1)))
})

test_that("a nested replacement reports the outer assignment, never the value", {
  x <- mx_rec()
  e <- caught(x$locked$col[1] <- 5)
  expect_equal(conditionCall(e), quote(x$locked <- value))
  e <- caught(x[["locked"]]$col <- 9)
  expect_equal(conditionCall(e), quote(x[["locked"]] <- value))
  # R hands the outer method the evaluated inner value, which the dispatched
  # call carries inlined: a thousand rows would deparse to hundreds of lines.
  big <- data.frame(n = seq_len(1000), s = as.character(seq_len(1000)))
  e <- caught(x$locked$col <- big)
  expect_length(deparse(conditionCall(e)), 1L)
  expect_equal(conditionCall(e), quote(x$locked <- value))
  e <- caught(x$locked <- big)
  expect_length(deparse(conditionCall(e)), 1L)
})

test_that("an argument error from a replacement method names the assignment", {
  x <- mx_rec()
  # `value` of `names<-` must be character.
  e <- caught(names(x) <- 1:3)
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "value")
  expect_equal(conditionCall(e), quote(names(x) <- value))
})

# endregion

# region: an impl-block S3 class

test_that("an impl-block method names the generic's call", {
  t <- new_mxtally(c(1, 2, 3))
  expect_identical(peek(t, 2L), 2)
  e <- caught(peek(t, 9L))
  expect_s3_class(e, "rust_error")
  expect_equal(conditionCall(e), quote(peek(t, 9L)))
  e <- caught(peek.MxTally(t, 9L))
  expect_equal(conditionCall(e), quote(peek.MxTally(t, 9L)))
  w <- warned(peek(t, 0L))
  expect_s3_class(w, "rust_warning")
  expect_equal(conditionCall(w), quote(peek(t, 0L)))
  expect_identical(suppressWarnings(peek(t, 0L)), NA_real_)
  # The R-side check of `i` (an integer scalar) too.
  e <- caught(peek(t, c(1L, 2L)))
  expect_identical(e$param, "i")
  expect_equal(conditionCall(e), quote(peek(t, c(1L, 2L))))
})

test_that("an impl-block operator method names the operator's call", {
  t <- new_mxtally(c(1, 2, 3))
  expect_identical(t[[3L]], 3)
  e <- caught(t[[5L]])
  expect_equal(conditionCall(e), quote(t[[5L]]))
  e <- caught(t[["a"]])
  expect_identical(e$param, "i")
  expect_equal(conditionCall(e), quote(t[["a"]]))
})

# endregion

# region: call = none

test_that("`call = none` on an S3 method drops the call however it is reached", {
  x <- mx_rec(a = -1)
  expect_null(conditionCall(caught(format(x))))
  expect_null(conditionCall(caught(format.mx_rec(x))))
  expect_null(conditionCall(caught(lapply(list(x), format))))
  w <- warned(format(mx_rec(a = 0)))
  expect_s3_class(w, "rust_warning")
  expect_null(conditionCall(w))
  expect_identical(format(mx_rec(a = 2)), "mx_rec(a = 2)")
})

test_that("`call = none` on a function drops the call from every condition", {
  # An error from the body.
  e <- caught(no_call_verb(-1L, "Fast"))
  expect_s3_class(e, "rust_error")
  expect_identical(conditionMessage(e), "x must be non-negative, got -1")
  expect_null(conditionCall(e))
  # A warning deferred from the body.
  w <- warned(no_call_verb(0L, "Fast"))
  expect_s3_class(w, "rust_warning")
  expect_null(conditionCall(w))
  expect_identical(suppressWarnings(no_call_verb(0L, "Fast")), 0L)
  # A panic.
  e <- caught(no_call_verb(1L, "Fast"))
  expect_match(conditionMessage(e), "x is one")
  expect_null(conditionCall(e))
  # A classed error.
  e <- caught(no_call_verb(2L, "Fast"))
  expect_s3_class(e, "pkg_two")
  expect_null(conditionCall(e))
  # The R-side check of `x` (an integer scalar).
  e <- caught(no_call_verb(1.5, "Fast"))
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "x")
  expect_null(e$rust_type)
  expect_null(conditionCall(e))
  # A failed conversion of `x` (`NA` is not an i32).
  e <- caught(no_call_verb(NA_integer_, "Fast"))
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "x")
  expect_null(conditionCall(e))
  # The `match_arg` check of `mode`.
  e <- caught(no_call_verb(3L, "Slow"))
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "mode")
  expect_null(conditionCall(e))
  expect_identical(no_call_verb(3L, "Fast"), 3L)
})

test_that("the sibling with the default attribution keeps its call", {
  expect_equal(conditionCall(caught(with_call_verb(-1L, "Fast"))), quote(with_call_verb(-1L, "Fast")))
  expect_equal(conditionCall(warned(with_call_verb(0L, "Fast"))), quote(with_call_verb(0L, "Fast")))
  expect_equal(conditionCall(caught(with_call_verb(1L, "Fast"))), quote(with_call_verb(1L, "Fast")))
  expect_equal(conditionCall(caught(with_call_verb(2L, "Fast"))), quote(with_call_verb(2L, "Fast")))
  expect_equal(conditionCall(caught(with_call_verb(1.5, "Fast"))), quote(with_call_verb(1.5, "Fast")))
  expect_equal(
    conditionCall(caught(with_call_verb(NA_integer_, "Fast"))),
    quote(with_call_verb(NA_integer_, "Fast"))
  )
  expect_equal(conditionCall(caught(with_call_verb(3L, "Slow"))), quote(with_call_verb(3L, "Slow")))
})

# endregion
