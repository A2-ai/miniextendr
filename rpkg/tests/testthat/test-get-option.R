# get_option(): reading getOption() from Rust.

test_that("an unset or NULL option reads None", {
  withr::local_options(mx.test.unset = NULL)
  # A scalar `None` returns NA, a vector's NULL.
  expect_identical(get_option_int("mx.test.unset"), NA_integer_)
  expect_null(get_option_strings("mx.test.never.set"))
})

test_that("a set option converts", {
  withr::local_options(mx.test.int = 3L, mx.test.chr = c("a", "b"))
  expect_identical(get_option_int("mx.test.int"), 3L)
  expect_identical(get_option_strings("mx.test.chr"), c("a", "b"))
})

test_that("a value that doesn't convert raises a classed error naming the option", {
  withr::local_options(mx.test.bad = "wide")
  err <- expect_error(
    get_option_int("mx.test.bad"),
    'options("mx.test.bad") is invalid: expected integer, got character',
    fixed = TRUE,
    class = "miniextendr_option_error"
  )
  expect_identical(err$option, "mx.test.bad")
})
