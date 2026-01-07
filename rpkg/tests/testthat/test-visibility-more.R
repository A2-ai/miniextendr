test_that("force_invisible_i32 returns 42 invisibly", {
  res <- withVisible(force_invisible_i32())
  expect_false(res$visible)
  expect_identical(res$value, 42L)
})

test_that("force_visible_unit returns visibly", {
  res <- withVisible(force_visible_unit())
  expect_true(res$visible)
  expect_null(res$value)
})

test_that("with_interrupt_check doubles input", {
  expect_equal(with_interrupt_check(5L), 10L)
})

test_that("greet_hidden uses default and explicit names", {
  expect_equal(miniextendr:::greet_hidden(), "Hello, World!")
  expect_equal(miniextendr:::greet_hidden("Bob"), "Hello, Bob!")
})

# Result handling tests

test_that("result_null_on_err returns value on Ok", {
  expect_equal(result_null_on_err(5L), 10L)
  expect_equal(result_null_on_err(0L), 0L)
})

test_that("result_null_on_err returns NULL on Err", {
  expect_null(result_null_on_err(-1L))
  expect_null(result_null_on_err(-100L))
})

test_that("result_unwrap_in_r returns value on Ok", {
  expect_equal(result_unwrap_in_r(5L), 10L)
  expect_equal(result_unwrap_in_r(0L), 0L)
})

test_that("result_unwrap_in_r returns list(error=...) on Err", {
  res <- result_unwrap_in_r(-1L)
  expect_type(res, "list")
  expect_true("error" %in% names(res))
  expect_true(grepl("negative input", res$error))
})
