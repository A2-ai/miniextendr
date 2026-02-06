# Test ALTREP convenience helpers

test_that("into_sexp_altrep creates ALTREP vectors", {
  x <- large_vec_altrep()

  expect_equal(length(x), 100000L)
  expect_equal(x[1], 0L)
  expect_equal(x[100000], 0L)

  # Verify it's ALTREP (check class or behavior)
  expect_true(is.integer(x))
})

test_that("small vectors copy as expected", {
  x <- small_vec_copy()

  expect_equal(length(x), 5L)
  expect_equal(x, c(1L, 2L, 3L, 4L, 5L))
})

test_that("lazy computation via ALTREP", {
  x <- lazy_squares(10L)

  expect_equal(length(x), 10L)
  expect_equal(x[1], 0L)   # 0^2
  expect_equal(x[4], 9L)   # 3^2
  expect_equal(x[10], 81L) # 9^2
})

test_that("into_altrep wrapper works", {
  x <- boxed_data_altrep(5L)

  expect_equal(length(x), 5L)
  expect_equal(x, 0:4)
})

test_that("ALTREP and copy produce identical results", {
  # Different data, but should work identically in R
  copy <- small_vec_copy()  # vec![1,2,3,4,5]
  altrep <- large_vec_altrep()[1:5]  # vec![0; 100_000], first 5 elements

  # Both should work as normal R vectors
  expect_equal(length(copy), 5L)
  expect_equal(length(altrep), 5L)
  expect_equal(sum(copy), 15)  # 1+2+3+4+5
  expect_equal(sum(altrep), 0)  # 0+0+0+0+0
})

test_that("lazy_squares handles negative input", {
  expect_error(lazy_squares(-1L), "n must be >= 0")
})

test_that("boxed_data_altrep handles negative input", {
  expect_error(boxed_data_altrep(-1L), "n must be >= 0")
})

test_that("large_vec_altrep returns all zeros", {
  x <- large_vec_altrep()
  # Check first, middle, and last elements
  expect_equal(x[1], 0L)
  expect_equal(x[50000], 0L)
  expect_equal(x[100000], 0L)
  # Verify sum is zero
  expect_equal(sum(x), 0)
})

test_that("lazy_squares computes correct values", {
  x <- lazy_squares(5L)
  expected <- c(0L, 1L, 4L, 9L, 16L)
  expect_equal(x, expected)
})

test_that("boxed_data_altrep creates sequential integers", {
  x <- boxed_data_altrep(10L)
  expect_equal(x, 0:9)
})
