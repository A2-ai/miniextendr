# Tests for ndarray integration
#
# These tests verify the ndarray adapter traits (RNdArrayOps, RNdSlice, RNdSlice2D, RNdIndex)
# through wrapper types NdVec, NdMatrix, NdArrayDyn, and NdIntVec.

# Helper to skip if ndarray feature is not enabled
skip_if_ndarray_disabled <- function() {
  skip_if_not("ndarray" %in% miniextendr::rpkg_enabled_features(), "ndarray feature not enabled")
}

# =============================================================================
# NdVec tests (1D array)
# =============================================================================

test_that("NdVec can be created", {
  skip_if_ndarray_disabled()
  v <- NdVec$new(c(1, 2, 3, 4, 5))
  expect_true(inherits(v, "NdVec"))
})

test_that("NdVec from_range works", {
  skip_if_ndarray_disabled()
  v <- NdVec$from_range(0, 5, 1)
  expect_equal(v$len(), 5L)
  expect_equal(v$first(), 0)
  expect_equal(v$last(), 4)
})

test_that("NdVec RNdArrayOps - metadata", {
  skip_if_ndarray_disabled()
  v <- NdVec$new(c(1, 2, 3, 4, 5))

  expect_equal(v$len(), 5L)
  expect_false(v$is_empty())
  expect_equal(v$ndim(), 1L)
  expect_equal(v$shape(), 5L)
})

test_that("NdVec RNdArrayOps - aggregations", {
  skip_if_ndarray_disabled()
  v <- NdVec$new(c(1, 2, 3, 4, 5))

  expect_equal(v$sum(), 15)
  expect_equal(v$mean(), 3)
  expect_equal(v$min(), 1)
  expect_equal(v$max(), 5)
  expect_equal(v$product(), 120)
})

test_that("NdVec RNdArrayOps - variance and std", {
  skip_if_ndarray_disabled()
  # Data with known variance
  v <- NdVec$new(c(2, 4, 4, 4, 5, 5, 7, 9))
  expect_equal(v$mean(), 5)
  expect_equal(v$var(), 4)
  expect_equal(v$std(), 2)
})

test_that("NdVec RNdArrayOps - empty array", {
  skip_if_ndarray_disabled()
  v <- NdVec$new(numeric(0))

  expect_equal(v$len(), 0L)
  expect_true(v$is_empty())
  expect_true(is.nan(v$mean()))
  expect_true(is.nan(v$var()))
})

test_that("NdVec RNdSlice - element access", {
  skip_if_ndarray_disabled()
  v <- NdVec$new(c(10, 20, 30, 40, 50))

  # 0-indexed access
  expect_equal(v$get(0L), 10)
  expect_equal(v$get(2L), 30)
  expect_equal(v$get(4L), 50)

  # Out of bounds returns error (Option::None becomes error in miniextendr)
  expect_error(v$get(5L), "returned None")
  expect_error(v$get(-1L), "returned None")
})

test_that("NdVec RNdSlice - first and last", {
  skip_if_ndarray_disabled()
  v <- NdVec$new(c(1, 2, 3))

  expect_equal(v$first(), 1)
  expect_equal(v$last(), 3)

  # Empty vector - Option::None becomes error
  empty <- NdVec$new(numeric(0))
  expect_error(empty$first(), "returned None")
  expect_error(empty$last(), "returned None")
})

test_that("NdVec RNdSlice - slice_1d", {
  skip_if_ndarray_disabled()
  v <- NdVec$new(c(1, 2, 3, 4, 5))

  # Normal slice [1, 4) -> elements 2, 3, 4
  expect_equal(v$slice_1d(1L, 4L), c(2, 3, 4))

  # Full slice
  expect_equal(v$slice_1d(0L, 5L), c(1, 2, 3, 4, 5))

  # Clamped end
  expect_equal(v$slice_1d(3L, 100L), c(4, 5))

  # Empty range
  expect_equal(v$slice_1d(3L, 2L), numeric(0))
})

