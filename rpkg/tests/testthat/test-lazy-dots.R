# The unforced dots, `LazyDots` (#1892): src/rust/lazy_dots_tests.rs.
#
# A `LazyDots` parameter is R's `...`, passed as the wrapper's own frame
# (`environment()`) instead of `list(...)`: the body counts, names and reads
# the dots without forcing them, sees the empty ones, and forces each element
# in its own environment. The forcing fixtures hold a Rust value whose
# destructor counts its runs (`lazy_dots_sentinel_drops()`), so an R exit that
# leaves through the Rust frames can be seen to drop them.

drops <- function() miniextendr:::lazy_dots_sentinel_drops()

# region: reading without forcing

test_that("the count, emptiness and names force nothing", {
  expect_identical(miniextendr:::lazy_dots_len(), 0L)
  expect_identical(miniextendr:::lazy_dots_len(a = stop("forced"), , 3), 3L)
  expect_true(miniextendr:::lazy_dots_is_empty())
  expect_false(miniextendr:::lazy_dots_is_empty(stop("forced")))
  expect_false(miniextendr:::lazy_dots_is_empty(, ))

  expect_identical(
    miniextendr:::lazy_dots_names(a = stop("forced"), , b = , 4),
    c("a", NA, "b", NA)
  )
  expect_identical(miniextendr:::lazy_dots_names(1, 2), c(NA_character_, NA_character_))
  expect_identical(miniextendr:::lazy_dots_names(), character(0))
  # A repeated name is kept.
  expect_identical(miniextendr:::lazy_dots_names(a = 1, a = 2), c("a", "a"))
})

test_that("is_missing_arg() is TRUE for an empty element only", {
  # `f(a = )`, a positional empty and `x[1, , , ]`-style trailing empties.
  expect_identical(
    miniextendr:::lazy_dots_missing(a = , stop("forced"), , b = 1, ),
    c(TRUE, FALSE, TRUE, FALSE, TRUE)
  )
  expect_identical(
    miniextendr:::lazy_dots_describe(a = , stop("forced"), b = 2),
    c("a:empty", ":given", "b:given")
  )
  # An argument that evaluates to the empty symbol is not an empty element.
  empty <- quote(expr = )
  expect_identical(miniextendr:::lazy_dots_missing(empty), FALSE)
})

test_that("expr() is each element as written, the empty symbol for an empty one", {
  out <- miniextendr:::lazy_dots_exprs(a = x + 1, , stop("forced"), "lit")
  expect_identical(out[[1]], quote(x + 1))
  expect_identical(out[[2]], quote(expr = ))
  expect_identical(out[[3]], quote(stop("forced")))
  expect_identical(out[[4]], "lit")
  # `ID` is unbound here: reading it does not evaluate it.
  expect_identical(miniextendr:::lazy_dots_exprs(select = ID), list(quote(ID)))
})

test_that("forwarded dots keep their expressions and empty elements", {
  fwd <- function(...) miniextendr:::lazy_dots_exprs(first = y, ...)
  out <- fwd(x + 1, , z)
  expect_identical(out, list(quote(y), quote(x + 1), quote(expr = ), quote(z)))
  fwd_missing <- function(...) miniextendr:::lazy_dots_missing(...)
  expect_identical(fwd_missing(1, , 3), c(FALSE, TRUE, FALSE))
  # Through two levels of forwarding.
  fwd2 <- function(...) fwd_missing(...)
  expect_identical(fwd2(, b = 2), c(TRUE, FALSE))
})

test_that("a forwarded missing argument counts as present", {
  # `a` is missing in `k()`, but the element of `...` is the promise for `a`,
  # not the empty symbol.
  k <- function(a) miniextendr:::lazy_dots_missing(1, a)
  expect_identical(k(), c(FALSE, FALSE))
  k_exprs <- function(a) miniextendr:::lazy_dots_exprs(a)
  expect_identical(k_exprs(), list(quote(a)))
})

test_that("a byte-compiled caller passes its dots the same way", {
  f <- compiler::cmpfun(function(...) miniextendr:::lazy_dots_describe(...))
  expect_identical(f(a = , b = stop("forced")), c("a:empty", "b:given"))
  g <- compiler::cmpfun(function(...) miniextendr:::lazy_dots_force(...))
  y <- 3
  expect_identical(g(y * 2), list(6))
})

