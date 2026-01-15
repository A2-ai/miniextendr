# Sparse Iterator ALTREP tests
#
# These tests verify the sparse iterator ALTREP types that use Iterator::nth()
# to skip elements efficiently. Unlike prefix-caching iterators, sparse
# iterators only cache accessed elements and return NA for skipped elements.

# =============================================================================
# Sparse Integer Iterator tests
# =============================================================================

test_that("sparse integer iterator works for basic access", {
  x <- sparse_iter_int(1L, 11L)

  expect_equal(length(x), 10L)
  expect_equal(x[1], 1L)
  expect_equal(x[2], 2L)
  expect_equal(x[10], 10L)
})

test_that("sparse integer iterator skips elements correctly", {
  x <- sparse_iter_int(1L, 101L)

  # Access element 50 first - this skips elements 1-49
  expect_equal(x[50], 50L)

  # Elements 1-49 were skipped and should now return NA
  expect_true(is.na(x[1]))
  expect_true(is.na(x[25]))
  expect_true(is.na(x[49]))

 # Element 50 was cached, should still work
  expect_equal(x[50], 50L)

  # Elements 51+ should still work (not yet accessed)
  expect_equal(x[51], 51L)
  expect_equal(x[100], 100L)
})

test_that("sparse integer iterator works with sequential access", {
  x <- sparse_iter_int(0L, 5L)

  # Access sequentially - should all work
  expect_equal(x[1], 0L)
  expect_equal(x[2], 1L)
  expect_equal(x[3], 2L)
  expect_equal(x[4], 3L)
  expect_equal(x[5], 4L)
})

test_that("sparse integer iterator sum includes NAs for skipped elements", {
  x <- sparse_iter_int(1L, 6L)  # Elements: 1, 2, 3, 4, 5

  # Access only element 5 first (skips 1-4)
  val5 <- x[5]
  expect_equal(val5, 5L)

  # Sum should be NA because elements 1-4 are NA
  expect_true(is.na(sum(x)))

  # Sum with na.rm should give just the cached elements
  # Only element 5 is cached at this point
  expect_equal(sum(x, na.rm = TRUE), 5L)
})

test_that("sparse integer iterator squares works", {
  x <- sparse_iter_int_squares(5L)

  expect_equal(length(x), 5L)
  expect_equal(x[1], 0L)   # 0^2
  expect_equal(x[2], 1L)   # 1^2
  expect_equal(x[3], 4L)   # 2^2
  expect_equal(x[4], 9L)   # 3^2
  expect_equal(x[5], 16L)  # 4^2
})

# =============================================================================
# Sparse Real Iterator tests
# =============================================================================

test_that("sparse real iterator works for basic access", {
  x <- sparse_iter_real(0, 1, 5L)  # 0, 1, 2, 3, 4

  expect_equal(length(x), 5L)
  expect_equal(x[1], 0)
  expect_equal(x[2], 1)
  expect_equal(x[5], 4)
})

test_that("sparse real iterator skips elements correctly", {
  x <- sparse_iter_real(0, 0.5, 100L)

  # Access element 50 first (value = 0 + 49 * 0.5 = 24.5)
  expect_equal(x[50], 24.5)

  # Elements 1-49 were skipped and should return NaN (NA for reals)
  expect_true(is.na(x[1]))
  expect_true(is.na(x[25]))

  # Element 50 was cached
  expect_equal(x[50], 24.5)

  # Elements 51+ should still work
  expect_equal(x[51], 25)
})

# =============================================================================
# Sparse Logical Iterator tests
# =============================================================================

test_that("sparse logical iterator works for basic access", {
  x <- sparse_iter_logical(6L)  # TRUE, FALSE, TRUE, FALSE, TRUE, FALSE

  expect_equal(length(x), 6L)
  expect_true(x[1])   # 0 % 2 == 0 -> TRUE
  expect_false(x[2])  # 1 % 2 == 0 -> FALSE
  expect_true(x[3])
  expect_false(x[4])
})

test_that("sparse logical iterator skips elements correctly", {
  x <- sparse_iter_logical(10L)

  # Access element 8 first (value = (7 % 2 == 0) = FALSE)
  expect_false(x[8])

  # Elements 1-7 were skipped and should return NA
  expect_true(is.na(x[1]))
  expect_true(is.na(x[5]))

  # Element 8 was cached
  expect_false(x[8])

  # Elements 9-10 should still work
  expect_true(x[9])   # 8 % 2 == 0 -> TRUE
  expect_false(x[10]) # 9 % 2 == 0 -> FALSE
})

# =============================================================================
# Sparse Raw Iterator tests
# =============================================================================

test_that("sparse raw iterator works for basic access", {
  x <- sparse_iter_raw(10L)

  expect_equal(length(x), 10L)
  expect_equal(x[1], as.raw(0))
  expect_equal(x[2], as.raw(1))
  expect_equal(x[10], as.raw(9))
})

test_that("sparse raw iterator skips elements correctly", {
  x <- sparse_iter_raw(20L)

  # Access element 15 first
  expect_equal(x[15], as.raw(14))

  # Elements 1-14 were skipped and should return 0 (raw doesn't have NA)
  expect_equal(x[1], as.raw(0))  # Note: raw(0) is the default, same as actual value here

  # Let's use a different test with clearly different values
  y <- sparse_iter_raw(300L)

  # Access element 260 first (value = 259 % 256 = 3)
  expect_equal(y[260], as.raw(3))

  # Element 100 was skipped - returns 0
  expect_equal(y[100], as.raw(0))

  # Element 260 was cached
  expect_equal(y[260], as.raw(3))
})

# =============================================================================
# Edge cases
# =============================================================================

test_that("sparse iterators handle zero-length vectors", {
  empty_int <- sparse_iter_int(5L, 5L)  # Empty range
  expect_equal(length(empty_int), 0L)

  empty_real <- sparse_iter_real(0, 1, 0L)
  expect_equal(length(empty_real), 0L)

  empty_lgl <- sparse_iter_logical(0L)
  expect_equal(length(empty_lgl), 0L)

  empty_raw <- sparse_iter_raw(0L)
  expect_equal(length(empty_raw), 0L)
})

test_that("sparse iterators handle single-element vectors", {
  single_int <- sparse_iter_int(42L, 43L)
  expect_equal(length(single_int), 1L)
  expect_equal(single_int[1], 42L)

  single_real <- sparse_iter_real(3.14, 1, 1L)
  expect_equal(length(single_real), 1L)
  expect_equal(single_real[1], 3.14)
})

test_that("sparse iterators work with large vectors", {
  # Create a large sparse iterator
  big <- sparse_iter_int(1L, 1000001L)
  expect_equal(length(big), 1000000L)

  # Access only the last element - should skip directly there
  expect_equal(big[1000000], 1000000L)

  # First element was skipped
  expect_true(is.na(big[1]))
})

# =============================================================================
# Comparison with regular iterators
# =============================================================================

test_that("sparse vs regular iterator behavior differs for skipped elements", {
  # Regular iterator caches prefix
  regular <- iter_int_range(1L, 101L)

  # Access element 50
  val50_reg <- regular[50]
  expect_equal(val50_reg, 50L)

  # Element 1 should still be accessible (cached in prefix)
  expect_equal(regular[1], 1L)

  # Sparse iterator skips
  sparse <- sparse_iter_int(1L, 101L)

  # Access element 50
  val50_sparse <- sparse[50]
  expect_equal(val50_sparse, 50L)

  # Element 1 was skipped and returns NA
  expect_true(is.na(sparse[1]))
})