test_that("NdVec RNdSlice - get_many", {
  skip_if_ndarray_disabled()
  v <- NdVec$new(c(10, 20, 30, 40, 50))

  results <- v$get_many(c(0L, 2L, 4L, 10L))
  expect_equal(results[[1]], 10)
  expect_equal(results[[2]], 30)
  expect_equal(results[[3]], 50)
  expect_true(is.na(results[[4]]))  # Out of bounds returns NA
})

test_that("NdVec RNdSlice - is_valid_index", {
  skip_if_ndarray_disabled()
  v <- NdVec$new(c(1, 2, 3))

  expect_true(v$is_valid_index(0L))
  expect_true(v$is_valid_index(2L))
  expect_false(v$is_valid_index(3L))
  expect_false(v$is_valid_index(-1L))
})

test_that("NdVec to_r returns R vector", {
  skip_if_ndarray_disabled()
  v <- NdVec$new(c(1, 2, 3))

  r_vec <- v$to_r()
  expect_equal(r_vec, c(1, 2, 3))
})

# =============================================================================
# NdMatrix tests (2D array)
# =============================================================================

test_that("NdMatrix can be created from R matrix", {
  skip_if_ndarray_disabled()
  m <- NdMatrix$new(matrix(as.double(1:6), nrow = 2, ncol = 3))
  expect_true(inherits(m, "NdMatrix"))
})

test_that("NdMatrix from_rows works", {
  skip_if_ndarray_disabled()
  m <- NdMatrix$from_rows(2L, 3L, c(1, 2, 3, 4, 5, 6))
  expect_equal(m$nrows(), 2L)
  expect_equal(m$ncols(), 3L)
})

test_that("NdMatrix identity works", {
  skip_if_ndarray_disabled()
  m <- NdMatrix$identity(3L)
  expect_equal(m$nrows(), 3L)
  expect_equal(m$ncols(), 3L)
  expect_equal(m$diag(), c(1, 1, 1))
})

test_that("NdMatrix RNdArrayOps - metadata", {
  skip_if_ndarray_disabled()
  m <- NdMatrix$new(matrix(as.double(1:6), nrow = 2, ncol = 3))

  expect_equal(m$len(), 6L)
  expect_false(m$is_empty())
  expect_equal(m$ndim(), 2L)
  expect_equal(m$shape(), c(2L, 3L))
})

test_that("NdMatrix RNdArrayOps - aggregations", {
  skip_if_ndarray_disabled()
  m <- NdMatrix$new(matrix(as.double(1:6), nrow = 2, ncol = 3))

  expect_equal(m$sum(), 21)
  expect_equal(m$mean(), 3.5)
  expect_equal(m$min(), 1)
  expect_equal(m$max(), 6)
})

test_that("NdMatrix RNdSlice2D - element access", {
  skip_if_ndarray_disabled()
  # R matrix fills by column: [[1,3,5], [2,4,6]]
  m <- NdMatrix$new(matrix(as.double(1:6), nrow = 2, ncol = 3))

  # 0-indexed access
  expect_equal(m$get_2d(0L, 0L), 1)
  expect_equal(m$get_2d(1L, 0L), 2)
  expect_equal(m$get_2d(0L, 2L), 5)
  expect_equal(m$get_2d(1L, 2L), 6)

  # Out of bounds - Option::None becomes error
  expect_error(m$get_2d(2L, 0L), "returned None")
  expect_error(m$get_2d(-1L, 0L), "returned None")
})

test_that("NdMatrix RNdSlice2D - row and col", {
  skip_if_ndarray_disabled()
  m <- NdMatrix$new(matrix(as.double(1:6), nrow = 2, ncol = 3))

  # Rows
  expect_equal(m$row(0L), c(1, 3, 5))
  expect_equal(m$row(1L), c(2, 4, 6))
  expect_equal(m$row(2L), numeric(0))  # Out of bounds

  # Columns
  expect_equal(m$col(0L), c(1, 2))
  expect_equal(m$col(1L), c(3, 4))
  expect_equal(m$col(2L), c(5, 6))
  expect_equal(m$col(3L), numeric(0))  # Out of bounds
})

