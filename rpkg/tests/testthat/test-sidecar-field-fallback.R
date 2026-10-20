# The field fallback of `r_data_accessors` (#2150): `fallback = "Self::f"`
# answers, on the bare pointer, the names no `#[r_data]` field has, and
# `fallback_names = "Self::g"` adds its names to `.DollarNames()`. Fields
# win, a `[[` position counts the fields only, `names()` / `as.list()` and
# the writes stay the fields', no name matches a prefix, and the fallback's
# `None` reads what an unknown name reads without it: `miniextendr_no_field`
# on S3 (default lookup) and S4, `NULL` under `lookup = "list"` and on an
# env class, whose `$` tries the methods before the fallback.
#
# Fixtures: `FallbackS3` (`fallback_rec`), `FallbackList` (`fallback_list`),
# `FallbackS4` and `FallbackEnv` in `src/rust/rdata_fallback_tests.rs`.

caught <- function(expr) tryCatch(expr, error = identity)

reload <- function(x) {
  path <- tempfile(fileext = ".rds")
  on.exit(unlink(path), add = TRUE)
  saveRDS(x, path)
  readRDS(path)
}

no_field <- c("fallback_error_field", "fallback_error", "miniextendr_no_field")

# region: S3, the default lookup

test_that("S3: a name no field has reads the fallback's value, by `$` and `[[`", {
  x <- new_fallback_rec(7L)
  expect_identical(x$alpha, c(1, 2))
  expect_identical(x[["alpha"]], c(1, 2))
  expect_identical(x$beta, 7)
  expect_identical(x$aleph, numeric())
})

test_that("S3: a field wins over the fallback's value of the same name", {
  x <- new_fallback_rec(7L)
  expect_identical(x$id, 7L)
  expect_identical(x[["id"]], 7L)
  expect_identical(x$columns, c("alpha", "beta", "id", "aleph"))
})

test_that("S3: a `[[` position counts the fields only", {
  x <- new_fallback_rec(7L)
  expect_identical(x[[1]], 7L)
  expect_identical(x[[2]], c("alpha", "beta", "id", "aleph"))
  oob <- c("fallback_error_field", "fallback_error", "subscriptOutOfBoundsError")
  for (read in alist(x[[3]], x[[0]])) {
    e <- caught(eval(read))
    expect_identical(class(e)[1:3], oob, label = deparse(read))
    expect_equal(conditionCall(e), read, label = deparse(read))
  }
})

test_that("S3: the fallback's `None` raises `miniextendr_no_field`, with the type's classes and hint", {
  x <- new_fallback_rec(7L)
  expected <- paste0(
    "`nope` is not a field of `fallback_rec`; its fields are `id`, `columns`. ",
    "Its columns are `x$columns`."
  )
  for (read in alist(x$nope, x[["nope"]])) {
    e <- caught(eval(read))
    expect_identical(class(e)[1:3], no_field, label = deparse(read))
    expect_identical(conditionMessage(e), expected, label = deparse(read))
    expect_equal(conditionCall(e), read, label = deparse(read))
  }
})

test_that("S3: no name matches a prefix of a field or of a fallback name", {
  x <- new_fallback_rec(7L)
  withr::local_options(warnPartialMatchDollar = TRUE)
  # `alp` is a prefix of `alpha` and `aleph`, `col` of the field `columns`.
  for (read in alist(x$alp, x$al, x$col, x[["alp"]], x[["col"]])) {
    e <- caught(eval(read))
    expect_identical(class(e)[1:3], no_field, label = deparse(read))
    expect_equal(conditionCall(e), read, label = deparse(read))
  }
})

test_that("S3: the fallback's own error reaches R with its class and the call", {
  x <- new_fallback_rec(7L)
  for (read in alist(x$boom, x[["boom"]])) {
    e <- caught(eval(read))
    expect_s3_class(e, "fallback_boom")
    expect_identical(conditionMessage(e), "no column of table 7 can be named `boom`")
    expect_equal(conditionCall(e), read, label = deparse(read))
  }
})

