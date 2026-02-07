# SimpleCounter tests
#
# Tests for the Counter trait implementation and trait dispatch within
# the producer package.

# =============================================================================
# SimpleCounter inherent methods
# =============================================================================

test_that("SimpleCounter inherent get_value works", {
  counter <- new_counter(42L)
  # get_value is the inherent method (not trait dispatch)
  # new_counter wraps via mx_wrap, but the object is still a SimpleCounter
  expect_equal(counter_get_value(counter), 42L)
})

# =============================================================================
# Counter trait dispatch (within same package)
# =============================================================================

test_that("counter_get_value uses trait dispatch correctly", {
  counter <- new_counter(0L)
  expect_equal(counter_get_value(counter), 0L)

  counter <- new_counter(-100L)
  expect_equal(counter_get_value(counter), -100L)

  counter <- new_counter(2147483647L) # i32::MAX
  expect_equal(counter_get_value(counter), 2147483647L)
})

test_that("multiple counters are independent", {
  c1 <- new_counter(10L)
  c2 <- new_counter(20L)
  c3 <- new_counter(30L)

  expect_equal(counter_get_value(c1), 10L)
  expect_equal(counter_get_value(c2), 20L)
  expect_equal(counter_get_value(c3), 30L)
})

# =============================================================================
# Debug/utility functions
# =============================================================================

test_that("debug_tag_counter returns consistent 32-char hex tag", {
  tag1 <- debug_tag_counter()
  tag2 <- debug_tag_counter()

  expect_equal(tag1, tag2)
  expect_equal(nchar(tag1), 32L)
  expect_match(tag1, "^[0-9a-f]{32}$")
})

test_that("debug_shared_data_type_name returns a non-empty string", {
  name <- debug_shared_data_type_name()
  expect_type(name, "character")
  expect_true(nchar(name) > 0L)
  expect_match(name, "SharedData")
})

test_that("get_r_class returns correct classes", {
  p <- EnvPoint$new(1, 2)
  cls <- get_r_class(p)
  expect_true("EnvPoint" %in% cls)

  p3 <- new_s3point(1, 2)
  cls3 <- get_r_class(p3)
  expect_true("S3Point" %in% cls3)

  # Non-classed object returns NULL
  cls_int <- get_r_class(42L)
  expect_null(cls_int)
})
