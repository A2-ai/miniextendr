test_that("rooted wrappers distinguish omitted, NULL and failed conversions", {
  missing <- miniextendr:::rooted_missing_strings
  result <- miniextendr:::rooted_result_strings
  nested <- miniextendr:::rooted_nested_values
  expect_identical(missing(), "absent")
  expect_identical(missing(c("a", NA_character_)), c("a", "NA"))
  expect_identical(missing(character()), character())
  expect_error(missing(NULL), class = "rust_error")
  expect_error(missing(1L), class = "rust_error")
  expect_identical(result(NULL), "NULL")
  expect_identical(result(c("a", NA_character_)), c("a", "NA"))
  expect_error(result(1L), class = "rust_error")
  expect_identical(nested(), -1L)
  expect_identical(nested(NULL), -2L)
  expect_identical(nested(c(1L, NA_integer_)), c(1L, NA_integer_))
  expect_error(nested(1.5), class = "rust_error")
})

test_that("rooted native wrappers still preflight aliases", {
  alias <- miniextendr:::rooted_wrapper_alias
  x <- c(1L, 2L)
  expect_error(alias(x, x), "alias")
  expect_identical(x, c(1L, 2L))
  expect_identical(alias(NULL, c(10L, 20L)), 11L)
  expect_identical(alias(other = c(10L, 20L)), 11L)
  expect_identical(alias(c(1L, 2L), c(10L, 20L)), 14L)
})

test_that("inherent and trait wrappers retain their nested roots", {
  host <- miniextendr:::RootedWrapperHost$new()
  expect_identical(host$strings(), "absent")
  expect_identical(host$strings("method"), "method")
  expect_identical(host$values(NULL), -2L)
  expect_identical(host$values(1:3), 1:3)
  expect_error(host$strings(1L), class = "rust_error")
  expect_error(host$values("wrong"), class = "rust_error")
  expect_identical(host$RootedWrapperTrait$read("trait", 1:2), c("trait", "1", "2"))
  expect_identical(host$RootedWrapperTrait$read(values = NULL), c("absent", "-2"))
  expect_error(host$RootedWrapperTrait$read(1L, NULL), class = "rust_error")
  expect_identical(
    host$GenericRootedWrapper$read_generic("generic", 1:2, 7L),
    c("generic", "1", "2", "7")
  )
})

test_that("both vtable shim generators use rooted wrapper conversions", {
  for (generic in c(FALSE, TRUE)) {
    extra <- if (generic) "7" else character()
    expect_identical(
      miniextendr:::rooted_wrapper_shim("shim", 1:2, generic),
      c("shim", "1", "2", extra)
    )
    expect_identical(
      miniextendr:::rooted_wrapper_shim(character(), NULL, generic),
      c("-2", extra)
    )
    expect_error(
      miniextendr:::rooted_wrapper_shim(1L, NULL, generic),
      class = "rust_error"
    )
    expect_error(
      miniextendr:::rooted_wrapper_shim("shim", "wrong", generic),
      class = "rust_error"
    )
  }
})

test_that("nested roots retain transient ALTSTRING elements under GC torture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  for (i in seq_len(3L)) {
    x <- miniextendr:::rooted_uncached_strings()
    expect_true(miniextendr:::rooted_wrapper_strings_survive(x, x))
    expect_null(miniextendr:::gc_stress_rooted_wrappers())
  }
})


test_that("the trait View encodes missing and unit-error arguments", {
  view <- miniextendr:::rooted_wrapper_view
  expect_identical(view(values = NULL), c("absent", "-2"))
  expect_identical(view("view", 1:2), c("view", "1", "2"))
  expect_identical(view(character(), integer()), character())
})

test_that("rooted conversion errors preserve argument fields", {
  err <- tryCatch(miniextendr:::rooted_nested_values("wrong"), error = identity)
  expect_s3_class(err, "rust_error")
  expect_identical(err$kind, "conversion")
  expect_identical(err$param, "values")
  err <- tryCatch(miniextendr:::rooted_wrapper_shim(1L, NULL, TRUE), error = identity)
  expect_s3_class(err, "rust_error")
  expect_identical(err$kind, "conversion")
  expect_identical(err$param, "strings")
})
