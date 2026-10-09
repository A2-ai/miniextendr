# A `List`, `Option<List>` or `NamedList` argument refuses a pairlist (#1866).
# It used to coerce one to a list, a new object that nothing rooted, so an
# allocation in the function could free it. R's `is.list()` is TRUE for a
# pairlist, so the R guard lets it through and the Rust conversion refuses it,
# with a message that says to use `as.list()`. Fixtures in
# `src/rust/list_pairlist_tests.rs`.

catch_error <- function(expr) tryCatch(expr, error = function(e) e)
pl <- pairlist(a = 1L, b = 2L, c = 3L)

expect_pairlist_refused <- function(e, expected, rust_type) {
  expect_s3_class(e, "rust_error")
  expect_identical(
    conditionMessage(e),
    sprintf("'x' must be %s: got pairlist, convert it with as.list()", expected)
  )
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "x")
  expect_identical(e$rust_type, rust_type)
}

test_that("a pairlist is refused with a type error that names as.list()", {
  e <- catch_error(miniextendr:::list_read_after_alloc(pl))
  expect_pairlist_refused(e, "a list", "List")
  expect_identical(conditionCall(e)[[1L]], quote(miniextendr:::list_read_after_alloc))

  expect_pairlist_refused(
    catch_error(miniextendr:::option_list_read_after_alloc(pl)),
    "NULL or a list",
    "Option<List>"
  )
  expect_pairlist_refused(
    catch_error(miniextendr:::named_list_read_after_alloc(pl)),
    "a list",
    "NamedList"
  )
  # Formals are a pairlist too.
  expect_pairlist_refused(
    catch_error(miniextendr:::list_read_after_alloc(formals(function(a, b) NULL))),
    "a list",
    "List"
  )
})

test_that("as.list() of the pairlist converts", {
  expect_identical(miniextendr:::list_read_after_alloc(as.list(pl)), 1:3)
  expect_identical(miniextendr:::option_list_read_after_alloc(as.list(pl)), 1:3)
  expect_identical(miniextendr:::named_list_read_after_alloc(as.list(pl)), 1:3)
  expect_identical(miniextendr:::option_list_read_after_alloc(NULL), integer(0))
})

test_that("a pairlist element of a list of lists gets the same hint", {
  e <- catch_error(miniextendr:::list_newtype_lengths(list(list(), pl)))
  expect_identical(
    conditionMessage(e),
    "invalid 'x' argument: expected list, got pairlist, convert it with as.list() (element 2)"
  )
})

test_that("a typed_list! list() field refuses a pairlist and still takes NULL", {
  e <- catch_error(miniextendr:::validate_numeric_args(alpha = c(1, 2, 3, 4), beta = pl))
  expect_match(conditionMessage(e), 'field "beta" has wrong type: expected list, got pairlist', fixed = TRUE)
  expect_identical(miniextendr:::validate_numeric_args(alpha = c(1, 2, 3, 4), beta = NULL), 4L)
})

test_that("the pairlist fixture refuses and the plain list reads back", {
  expect_null(miniextendr:::gc_stress_list_pairlist())
})

test_that("a list argument reads back whole under gctorture, and a pairlist is refused", {
  skip_gc_stress_if_disabled()
  x <- list(a = 1L, b = 2L, c = 3L)
  read_list <- miniextendr:::list_read_after_alloc
  read_option <- miniextendr:::option_list_read_after_alloc
  read_named <- miniextendr:::named_list_read_after_alloc
  fixture <- miniextendr:::gc_stress_list_pairlist
  # Load the package first, then enable gctorture: see docs/GCTORTURE_TESTING.md.
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)

  out <- list(
    list = read_list(x),
    option = read_option(x),
    named = read_named(x),
    refused = tryCatch(read_list(pl), error = conditionMessage)
  )
  fixture()
  gctorture(FALSE)

  expect_identical(out$list, 1:3)
  expect_identical(out$option, 1:3)
  expect_identical(out$named, 1:3)
  expect_identical(out$refused, "'x' must be a list: got pairlist, convert it with as.list()")
})