# endregion

# region: forcing

test_that("force() evaluates each element in its own environment", {
  y <- 1
  outer <- function(...) {
    y <- 10
    miniextendr:::lazy_dots_force(first = y, ...)
  }
  expect_identical(outer(y + 1, y * 3), list(10, 2, 3))
  expect_identical(miniextendr:::lazy_dots_force_at(1L, stop("never"), y + 5), 6)

  # h -> g -> the fixture: each element is forced where it was written.
  g <- function(...) {
    y <- 20
    miniextendr:::lazy_dots_force(from_g = y, ...)
  }
  h <- function(...) {
    y <- 30
    g(from_h = y, ...)
  }
  expect_identical(h(y), list(20, 30, 1))
})

test_that("force() forces an element once", {
  n <- 0
  out <- miniextendr:::lazy_dots_force_twice({
    n <- n + 1
    n
  })
  expect_identical(out, list(1, 1))
  expect_identical(n, 1)
})

test_that("forcing an empty element raises R's missing-argument error", {
  before <- drops()
  e <- tryCatch(miniextendr:::lazy_dots_force(1, , 3), error = identity)
  expect_s3_class(e, "error")
  expect_identical(conditionMessage(e), 'argument "..2" is missing, with no default')
  if (getRversion() >= "4.6.0") {
    expect_s3_class(e, "missingArgError")
  } else {
    expect_s3_class(e, "simpleError")
  }
  # The call is the generated function's own (checked on R 4.4.3 and 4.6.1).
  expect_equal(conditionCall(e), quote(miniextendr:::lazy_dots_force(1, , 3)))
  f <- compiler::cmpfun(function() miniextendr:::lazy_dots_force(1, , 3))
  expect_equal(conditionCall(tryCatch(f(), error = identity)), quote(miniextendr:::lazy_dots_force(1, , 3)))
  expect_identical(drops() - before, 2L)

  # A forwarded missing argument raises when forced, naming its own frame.
  k <- function(a) miniextendr:::lazy_dots_force(1, a)
  e <- tryCatch(k(), error = identity)
  expect_identical(conditionMessage(e), 'argument "a" is missing, with no default')
  expect_equal(conditionCall(e), quote(k()))

  # Skipping the empty elements forces the others.
  expect_identical(miniextendr:::lazy_dots_force_present(1, , 3), list(1, NULL, 3))
})

test_that("an index past the end panics before any R code runs", {
  hit <- FALSE
  e <- tryCatch(
    miniextendr:::lazy_dots_force_at(2L, hit <- TRUE, stop("never")),
    error = identity
  )
  expect_s3_class(e, "rust_error")
  expect_match(
    conditionMessage(e),
    "index out of bounds: `...` has 2 elements but the index is 2",
    fixed = TRUE
  )
  expect_false(hit)
})

test_that("conditions from a forced element reach the caller's handlers", {
  # A calling handler sees the warning and muffles it; the value comes back.
  seen <- NULL
  out <- withCallingHandlers(
    miniextendr:::lazy_dots_force({
      warning("careful")
      1
    }),
    warning = function(w) {
      seen <<- conditionMessage(w)
      invokeRestart("muffleWarning")
    }
  )
  expect_identical(out, list(1))
  expect_identical(seen, "careful")
  expect_identical(suppressWarnings(miniextendr:::lazy_dots_force({
    warning("quiet")
    2
  })), list(2))

  # An error keeps its own class, and the Rust frames drop on the way out.
  before <- drops()
  e <- tryCatch(
    miniextendr:::lazy_dots_force(1, stop(errorCondition("boom", class = "mx_lazy_boom"))),
    error = identity
  )
  expect_identical(class(e), c("mx_lazy_boom", "error", "condition"))
  expect_identical(conditionMessage(e), "boom")
  expect_identical(drops() - before, 1L)

  # An exiting handler and a restart leave through the Rust frames too.
  before <- drops()
  w <- tryCatch(miniextendr:::lazy_dots_force(warning("exit")), warning = identity)
  expect_identical(conditionMessage(w), "exit")
  out <- withRestarts(
    miniextendr:::lazy_dots_force(invokeRestart("mx_restart", 5)),
    mx_restart = function(v) v * 2
  )
  expect_identical(out, 10)
  expect_identical(drops() - before, 2L)
})

