# Cross-Package Tests for consumer.pkg
#
# These tests verify cross-package trait dispatch between producer.pkg and consumer.pkg.
# The Counter trait is defined in both packages with identical signatures, enabling
# consumer.pkg to call trait methods on objects created by producer.pkg.

test_that("consumer.pkg functions work standalone", {
  # Test consumer_magic_number
  expect_equal(consumer_magic_number(), 42L)

  # Test consumer_greet
  greeting <- consumer_greet("World")
  expect_equal(greeting, "Hello World from consumer.pkg!")
})

test_that("producer.pkg Counter can be used via consumer.pkg trait dispatch", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  # Create a Counter using mx_erased wrapper (required for cross-package dispatch)
  counter <- new_counter(10L)

  # Use consumer's trait dispatch to check if it's a Counter
  expect_true(is_counter(counter))

  # Use consumer's increment_twice via trait dispatch
  result <- increment_twice(counter)
  expect_equal(result, 12L)

  # Counter state persists - use trait dispatch to check value
  expect_equal(peek_value(counter), 12L)
})

test_that("peek_value works via trait dispatch", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(5L)

  # peek_value should return the value without modifying
  expect_equal(peek_value(counter), 5L)
  expect_equal(peek_value(counter), 5L)

  # Counter state unchanged
  expect_equal(peek_value(counter), 5L)
})

test_that("add_and_get works via trait dispatch", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(100L)

  # add_and_get should add and return new value
  result <- add_and_get(counter, 50L)
  expect_equal(result, 150L)

  # Counter state updated
  expect_equal(peek_value(counter), 150L)
})

test_that("trait dispatch works after multiple operations", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(0L)

  # Mix producer methods and consumer trait dispatch
  expect_equal(peek_value(counter), 0L)

  # Use consumer's increment_twice (increments by 1 each time for SimpleCounter)
  increment_twice(counter)
  expect_equal(peek_value(counter), 2L)

  # Use consumer's add_and_get
  add_and_get(counter, 8L)
  expect_equal(peek_value(counter), 10L)
})

test_that("both packages can be loaded together", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  # Verify both packages are functional simultaneously
  expect_equal(consumer_magic_number(), 42L)

  counter <- new_counter(100L)
  expect_equal(peek_value(counter), 100L)
  expect_true(is_counter(counter))
})

test_that("bidirectional trait dispatch: consumer's DoubleCounter works with producer", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  # Create consumer's DoubleCounter (increments by 2)
  double_counter <- new_double_counter(10L)

  # Should be recognized as Counter
  expect_true(is_counter(double_counter))

  # Producer's counter_get_value can read consumer's DoubleCounter
  expect_equal(counter_get_value(double_counter), 10L)

  # DoubleCounter increments by 2, so increment_twice adds 4
  result <- increment_twice(double_counter)
  expect_equal(result, 14L)  # 10 + 2 + 2 = 14
})

test_that("SimpleCounter and DoubleCounter behave differently", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  # SimpleCounter increments by 1
  simple <- new_counter(0L)
  result_simple <- increment_twice(simple)
  expect_equal(result_simple, 2L)  # 0 + 1 + 1 = 2

  # DoubleCounter increments by 2
  double <- new_double_counter(0L)
  result_double <- increment_twice(double)
  expect_equal(result_double, 4L)  # 0 + 2 + 2 = 4
})

# =============================================================================
# Cross-Package ExternalPtr Pass-Through Tests
# =============================================================================
# These tests verify that ExternalPtr objects can be passed through consumer.pkg
# without consumer knowing the type - true opaque cross-package ExternalPtr usage.

test_that("ExternalPtr can pass through consumer opaquely", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  # Create SharedData in producer.pkg
  data <- SharedData$create(3.0, 4.0, "test point")

  # Consumer checks it's an ExternalPtr (type-agnostic)
  expect_true(is_external_ptr(data))

  # Consumer passes it through without knowing the type
  returned <- passthrough_ptr(data)

  # Producer can still use it after round-trip through consumer
  expect_equal(data$get_x(), 3.0)
  expect_equal(returned$get_x(), 3.0)
  expect_equal(data$get_label(), "test point")
})

test_that("is_external_ptr works correctly", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  # ExternalPtr types
  data <- SharedData$create(1.0, 2.0, "test")
  counter <- new_counter(5L)

  expect_true(is_external_ptr(data))
  expect_true(is_external_ptr(counter))

  # Non-ExternalPtr types
  expect_false(is_external_ptr(42L))
  expect_false(is_external_ptr("hello"))
  expect_false(is_external_ptr(list(a = 1)))
})
