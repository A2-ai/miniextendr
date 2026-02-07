test_that("greet() uses default and accepts explicit name", {
  expect_equal(greet(), "Hello, World!")
  expect_equal(greet("Alice"), "Hello, Alice!")
})

test_that("add_with_defaults() applies missing args", {
  expect_equal(add_with_defaults(1L), 2L)          # 1 + 0 + 1
  expect_equal(add_with_defaults(1L, 2L), 4L)      # 1 + 2 + 1
  expect_equal(add_with_defaults(1L, 2L, 3L), 6L)
})

test_that("with_flag() defaults to FALSE and passes TRUE", {
  expect_false(with_flag())
  expect_true(with_flag(TRUE))
})

test_that("underscore_it_all() accepts discarded parameters", {
  expect_null(underscore_it_all(1L, 2))
})

test_that("do_nothing() returns raw SEXP integer directly", {
  expect_identical(do_nothing(), 42L)
})
