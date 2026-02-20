# Multi-trait cross-package tests
#
# Tests for multiple trait dispatch on the same type across packages.
# Verifies that types implementing BOTH Counter and Resettable can
# be dispatched correctly from consumer.pkg using objects from producer.pkg.

# =============================================================================
# Resettable trait dispatch on SimpleCounter (from producer.pkg)
# =============================================================================

test_that("SimpleCounter implements Resettable (cross-package)", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(42L)

  # Should be recognized as Resettable
  expect_true(is_resettable(counter))

  # Not in default state (value is 42, not 0)
  expect_false(check_is_default(counter))

  # Reset and verify it's now in default state
  expect_true(reset_and_check(counter))

  # Value should be 0 after reset
  expect_equal(peek_value(counter), 0L)
})

test_that("SimpleCounter with initial 0 is in default state", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(0L)
  expect_true(check_is_default(counter))
})

# =============================================================================
# Resettable trait dispatch on StatefulCounter (from producer.pkg)
# =============================================================================

test_that("StatefulCounter implements Resettable (cross-package)", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_stateful_counter(100L)

  # Should be recognized as both Counter and Resettable
  expect_true(is_counter(counter))
  expect_true(is_resettable(counter))

  # Not in default state
  expect_false(check_is_default(counter))

  # Reset and check
  expect_true(reset_and_check(counter))
  expect_equal(peek_value(counter), 0L)
})

test_that("StatefulCounter default state considers history", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  # StatefulCounter(0) starts with history = [0], which IS default
  counter <- new_stateful_counter(0L)
  expect_true(check_is_default(counter))

  # After increment, history grows, no longer default even if value returns to 0
  increment_twice(counter)  # value = 2, history = [0, 1, 2]
  expect_false(check_is_default(counter))

  # Reset clears history back to [0] and sets value = 0
  expect_true(reset_and_check(counter))
})

# =============================================================================
# Combined trait usage: Counter + Resettable on same object
# =============================================================================

test_that("increment_then_reset works on SimpleCounter", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(0L)

  # increment twice, then reset - should be in default state
  expect_true(increment_then_reset(counter))

  # Value should be 0 after reset
  expect_equal(peek_value(counter), 0L)
})

test_that("increment_then_reset works on StatefulCounter", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_stateful_counter(0L)

  # increment twice, then reset - should be in default state
  expect_true(increment_then_reset(counter))
  expect_equal(peek_value(counter), 0L)
})

test_that("get_reset_get returns 0 for SimpleCounter", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(999L)
  result <- get_reset_get(counter)
  expect_equal(result, 0L)
})

test_that("get_reset_get returns 0 for StatefulCounter", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_stateful_counter(42L)
  result <- get_reset_get(counter)
  expect_equal(result, 0L)
})

test_that("combined trait usage: increment, peek, reset, peek cycle", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(0L)

  # Increment via Counter
  increment_twice(counter)
  expect_equal(peek_value(counter), 2L)

  # Reset via Resettable
  expect_true(reset_and_check(counter))
  expect_equal(peek_value(counter), 0L)

  # Increment again after reset
  add_and_get(counter, 10L)
  expect_equal(peek_value(counter), 10L)
  expect_false(check_is_default(counter))

  # Reset again
  expect_true(reset_and_check(counter))
  expect_equal(peek_value(counter), 0L)
})

# =============================================================================
# Type discrimination: is_resettable rejects non-Resettable types
# =============================================================================

test_that("is_resettable rejects non-Resettable types", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  # DoubleCounter (consumer-side) does NOT implement Resettable
  dc <- new_double_counter(10L)
  expect_true(is_counter(dc))
  expect_false(is_resettable(dc))

  # Non-ExternalPtr types
  expect_false(is_resettable(42L))
  expect_false(is_resettable("hello"))
  expect_false(is_resettable(NULL))
  expect_false(is_resettable(list()))

  # Non-trait ExternalPtrs
  data <- SharedData$create(0, 0, "x")
  expect_false(is_resettable(data))
})

test_that("is_counter and is_resettable both TRUE for dual-trait types", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  simple <- new_counter(0L)
  expect_true(is_counter(simple))
  expect_true(is_resettable(simple))

  stateful <- new_stateful_counter(0L)
  expect_true(is_counter(stateful))
  expect_true(is_resettable(stateful))
})

# =============================================================================
# TAG_RESETTABLE consistency across packages
# =============================================================================

test_that("TAG_RESETTABLE is identical across producer and consumer", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  producer_tag <- debug_tag_resettable()
  consumer_tag <- debug_consumer_tag_resettable()

  expect_equal(producer_tag, consumer_tag)
  expect_equal(nchar(producer_tag), 32L)
})

test_that("TAG_RESETTABLE differs from TAG_COUNTER", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter_tag <- debug_consumer_tag_counter()
  resettable_tag <- debug_consumer_tag_resettable()

  expect_false(counter_tag == resettable_tag)
})

# =============================================================================
# StatefulCounter inherent methods
# =============================================================================

test_that("StatefulCounter Counter trait works via consumer dispatch", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_stateful_counter(5L)

  # Counter trait dispatch
  expect_equal(peek_value(counter), 5L)

  result <- increment_twice(counter)
  expect_equal(result, 7L)  # 5 + 1 + 1

  result <- add_and_get(counter, 3L)
  expect_equal(result, 10L)
})

# =============================================================================
# Stress: repeated reset cycles
# =============================================================================

test_that("repeated increment-reset cycles are consistent", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(0L)

  for (i in seq_len(20)) {
    add_and_get(counter, 10L)
    expect_equal(peek_value(counter), 10L)
    expect_false(check_is_default(counter))

    reset_and_check(counter)
    expect_equal(peek_value(counter), 0L)
    expect_true(check_is_default(counter))
  }
})

test_that("StatefulCounter history resets correctly across cycles", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_stateful_counter(0L)

  for (i in seq_len(10)) {
    # Increment
    increment_twice(counter)
    expect_false(check_is_default(counter))

    # Reset
    expect_true(reset_and_check(counter))
    expect_true(check_is_default(counter))
    expect_equal(peek_value(counter), 0L)
  }
})
