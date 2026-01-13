# Tests for rayon parallel computation feature

# Skip all tests if rayon feature is not enabled
skip_if_missing_feature("rayon")

test_that("rayon_parallel_sum computes correct sum", {
  x <- as.numeric(1:1000)
  result <- rayon_parallel_sum(x)
  expect_equal(result, sum(x))
})
test_that("rayon_parallel_sum handles empty vector", {
  result <- rayon_parallel_sum(numeric(0))
  expect_equal(result, 0)
})

test_that("rayon_parallel_sqrt computes element-wise sqrt", {
  x <- c(1, 4, 9, 16, 25)
  result <- rayon_parallel_sqrt(x)
  expect_equal(result, sqrt(x))
})

test_that("rayon_parallel_sqrt handles large vector", {
  x <- as.numeric(1:10000)
  result <- rayon_parallel_sqrt(x)
  expect_equal(result, sqrt(x))
})

test_that("rayon_parallel_filter_positive filters correctly", {
  x <- c(-2, -1, 0, 1, 2, 3)
  result <- rayon_parallel_filter_positive(x)
  expect_equal(sort(result), c(1, 2, 3))
})

test_that("rayon_parallel_filter_positive returns empty for all negative", {
  x <- c(-3, -2, -1)
  result <- rayon_parallel_filter_positive(x)
  expect_length(result, 0)
})

test_that("rayon_vec_collect returns correct length and values", {
  result <- rayon_vec_collect(100L)
  expect_length(result, 100)
  expect_equal(result[1], 0)
  expect_equal(result[2], 1)
  expect_equal(result[5], 2) # sqrt(4) = 2
})

test_that("rayon_with_r_vec returns correct R vector", {
  result <- rayon_with_r_vec(100L)
  expect_type(result, "double")
  expect_length(result, 100)
  expect_equal(result[1], 0)
  expect_equal(result[2], 1)
  expect_equal(result[5], 2) # sqrt(4) = 2
})

test_that("rayon_with_r_matrix returns correct matrix", {
  result <- rayon_with_r_matrix(3L, 4L)
  expect_true(is.matrix(result))
  expect_equal(dim(result), c(3, 4))
  # Value at [row, col] should be (row-1) * (col-1) (0-indexed in Rust)
  expect_equal(result[1, 1], 0) # 0 * 0 = 0
  expect_equal(result[2, 2], 1) # 1 * 1 = 1
  expect_equal(result[3, 3], 4) # 2 * 2 = 4
  expect_equal(result[3, 4], 6) # 2 * 3 = 6
})

test_that("rayon_parallel_stats returns correct statistics", {
  x <- c(1, 2, 3, 4, 5)
  result <- rayon_parallel_stats(x)
  expect_length(result, 4)
  expect_equal(result[1], sum(x))    # sum = 15

  expect_equal(result[2], min(x))    # min = 1
  expect_equal(result[3], max(x))    # max = 5
  expect_equal(result[4], mean(x))   # mean = 3
})

test_that("rayon_parallel_stats handles single element", {
  x <- c(42)
  result <- rayon_parallel_stats(x)
  expect_equal(result[1], 42)  # sum
  expect_equal(result[2], 42)  # min
  expect_equal(result[3], 42)  # max
  expect_equal(result[4], 42)  # mean
})

test_that("rayon_parallel_sum_int computes integer sum", {
  x <- 1:100L
  result <- rayon_parallel_sum_int(x)
  expect_equal(result, sum(x))
})

test_that("rayon_num_threads returns positive number", {
  result <- rayon_num_threads()
  expect_true(result >= 1)
})

test_that("rayon_in_thread returns FALSE when called from R", {
  # When called from R main thread, we should NOT be in a rayon thread

  result <- rayon_in_thread()
  expect_false(result)
})

test_that("rayon handles large parallel workload", {
  skip_on_cran()

  # Stress test with moderately large data (reduced for CI speed)
  n <- 100000L
  withr::local_seed(42)
  x <- runif(n)

  # Parallel reduction order can introduce floating point differences
  # Use relative tolerance based on magnitude of result
  sum_result <- rayon_parallel_sum(x)
  expected_sum <- sum(x)
  # Allow for 1e-10 relative error

  expect_equal(sum_result, expected_sum, tolerance = abs(expected_sum) * 1e-10)

  # Parallel sqrt - element-wise, so order doesn't matter
  sqrt_result <- rayon_parallel_sqrt(x)
  expect_equal(sqrt_result, sqrt(x), tolerance = 1e-14)
})
