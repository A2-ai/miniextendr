test_that("add() works correctly", {
  expect_equal(add(2L, 3L), 5L)
  expect_equal(add(0L, 0L), 0L)
  expect_equal(add(-1L, 1L), 0L)
  expect_equal(add(100L, 200L), 300L)
})

test_that("add2() works with dummy argument", {
  expect_equal(add2(2L, 3L, NULL), 5L)
  expect_equal(add2(10L, 20L, NULL), 30L)
})

test_that("add3() adds integers with a dummy argument", {
  expect_equal(add3(2L, 3L, NULL), 5L)
  expect_equal(add3(10L, 20L, NULL), 30L)
})

test_that("add4() divides and errors on zero divisor", {
  expect_equal(add4(10L, 2L), 5L)
  expect_equal(add4(100L, 10L), 10L)
  # Division by zero should error

  expect_error(add4(10L, 0L), "don't divide by zero")
})

test_that("take_and_return_nothing() returns invisibly", {
  result <- take_and_return_nothing()
  expect_null(result)
})

test_that("invisibly_return_no_arrow() returns NULL invisibly", {
  result <- invisibly_return_no_arrow()
  expect_null(result)
})

test_that("invisibly_return_arrow() returns NULL invisibly", {
  result <- invisibly_return_arrow()
  expect_null(result)
})

test_that("invisibly_option_return_some() returns NULL invisibly", {
  result <- invisibly_option_return_some()
  expect_null(result)
})

test_that("invisibly_option_return_none() errors when no value is available", {
  expect_error(invisibly_option_return_none(), "returned None")
})

test_that("invisibly_result_return_ok() returns NULL invisibly", {
  result <- invisibly_result_return_ok()
  expect_null(result)
})

test_that("add variants with different argument styles work", {
  expect_equal(add_left_mut(2L, 3L), 5L)
  expect_equal(add_right_mut(2L, 3L), 5L)
  expect_equal(add_left_right_mut(2L, 3L), 5L)
})