test_that("try_force() returns an R error as a value and keeps the caller's handlers", {
  before <- drops()
  out <- miniextendr:::lazy_dots_try_force(
    1,
    stop(errorCondition("boom", class = "mx_lazy_boom")),
    ,
    "four"
  )
  expect_identical(out[[1]], 1)
  expect_s3_class(out[[2]], "mx_lazy_boom")
  expect_identical(conditionMessage(out[[2]]), "boom")
  expect_s3_class(out[[3]], "error")
  expect_identical(conditionMessage(out[[3]]), 'argument "..3" is missing, with no default')
  expect_identical(out[[4]], "four")
  expect_identical(drops() - before, 1L)

  # Warnings go to the caller's handlers, not into the result.
  seen <- NULL
  out <- withCallingHandlers(
    miniextendr:::lazy_dots_try_force({
      warning("careful")
      2
    }),
    warning = function(w) {
      seen <<- conditionMessage(w)
      invokeRestart("muffleWarning")
    }
  )
  expect_identical(out, list(2))
  expect_identical(seen, "careful")

  # A non-error exit still leaves through the Rust frames.
  before <- drops()
  w <- tryCatch(miniextendr:::lazy_dots_try_force(warning("exit")), warning = identity)
  expect_identical(conditionMessage(w), "exit")
  expect_identical(drops() - before, 1L)
})

# endregion

# region: S3 methods

lazy_tbl <- structure(
  data.frame(ID = 1:3, v = c(2, 4, 6)),
  class = c("mx_lazy", "data.frame")
)

test_that("subset() refuses extra arguments before evaluating them", {
  # `ID` is a column, not a variable: evaluating `select = ID` here would
  # fail with "object 'ID' not found".
  e <- tryCatch(subset(lazy_tbl, TRUE, select = ID), error = identity)
  expect_s3_class(e, "mx_lazy_unsupported_args")
  expect_identical(
    conditionMessage(e),
    "subset() on an mx_lazy takes no further arguments, got: select"
  )
  expect_equal(conditionCall(e), quote(subset(lazy_tbl, TRUE, select = ID)))
  e <- tryCatch(subset(lazy_tbl, v > 2, stop("forced"), drop = ), error = identity)
  expect_s3_class(e, "mx_lazy_unsupported_args")
  expect_match(conditionMessage(e), "got: ..1, drop", fixed = TRUE)

  out <- subset(lazy_tbl, v > 2)
  expect_s3_class(out, "mx_lazy")
  expect_identical(out$ID, 2:3)
  expect_identical(subset(lazy_tbl), lazy_tbl)
})

test_that("update() reads empty steps and forces the others", {
  out <- update(lazy_tbl, select = , filter = 1 + 1)
  expect_identical(out$steps, c("select", "filter"))
  expect_identical(out$empty, c(TRUE, FALSE))
  expect_identical(out$values, list(NULL, 2))
  e <- tryCatch(update(lazy_tbl, stop("forced")), error = identity)
  expect_s3_class(e, "mx_lazy_unsupported_args")
  expect_identical(
    conditionMessage(e),
    "update() on an mx_lazy takes no further arguments, got: ..1"
  )
})

test_that("`[` sees x[1, , , ] with Missing formals, NArgs and empty dots", {
  # `[.mx_lazy_grid` <- function(x, i, j, ..., drop): base `[`'s order.
  grid <- structure(list(), class = "mx_lazy_grid")
  expect_identical(grid[1, , , ], "nargs=5 i=given j=empty drop=empty dots=[<empty>, <empty>]")
  expect_identical(
    grid[1, , , 2, ],
    "nargs=6 i=given j=empty drop=empty dots=[<empty>, 2, <empty>]"
  )
  expect_identical(grid[1], "nargs=2 i=given j=empty drop=empty dots=[]")
  expect_identical(grid[1, ], "nargs=3 i=given j=empty drop=empty dots=[]")
  expect_identical(grid[, 2], "nargs=3 i=empty j=given drop=empty dots=[]")
  expect_identical(
    grid[1, 2, 3, , drop = FALSE],
    "nargs=6 i=given j=given drop=given dots=[3, <empty>]"
  )
  expect_identical(
    grid[1, , , k = , 4],
    "nargs=6 i=given j=empty drop=empty dots=[<empty>, k=<empty>, 4]"
  )
  # An element that is not a number is forced and reported as such.
  expect_identical(grid[, , "a"], "nargs=4 i=empty j=empty drop=empty dots=[?]")
})

