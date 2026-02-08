# Cross-package edge case tests
#
# These tests verify correctness of cross-package trait dispatch under
# boundary conditions and stress scenarios.

# =============================================================================
# Integer boundary tests
# =============================================================================

test_that("counter handles zero initial value", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(0L)
  expect_equal(peek_value(counter), 0L)

  result <- increment_twice(counter)
  expect_equal(result, 2L)
})

test_that("counter handles negative initial value", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(-10L)
  expect_equal(peek_value(counter), -10L)

  result <- increment_twice(counter)
  expect_equal(result, -8L)
})

test_that("add_and_get handles negative values", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(100L)
  result <- add_and_get(counter, -50L)
  expect_equal(result, 50L)
})

test_that("add_and_get with zero is a no-op", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(42L)
  result <- add_and_get(counter, 0L)
  expect_equal(result, 42L)
})

# =============================================================================
# State consistency under repeated operations
# =============================================================================

test_that("counter state is consistent after many operations", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(0L)

  # Increment 100 times via trait dispatch
  for (i in seq_len(50)) {
    increment_twice(counter)
  }

  # Each increment_twice adds 2 (SimpleCounter increments by 1 each time)
  expect_equal(peek_value(counter), 100L)
})

test_that("DoubleCounter state is consistent after many operations", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  double <- new_double_counter(0L)

  # DoubleCounter increments by 2 each time, increment_twice calls it twice
  for (i in seq_len(25)) {
    increment_twice(double)
  }

  # Each increment_twice adds 4 for DoubleCounter (2+2)
  expect_equal(peek_value(double), 100L)
})

test_that("interleaved add_and_get and increment_twice are consistent", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(0L)

  for (i in seq_len(10)) {
    increment_twice(counter)   # +2
    add_and_get(counter, 3L)   # +3
  }

  # Each iteration adds 5, 10 iterations = 50
  expect_equal(peek_value(counter), 50L)
})

# =============================================================================
# Type checking edge cases
# =============================================================================

test_that("is_counter rejects non-ExternalPtr types", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  expect_false(is_counter(42L))
  expect_false(is_counter("hello"))
  expect_false(is_counter(NULL))
  expect_false(is_counter(list()))
  expect_false(is_counter(TRUE))
  expect_false(is_counter(1.5))
})

test_that("is_external_ptr rejects NULL and NA", {
  expect_false(is_external_ptr(NULL))
  expect_false(is_external_ptr(NA))
  expect_false(is_external_ptr(NA_integer_))
  expect_false(is_external_ptr(NA_real_))
  expect_false(is_external_ptr(NA_character_))
})

# =============================================================================
# i32 boundary values
# =============================================================================

test_that("counter handles near-max i32 values", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  # i32::MAX - 2 = 2147483645
  counter <- new_counter(2147483645L)
  result <- increment_twice(counter)
  expect_equal(result, 2147483647L) # i32::MAX
})

test_that("counter handles large negative values", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  # Near i32::MIN (but not NA_integer_ which is i32::MIN itself)
  counter <- new_counter(-2147483647L) # i32::MIN + 1
  expect_equal(peek_value(counter), -2147483647L)
})

test_that("add_and_get works with large values", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(0L)
  result <- add_and_get(counter, 2147483647L) # i32::MAX
  expect_equal(result, 2147483647L)
})

# =============================================================================
# DoubleCounter trait namespace methods
# =============================================================================

test_that("DoubleCounter Counter trait methods via trait namespace", {
  dc <- DoubleCounter$create(10L)

  # Trait namespace: DoubleCounter$Counter$value
  expect_equal(dc$Counter$value(dc), 10L)

  # Trait namespace: DoubleCounter$Counter$increment (adds 2)
  dc$Counter$increment(dc)
  expect_equal(dc$Counter$value(dc), 12L)

  # Trait namespace: DoubleCounter$Counter$add
  dc$Counter$add(dc, 100L)
  expect_equal(dc$Counter$value(dc), 112L)
})
