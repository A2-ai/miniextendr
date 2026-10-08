# Subscript forms in S3 `[` methods: `i` / `j` / `drop` as `Missing`
# parameters and the argument count through `NArgs` (#1860).
# Fixtures: src/rust/s3_subscript_tests.rs.

plain_frame <- function() {
  data.frame(a = 1:4, b = c("w", "x", "y", "z"), c = c(1.5, 2.5, 3.5, 4.5))
}

mx_frame <- function() {
  structure(plain_frame(), class = c("mx_frame", "data.frame"))
}

test_that("a forwarding `[` method matches `[.data.frame` for every form (#1860)", {
  f <- mx_frame()
  # Each form against `[.data.frame` called with the same arguments, as typed.
  expect_identical(f[2:3], `[.data.frame`(f, 2:3))
  expect_identical(f[2:3, ], `[.data.frame`(f, 2:3, ))
  expect_identical(f[, 2], `[.data.frame`(f, , 2))
  expect_identical(f[, c("a", "c")], `[.data.frame`(f, , c("a", "c")))
  expect_identical(f[2:3, 2:3], `[.data.frame`(f, 2:3, 2:3))
  expect_identical(f[2:3, 2], `[.data.frame`(f, 2:3, 2))
  expect_identical(f[2:3, 2, drop = FALSE], `[.data.frame`(f, 2:3, 2, drop = FALSE))
  expect_identical(f[], `[.data.frame`(f, ))

  # The forms differ where `[.data.frame` tells them apart by `nargs()`:
  # `f[2]` is the second column (list-style), `f[2, ]` the second row.
  expect_named(f[2], "b")
  expect_identical(nrow(f[2, ]), 1L)
  expect_identical(f[, 2], c("w", "x", "y", "z"))
  expect_s3_class(f[2:3, 2, drop = FALSE], "mx_frame")

  # A plain data frame gives the same values.
  d <- plain_frame()
  strip <- function(v) {
    if (is.data.frame(v)) class(v) <- "data.frame"
    v
  }
  expect_identical(strip(f[2:3]), d[2:3])
  expect_identical(strip(f[2:3, ]), d[2:3, ])
  expect_identical(f[, 2], d[, 2])
  expect_identical(strip(f[2:3, 2:3]), d[2:3, 2:3])
  expect_identical(strip(f[2:3, 2, drop = FALSE]), d[2:3, 2, drop = FALSE])
  expect_identical(strip(f[]), d[])
})

test_that("a forwarding `[` method passes `[.data.frame`'s own conditions on (#1860)", {
  f <- mx_frame()
  expect_error(f[, "nope"], "undefined columns selected")
  # `drop` in the list-style form: `[.data.frame` warns that it is ignored.
  expect_warning(
    expect_identical(f[2, drop = FALSE], suppressWarnings(`[.data.frame`(f, 2, drop = FALSE))),
    "'drop' argument will be ignored"
  )
})

test_that("a refusing `[` method accepts x[i] and x[] only (#1860)", {
  v <- structure(c(10, 20, 30), class = "mx_vec1")
  expect_identical(v[c(1L, 3L)], c(10, 30))
  expect_identical(v[], c(10, 20, 30))

  for (form in list(
    quote(v[1L, ]),
    quote(v[, 2L]),
    quote(v[1L, 2L]),
    quote(v[1L, 2L, drop = FALSE]),
    quote(v[1L, , drop = FALSE])
  )) {
    err <- tryCatch(eval(form), error = identity)
    expect_s3_class(err, "mx_vec1_subscript_error")
    expect_identical(err$subscripts, 2L, info = deparse(form))
    expect_match(conditionMessage(err), "subset it as x\\[i\\]")
  }
  # The empty subscript of `v[1L, ]` reaches the method instead of failing
  # in the wrapper with R's "argument is missing, with no default".
  expect_error(v[1L, ], class = "mx_vec1_subscript_error")
})

test_that("`NArgs` counts a replacement call as typed: x[i] <- v is 3, x[i, ] <- v is 4 (#1860)", {
  x <- structure(list(), class = "mx_nargs")
  x[1] <- 5
  expect_identical(attr(x, "nargs"), 3L)
  x[1, ] <- 5
  expect_identical(attr(x, "nargs"), 4L)
  x[1, 2] <- 5
  expect_identical(attr(x, "nargs"), 4L)
  x[] <- 5
  expect_identical(attr(x, "nargs"), 3L)
  expect_s3_class(x, "mx_nargs")
})

test_that("`NArgs` on a trait method counts the method's own call (#1860)", {
  p <- MxNargsProbe$new()
  expect_identical(MxNargsProbe$ArgCount$count_args(p, 1L), 2L)
  expect_identical(p$count_args(1L), 2L)
})

test_that("`NArgs` is no R formal (#1860)", {
  expect_identical(names(formals(getS3method("[", "mx_vec1"))), c("x", "i", "j", "drop"))
  expect_identical(names(formals(getS3method("[", "mx_frame"))), c("x", "i", "j", "drop"))
  expect_identical(names(formals(getS3method("[<-", "mx_nargs"))), c("x", "i", "j", "value"))
  expect_identical(names(formals(getS3method("[[", "MxBagHandle"))), c("x", "i", "..."))
})