test_that("S3: names() and as.list() are the fields only", {
  x <- new_fallback_rec(7L)
  expect_identical(names(x), c("id", "columns"))
  expect_identical(
    as.list(x),
    list(id = 7L, columns = c("alpha", "beta", "id", "aleph"))
  )
})

test_that("S3: an assignment to a fallback name is refused; a field's is not", {
  x <- new_fallback_rec(7L)
  e <- caught(x$alpha <- 1)
  expect_identical(class(e)[1:3], no_field)
  expect_equal(conditionCall(e), quote(x$alpha <- value))
  e <- caught(x[["alpha"]] <- 1)
  expect_identical(class(e)[1:3], no_field)
  expect_identical(x$alpha, c(1, 2))
  x$id <- 8L
  expect_identical(x$id, 8L)
  expect_identical(x$beta, 7)
})

test_that("S3: .DollarNames() offers the fields, then the fallback's names, each once", {
  x <- new_fallback_rec(7L)
  # The column `id` is the field's name too, so it is listed once.
  expect_identical(
    utils::.DollarNames(x),
    c("id", "columns", "alpha", "beta", "aleph")
  )
  expect_identical(utils::.DollarNames(x, "^a"), c("alpha", "aleph"))
  expect_identical(utils::.DollarNames(x, "^i"), "id")
  expect_identical(utils::.DollarNames(x, "^z"), character(0))
})

test_that("S3: a method of another impl block sits next to the fallback", {
  x <- new_fallback_rec(7L)
  expect_identical(n_columns(x), 4L)
  expect_identical(x$alpha, c(1, 2))
})

test_that("S3: a restored object is refused before the fallback, and completes to its fields", {
  back <- reload(new_fallback_rec(7L))
  classes <- c(
    "fallback_error_saved", "fallback_error",
    "miniextendr_restored_no_value", "miniextendr_restored"
  )
  for (read in alist(back$alpha, back[["alpha"]], back$nope, back$id)) {
    e <- caught(eval(read))
    expect_identical(class(e)[1:4], classes, label = deparse(read))
    expect_equal(conditionCall(e), read, label = deparse(read))
  }
  # The field names need no Rust value; the fallback's do (#2218).
  expect_identical(utils::.DollarNames(back), c("id", "columns"))
})

test_that("S3: a list carrying the pointer keeps its own elements for other names", {
  l <- structure(list(.ptr = new_fallback_rec(7L), note = "n"), class = "fallback_rec")
  expect_identical(l$id, 7L)
  expect_identical(l$note, "n")
  expect_null(l$alpha)
  expect_null(l[["alpha"]])
  expect_identical(utils::.DollarNames(l), c("id", "columns", ".ptr", "note"))
})

test_that("S3: `[[` refuses an unused argument before it asks the fallback (#2174)", {
  x <- new_fallback_rec(7L)
  e <- caught(x[["alpha", exatc = TRUE]])
  expect_s3_class(e, "miniextendr_error_unused_argument")
  expect_identical(e$suggestion, "exact")
  expect_equal(conditionCall(e), quote(x[["alpha", exatc = TRUE]]))
  expect_identical(x[["alpha", exact = FALSE]], c(1, 2))
})

# endregion

# region: S3, a list's lookup and a scalar fallback

test_that("lookup = \"list\": the fallback's `None` reads NULL; its `Some(None)` of a scalar reads NA", {
  l <- new_fallback_list(4L)
  expect_identical(l$level, 4L)
  expect_identical(l$double, 8L)
  expect_identical(l[["double"]], 8L)
  expect_identical(l$unset, NA_integer_)
  expect_identical(l[["unset"]], NA_integer_)
  expect_null(l$nope)
  expect_null(l[["nope"]])
  # `NA` reads NULL without asking the fallback, as without one.
  expect_null(l[[NA_character_]])
  # No prefix match: `doub` is no name.
  expect_null(l$doub)
  expect_identical(l[[1]], 4L)
  expect_s3_class(caught(l[[2]]), "subscriptOutOfBoundsError")
  expect_identical(utils::.DollarNames(l), c("level", "double", "unset"))
})