test_that("NdMatrix RNdSlice2D - diag", {
  skip_if_ndarray_disabled()
  m <- NdMatrix$new(matrix(as.double(1:9), nrow = 3, ncol = 3))
  expect_equal(m$diag(), c(1, 5, 9))
})

test_that("NdMatrix RNdSlice2D - dimensions", {
  skip_if_ndarray_disabled()
  m <- NdMatrix$new(matrix(as.double(1:12), nrow = 3, ncol = 4))
  expect_equal(m$nrows(), 3L)
  expect_equal(m$ncols(), 4L)
})

test_that("NdMatrix to_r returns R matrix", {
  skip_if_ndarray_disabled()
  m <- NdMatrix$new(matrix(as.double(1:6), nrow = 2, ncol = 3))

  r_mat <- m$to_r()
  expect_true(is.matrix(r_mat))
  expect_equal(dim(r_mat), c(2, 3))
})

# =============================================================================
# NdArrayDyn tests (N-dimensional array)
# =============================================================================

test_that("NdArrayDyn can be created", {
  skip_if_ndarray_disabled()
  arr <- NdArrayDyn$new(c(2L, 3L), as.double(1:6))
  expect_true(inherits(arr, "NdArrayDyn"))
})

test_that("NdArrayDyn zeros and ones", {
  skip_if_ndarray_disabled()

  z <- NdArrayDyn$zeros(c(2L, 3L))
  expect_equal(z$len(), 6L)
  expect_equal(z$sum(), 0)


  o <- NdArrayDyn$ones(c(2L, 3L))
  expect_equal(o$len(), 6L)
  expect_equal(o$sum(), 6)
})

test_that("NdArrayDyn RNdIndex - metadata", {
  skip_if_ndarray_disabled()
  arr <- NdArrayDyn$new(c(2L, 3L, 4L), as.numeric(1:24))

  expect_equal(arr$ndim(), 3L)
  expect_equal(arr$shape_nd(), c(2L, 3L, 4L))
  expect_equal(arr$len_nd(), 24L)
})

test_that("NdArrayDyn RNdIndex - element access", {
  skip_if_ndarray_disabled()
  arr <- NdArrayDyn$new(c(2L, 3L), as.numeric(1:6))

  # 0-indexed access (row-major order)
  # Shape (2, 3) with data 1:6:
  # Row 0: [1, 2, 3], Row 1: [4, 5, 6]
  expect_equal(arr$get_nd(c(0L, 0L)), 1)
  expect_equal(arr$get_nd(c(0L, 1L)), 2)
  expect_equal(arr$get_nd(c(1L, 0L)), 4)
  expect_equal(arr$get_nd(c(1L, 2L)), 6)

  # Out of bounds - Option::None becomes error
  expect_error(arr$get_nd(c(2L, 0L)), "returned None")

  # Wrong number of indices - Option::None becomes error
  expect_error(arr$get_nd(c(0L)), "returned None")
  expect_error(arr$get_nd(c(0L, 0L, 0L)), "returned None")
})

test_that("NdArrayDyn RNdIndex - slice_nd", {
  skip_if_ndarray_disabled()
  # 3x3 array
  arr <- NdArrayDyn$new(c(3L, 3L), as.numeric(1:9))

  # Slice [0:2, 0:2] - 2x2 subarray
  slice <- arr$slice_nd(c(0L, 0L), c(2L, 2L))
  expect_length(slice, 4)
})

test_that("NdArrayDyn RNdIndex - flatten", {
  skip_if_ndarray_disabled()
  # 2x3 array
  arr <- NdArrayDyn$new(c(2L, 3L), as.numeric(1:6))

  # Fortran order (column-major, R-compatible)
  f <- arr$flatten()
  expect_length(f, 6)

  # C order (row-major)
  c_order <- arr$flatten_c()
  expect_length(c_order, 6)
})

