# Standalone S3 methods on extraction and replacement generics (#1853):
# `$`, `[[`, `$<-`, `[[<-`, `[<-`, `names<-`. Fixtures:
# src/rust/s3_replacement_tests.rs (an `mx_thing` is a classed list built here).
#
# A test runs in a child of the package namespace, where R finds every
# `<generic>.<class>` function by name, registered or not. `from_global()`
# evaluates its expression in a child of the global environment instead, so
# only the methods the package registered with R dispatch there, as for a
# user of the package.

mx_thing <- function() {
  structure(list(f = list(col = c(1, 2, 3)), g = "a"), class = "mx_thing")
}

from_global <- function(expr, ...) {
  eval(substitute(expr), list2env(list(...), parent = globalenv()))
}

# The fields without the class and the `mx_via` mark, for comparing with a
# plain list (`unclass()` keeps the other attributes).
fields <- function(x) {
  attributes(x) <- list(names = attr(x, "names"))
  x
}

# R CMD check runs these on the package: `checkS3methods()` compares each
# method's formals with its generic's, `checkReplaceFuns()` wants `value` as
# the last formal of every replacement function and method.
expect_s3_checks_clean <- function() {
  expect_length(tools::checkS3methods(package = "miniextendr"), 0L)
  expect_length(tools::checkReplaceFuns(package = "miniextendr"), 0L)
}

test_that("`$` reads a field by name", {
  x <- mx_thing()
  expect_identical(from_global(x$g, x = x), "a")
  expect_identical(from_global(x$f, x = x), list(col = c(1, 2, 3)))
  expect_error(from_global(x$nope, x = x), "an mx_thing has no field `nope`")
  expect_s3_checks_clean()
})

test_that("`[[` reads a field by name and by position", {
  x <- mx_thing()
  expect_identical(from_global(x[["g"]], x = x), "a")
  expect_identical(from_global(x[[2]], x = x), "a")
  expect_identical(from_global(x[[1L]], x = x), list(col = c(1, 2, 3)))
  expect_error(from_global(x[[5]], x = x), "no field at position 5")
  expect_error(from_global(x[["nope"]], x = x), "no field `nope`")
  expect_error(from_global(x[[TRUE]], x = x), "by name or by position")
  expect_s3_checks_clean()
})

test_that("`lapply()` and `str()` call the `[[` method", {
  # Wrap the registered method (the one in base's S3 methods table) to count
  # its calls; the wrapper forwards to it, so the Rust function still runs.
  table <- get(".__S3MethodsTable__.", envir = baseenv())
  method <- table[["[[.mx_thing"]]
  expect_identical(method, getS3method("[[", "mx_thing"))
  seen <- list()
  assign("[[.mx_thing", function(x, i) {
    seen[[length(seen) + 1L]] <<- i
    method(x, i)
  }, envir = table)
  withr::defer(assign("[[.mx_thing", method, envir = table))

  x <- mx_thing()
  expect_identical(lapply(x, length), list(f = 1L, g = 1L))
  expect_identical(seen, list(1L, 2L))
  seen <- list()
  invisible(capture.output(str(x)))
  expect_true(length(seen) > 0L)
  expect_s3_checks_clean()
})

test_that("`$<-` replaces a field, and gets the whole new field from `x$f$col[i] <- v`", {
  x <- from_global({
    x$g <- "b"
    x
  }, x = mx_thing())
  expect_s3_class(x, "mx_thing")
  expect_identical(attr(x, "mx_via"), "$<-")
  expect_identical(fields(x), list(f = list(col = c(1, 2, 3)), g = "b"))

  # R reads `x$f` through `$.mx_thing`, changes `col[2]` in that copy, and
  # hands `$<-.mx_thing` the whole new `f`.
  x <- from_global({
    x$f$col[2] <- 9
    x
  }, x = mx_thing())
  expect_identical(attr(x, "mx_via"), "$<-")
  expect_identical(fields(x), list(f = list(col = c(1, 9, 3)), g = "a"))

  expect_error(from_global(x$nope <- 1, x = mx_thing()), "no field `nope`")
  expect_s3_checks_clean()
})