test_that("lookup = \"list\": without `every_read`, a restored object's fallback refuses, its names answer", {
  back <- reload(new_fallback_list(4L))
  for (read in alist(back$double, back[["unset"]], back$nope, back$level)) {
    e <- caught(eval(read))
    expect_s3_class(e, "miniextendr_restored_no_value")
    expect_equal(conditionCall(e), read, label = deparse(read))
  }
  # The names need no Rust value: the fields, and no fallback names.
  expect_identical(names(back), "level")
  expect_identical(utils::.DollarNames(back), "level")
})

# endregion

# region: S4

test_that("S4: `$` reads the fallback's value; its `None` raises, a scalar is never NA", {
  o <- FallbackS4(3L)
  expect_identical(o$depth, 3L)
  expect_identical(o$twice, 6L)
  e <- caught(o$nope)
  expect_s3_class(e, "miniextendr_no_field")
  expect_identical(
    conditionMessage(e),
    "`nope` is not a field of `FallbackS4`; its fields are `depth`"
  )
  expect_equal(conditionCall(e), quote(o$nope))
  expect_s3_class(caught(o$twic), "miniextendr_no_field")
})

test_that("S4: an assignment to a fallback name is refused; completion adds the fallback's names", {
  o <- FallbackS4(3L)
  e <- caught(o$twice <- 1L)
  expect_s3_class(e, "miniextendr_no_field")
  expect_equal(conditionCall(e), quote(x$twice <- value))
  o$depth <- 5L
  expect_identical(o$twice, 10L)
  expect_identical(utils::.DollarNames(o), c("depth", "twice"))
  expect_identical(utils::.DollarNames(o, "^t"), "twice")
})

test_that("S4: a restored object is refused before the fallback", {
  back <- reload(FallbackS4(3L))
  e <- caught(back$twice)
  expect_s3_class(e, "miniextendr_restored_no_value")
  expect_equal(conditionCall(e), quote(back$twice))
  expect_identical(utils::.DollarNames(back), "depth")
})

# endregion

# region: env

test_that("env: `$` and `[[` read a field, then a method, then the fallback; its `None` reads NULL", {
  v <- FallbackEnv$new("noun")
  expect_identical(v$greeting, "hello")
  expect_identical(v[["greeting"]], "hello")
  # The field and the method hide the dictionary's words of the same name.
  expect_identical(v$kind, "noun")
  expect_true(is.function(v$describe))
  expect_identical(v$describe(), "noun with 3 words")
  expect_null(v$nope)
  expect_null(v$greet)
})

test_that("env: an assignment to a fallback name is refused; a field's is not", {
  v <- FallbackEnv$new("noun")
  e <- caught(v$greeting <- "hi")
  expect_s3_class(e, "miniextendr_no_field")
  expect_equal(conditionCall(e), quote(x$greeting <- value))
  v$kind <- "verb"
  expect_identical(v$kind, "verb")
  expect_identical(v$greeting, "hello")
})

test_that("env: .DollarNames() offers the fields, the methods, then the fallback's names, each once", {
  v <- FallbackEnv$new("noun")
  completions <- utils::.DollarNames(v)
  expect_identical(completions[1], "kind")
  expect_identical(completions[length(completions)], "greeting")
  expect_true(all(c("describe", "hold") %in% completions))
  expect_identical(anyDuplicated(completions), 0L)
  expect_identical(utils::.DollarNames(v, "^gr"), "greeting")
})

test_that("env: the fallback borrows the value, so a running method holding it mutably refuses it", {
  v <- FallbackEnv$new("noun")
  e <- caught(v$hold(function() v$greeting))
  expect_s3_class(e, "miniextendr_borrow_conflict")
  expect_identical(v$greeting, "hello")
})

test_that("env: a restored object is refused before the fallback; completion lists no fallback names", {
  back <- reload(FallbackEnv$new("noun"))
  for (read in alist(back$greeting, back$nope)) {
    e <- caught(eval(read))
    expect_s3_class(e, "miniextendr_restored_no_value")
  }
  expect_false("greeting" %in% utils::.DollarNames(back))
  expect_true("kind" %in% utils::.DollarNames(back))
})

# endregion