test_that("NdArrayDyn RNdIndex - is_valid_nd", {
  skip_if_ndarray_disabled()
  arr <- NdArrayDyn$new(c(2L, 3L), as.numeric(1:6))

  expect_true(arr$is_valid_nd(c(0L, 0L)))
  expect_true(arr$is_valid_nd(c(1L, 2L)))
  expect_false(arr$is_valid_nd(c(2L, 0L)))
  expect_false(arr$is_valid_nd(c(-1L, 0L)))
})

test_that("NdArrayDyn RNdIndex - axis_slice", {
  skip_if_ndarray_disabled()
  # 2x3 array
  arr <- NdArrayDyn$new(c(2L, 3L), as.numeric(1:6))

  # Row 0 (axis 0, index 0)
  row0 <- arr$axis_slice(0L, 0L)
  expect_length(row0, 3)

  # Column 1 (axis 1, index 1)
  col1 <- arr$axis_slice(1L, 1L)
  expect_length(col1, 2)

  # Invalid axis
  expect_length(arr$axis_slice(2L, 0L), 0)
})

test_that("NdArrayDyn RNdIndex - reshape", {
  skip_if_ndarray_disabled()
  arr <- NdArrayDyn$new(c(2L, 3L), as.numeric(1:6))

  # Valid reshape
  reshaped <- arr$reshape(c(3L, 2L))
  expect_length(reshaped, 6)

  # Reshape to 1D
  flat <- arr$reshape(c(6L))
  expect_length(flat, 6)

  # Invalid reshape (wrong total size) - Option::None becomes error
  expect_error(arr$reshape(c(2L, 2L)), "returned None")
})

# =============================================================================
# NdIntVec tests (integer array)
# =============================================================================

test_that("NdIntVec can be created", {
  skip_if_ndarray_disabled()
  v <- NdIntVec$new(1:5)
  expect_true(inherits(v, "NdIntVec"))
})

test_that("NdIntVec RNdArrayOps - aggregations return f64", {
  skip_if_ndarray_disabled()
  v <- NdIntVec$new(1:5)

  expect_equal(v$sum(), 15)
  expect_equal(v$mean(), 3)
  expect_equal(v$min(), 1)
  expect_equal(v$max(), 5)
  expect_equal(v$product(), 120)
})

test_that("NdIntVec RNdSlice - element access", {
  skip_if_ndarray_disabled()
  v <- NdIntVec$new(c(10L, 20L, 30L))

  expect_equal(v$get(0L), 10L)
  expect_equal(v$get(2L), 30L)
  # Out of bounds - Option::None becomes error
  expect_error(v$get(3L), "returned None")
})

test_that("NdIntVec to_r returns R integer vector", {
  skip_if_ndarray_disabled()
  v <- NdIntVec$new(1:5)

  r_vec <- v$to_r()
  expect_equal(r_vec, 1:5)
})

# =============================================================================
# Round-trip conversion tests
# =============================================================================

test_that("ndarray_roundtrip_vec preserves data", {
  skip_if_ndarray_disabled()
  x <- c(1.5, 2.5, 3.5)
  result <- ndarray_roundtrip_vec(x)
  expect_equal(result, x)
})

test_that("ndarray_roundtrip_matrix preserves data", {
  skip_if_ndarray_disabled()
  x <- matrix(as.double(1:6), nrow = 2, ncol = 3)
  result <- ndarray_roundtrip_matrix(x)
  expect_equal(result, x)
})

test_that("ndarray_roundtrip_array preserves data", {
  skip_if_ndarray_disabled()
  x <- array(as.double(1:24), dim = c(2, 3, 4))
  result <- ndarray_roundtrip_array(x)
  expect_equal(result, x)
})

test_that("ndarray_roundtrip_int_vec preserves data", {
  skip_if_ndarray_disabled()
  x <- 1:10
  result <- ndarray_roundtrip_int_vec(x)
  expect_equal(result, x)
})

test_that("ndarray_roundtrip_int_matrix preserves data", {
  skip_if_ndarray_disabled()
  x <- matrix(1:6, nrow = 2, ncol = 3)
  result <- ndarray_roundtrip_int_matrix(x)
  expect_equal(result, x)
})
