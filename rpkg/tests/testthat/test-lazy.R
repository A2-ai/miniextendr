# Tests for Lazy<T> ALTREP wrappers

# region: Lazy<Vec<T>>

test_that("Lazy<Vec<f64>> returns correct numeric values", {
  result <- test_lazy_vec_f64(5L)
  expect_type(result, "double")
  expect_length(result, 5)
  expect_equal(result, sin(0:4))
})
test_that("Lazy<Vec<i32>> returns ALTREP integer", {
  result <- test_lazy_vec_i32(5L)
  expect_type(result, "integer")
  expect_equal(result, c(0L, 1L, 4L, 9L, 16L))
})

# endregion

# region: Lazy<Arrow arrays>

test_that("Lazy<Float64Array> returns ALTREP numeric", {
  result <- test_lazy_arrow_f64(5L)
  expect_type(result, "double")
  expect_length(result, 5)
  expect_equal(result, (0:4) * 1.5)
})

test_that("Lazy<Int32Array> returns ALTREP integer", {
  result <- test_lazy_arrow_i32(5L)
  expect_type(result, "integer")
  expect_equal(result, c(0L, 10L, 20L, 30L, 40L))
})

test_that("Lazy<BooleanArray> returns ALTREP logical", {
  result <- test_lazy_arrow_bool(5L)
  expect_type(result, "logical")
  expect_length(result, 5)
  expect_true(result[1])
  expect_false(result[2])
  expect_true(result[3])
})

test_that("Lazy<Float64Array> with nulls becomes NA", {
  result <- test_lazy_arrow_f64_with_nulls()
  expect_type(result, "double")
  expect_length(result, 5)
  expect_equal(result[1], 1.0)
  expect_true(is.na(result[2]))
  expect_equal(result[3], 3.0)
  expect_true(is.na(result[4]))
  expect_equal(result[5], 5.0)
})

# endregion