test_that("`[[<-` replaces a field by name and by position", {
  x <- from_global({
    x[["g"]] <- "b"
    x
  }, x = mx_thing())
  expect_identical(attr(x, "mx_via"), "[[<-")
  expect_identical(fields(x)$g, "b")

  # `x[[2]]` passes a double, which a `&str` parameter would refuse.
  x <- from_global({
    x[[2]] <- "c"
    x
  }, x = mx_thing())
  expect_identical(attr(x, "mx_via"), "[[<-")
  expect_identical(fields(x)$g, "c")

  # `x[["f"]]$col <- v` reads through `[[`, writes back through `[[<-`.
  x <- from_global({
    x[["f"]]$col <- 0
    x
  }, x = mx_thing())
  expect_identical(attr(x, "mx_via"), "[[<-")
  expect_identical(fields(x)$f, list(col = 0))

  expect_error(from_global(x[[3]] <- 1, x = mx_thing()), "no field at position 3")
  expect_s3_checks_clean()
})

test_that("`[<-` takes the extra indices through `...` before `value`", {
  expect_identical(
    names(formals(getS3method("[<-", "mx_thing"))),
    c("x", "i", "...", "value")
  )
  x <- from_global({
    x["g"] <- list("b")
    x
  }, x = mx_thing())
  expect_identical(attr(x, "mx_via"), "[<-")
  expect_identical(fields(x)$g, "b")

  x <- from_global({
    x[c("g", "f")] <- list("c", 1)
    x
  }, x = mx_thing())
  expect_identical(fields(x), list(f = 1, g = "c"))

  # `x[1, 2] <- v` passes the 2 through `...`.
  expect_error(
    from_global(x[1, 2] <- list(1), x = mx_thing()),
    "takes one index in `\\[<-`, not 2"
  )
  expect_error(
    from_global(x[c("f", "g")] <- list(1), x = mx_thing()),
    "2 fields, 1 values"
  )
  expect_s3_checks_clean()
})

test_that("`names<-` gets the whole new names vector, also from `names(x)[i] <- v`", {
  x <- from_global({
    names(x) <- c("a", "b")
    x
  }, x = mx_thing())
  expect_identical(attr(x, "mx_via"), "names<-")
  expect_identical(names(fields(x)), c("a", "b"))

  x <- from_global({
    names(x)[2] <- "h"
    x
  }, x = mx_thing())
  expect_identical(attr(x, "mx_via"), "names<-")
  expect_identical(names(fields(x)), c("f", "h"))

  expect_error(
    from_global(names(x)[2] <- "f", x = mx_thing()),
    "`f` \\(field 2\\) repeats an earlier one"
  )
  expect_s3_checks_clean()
})

test_that("`modifyList()` goes through `[[<-`", {
  x <- from_global(modifyList(x, list(g = "z")), x = mx_thing())
  expect_s3_class(x, "mx_thing")
  expect_identical(attr(x, "mx_via"), "[[<-")
  expect_identical(fields(x)$g, "z")
  expect_error(
    from_global(modifyList(x, list(h = 1)), x = mx_thing()),
    "no field `h`"
  )
  expect_s3_checks_clean()
})

test_that("the generated argument checks run before the Rust method", {
  # A `List` receiver is checked with `is.list(x)`, a `&str` parameter for a
  # length-1 character vector: a number given as the name of `$<-` is refused
  # before the method runs.
  expect_error(getS3method("$", "mx_thing")(1:3, "f"), "'x' must be a list")
  expect_error(
    getS3method("$<-", "mx_thing")(mx_thing(), 2, 1),
    "'name' must be character"
  )
  expect_error(
    getS3method("$", "mx_thing")(mx_thing(), c("f", "g")),
    "'name' must have length 1"
  )
  expect_s3_checks_clean()
})

test_that("the replacement methods keep their copy rooted under gctorture", {
  skip_gc_stress_if_disabled()
  x <- mx_thing()
  result <- tryCatch({
    gctorture(TRUE)
    x$f$col[2] <- 9
    x[[2]] <- "b"
    x["g"] <- list("c")
    names(x)[2] <- "h"
    miniextendr:::gc_stress_s3_replacement()
  }, finally = gctorture(FALSE))
  expect_null(result)
  expect_identical(fields(x), list(f = list(col = c(1, 9, 3)), h = "c"))
  expect_identical(attr(x, "mx_via"), "names<-")
})

test_that("an impl-block `[[<-` method takes `value` last and returns the handle", {
  expect_identical(
    names(formals(getS3method("[[<-", "MxBagHandle"))),
    c("x", "i", "...", "value")
  )
  h <- new_mxbaghandle(c(10, 20))
  h <- from_global({
    h[[2L]] <- 5
    h
  }, h = h)
  expect_s3_class(h, "MxBagHandle")
  expect_identical(from_global(h[[2L]], h = h), 5)
  expect_error(from_global(h[[3L]] <- 1, h = h), "index 3 out of range for 2 values")
  expect_s3_checks_clean()
})