# endregion

# region: class systems

test_that("an env-class method takes LazyDots, through $ and [[", {
  obj <- miniextendr:::LazyDotsEnv$new(1)
  expect_identical(obj$report(a = stop("forced"), , 3), c("a:given", ":empty", ":given"))
  expect_identical(obj$sum(a = 1, , 3), 5)
  expect_identical(obj[["sum"]](2), 3)
  expect_identical(obj$report(), character(0))
})

test_that("an R6 method takes LazyDots", {
  obj <- miniextendr:::LazyDotsR6$new(1)
  expect_identical(obj$report(a = , b = stop("forced")), c("a:empty", "b:given"))
  expect_identical(obj$sum(1, , 2), 4)
})

test_that("an S3 method takes LazyDots, forwarded through UseMethod()", {
  obj <- new_lazydotss3(1)
  expect_identical(lazy_s3_report(obj, a = , stop("forced")), c("a:empty", ":given"))
  expect_identical(lazy_s3_sum(obj, 1, , 2), 4)
})

test_that("an S4 method takes LazyDots, through the .local rewrite", {
  obj <- miniextendr:::LazyDotsS4(1)
  expect_identical(
    miniextendr:::s4_lazy_report(obj, 2L, a = , stop("forced")),
    c("n=2", "a:empty", ":given")
  )
  expect_identical(miniextendr:::s4_lazy_sum(obj, 1, , 2), 4)
})

test_that("an S7 method takes LazyDots, through dispatch and the shortcut", {
  obj <- miniextendr:::LazyDotsS7(1)
  expect_identical(
    miniextendr:::lazy_s7_report(obj, a = , stop("forced")),
    c("a:empty", ":given")
  )
  expect_identical(
    miniextendr:::LazyDotsS7_lazy_s7_report(obj, a = , stop("forced")),
    c("a:empty", ":given")
  )
  expect_identical(miniextendr:::lazy_s7_sum(obj, 1, , 2), 4)
})

test_that("a vctrs static method and format() take LazyDots", {
  expect_identical(
    miniextendr:::lazydotsvctrs_report(a = , stop("forced")),
    c("a:empty", ":given")
  )
  v <- miniextendr:::new_lazydotsvctrs(c(1, 2))
  expect_identical(
    format(v, a = , stop("forced")),
    c("1|a:empty,:given", "2|a:empty,:given")
  )
})

# endregion

# region: gctorture

test_that("the LazyDots fixtures hold their values under gctorture", {
  skip_gc_stress_if_disabled()
  # Load the package first, then enable gctorture — see docs/GCTORTURE_TESTING.md.
  run <- function() {
    y <- 2
    stopifnot(
      identical(
        miniextendr:::lazy_dots_exprs(a = x + 1, , paste0("b", y)),
        list(quote(x + 1), quote(expr = ), quote(paste0("b", y)))
      ),
      identical(miniextendr:::lazy_dots_names(a = 1, , b = 2), c("a", NA, "b")),
      identical(miniextendr:::lazy_dots_force_present(y, , paste0("b", y)), list(2, NULL, "b2")),
      identical(
        vapply(
          miniextendr:::lazy_dots_try_force(stop("boom"), ),
          conditionMessage,
          ""
        ),
        c("boom", 'argument "..2" is missing, with no default')
      ),
      identical(
        update(lazy_tbl, select = , filter = y),
        list(steps = c("select", "filter"), empty = c(TRUE, FALSE), values = list(NULL, 2))
      )
    )
  }
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  ok <- 0L
  fail <- character(0L)
  for (i in seq_len(5L)) {
    res <- tryCatch(
      {
        run()
        "ok"
      },
      error = function(e) conditionMessage(e)
    )
    if (identical(res, "ok")) {
      ok <- ok + 1L
    } else {
      fail <- c(fail, sprintf("iteration %d: %s", i, res))
    }
  }
  gctorture(FALSE)
  expect_equal(ok, 5L, info = paste("failures:", paste(fail, collapse = "; ")))
})

# endregion
