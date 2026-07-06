# Tests for ndarray integration
#
# These tests verify the ndarray adapter traits (RNdArrayOps, RNdSlice, RNdSlice2D, RNdIndex)
# through wrapper types NdVec, NdMatrix, NdArrayDyn, and NdIntVec.

# Helper to skip if ndarray feature is not enabled
skip_if_ndarray_disabled <- function() {
  skip_if_not("ndarray" %in% miniextendr::miniextendr_enabled_features(), "ndarray feature not enabled")
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

  # 1-based access
  expect_equal(v$get(1L), 10)
  expect_equal(v$get(3L), 30)
  expect_equal(v$get(5L), 50)

  # Out of bounds / non-positive index errors
  expect_error(v$get(6L), "out of bounds")
  expect_error(v$get(0L), "positive 1-based")
  expect_error(v$get(-1L), "positive 1-based")
})

test_that("NdVec RNdSlice - first and last", {
  skip_if_ndarray_disabled()
  v <- NdVec$new(c(1, 2, 3))

  expect_equal(v$first(), 1)
  expect_equal(v$last(), 3)

  # Empty vector - no value becomes an error
  empty <- NdVec$new(numeric(0))
  expect_error(empty$first(), "returned no value")
  expect_error(empty$last(), "returned no value")
})

test_that("NdVec RNdSlice - slice_1d", {
  skip_if_ndarray_disabled()
  v <- NdVec$new(c(1, 2, 3, 4, 5))

  # 1-based inclusive bounds (R's x[start:end] convention)
  expect_equal(v$slice_1d(2L, 4L), c(2, 3, 4))

  # Single element
  expect_equal(v$slice_1d(3L, 3L), 3)

  # Full slice
  expect_equal(v$slice_1d(1L, 5L), c(1, 2, 3, 4, 5))

  # Out-of-bounds / invalid ranges error
  expect_error(v$slice_1d(0L, 5L), "out of bounds")
  expect_error(v$slice_1d(3L, 100L), "out of bounds")
  expect_error(v$slice_1d(3L, 2L), "out of bounds")
})

test_that("NdVec RNdSlice - get_many", {
  skip_if_ndarray_disabled()
  v <- NdVec$new(c(10, 20, 30, 40, 50))

  results <- v$get_many(c(1L, 3L, 5L, 11L))
  expect_equal(results[[1]], 10)
  expect_equal(results[[2]], 30)
  expect_equal(results[[3]], 50)
  expect_true(is.na(results[[4]]))  # Out of bounds returns NA
})

test_that("NdVec RNdSlice - is_valid_index", {
  skip_if_ndarray_disabled()
  v <- NdVec$new(c(1, 2, 3))

  expect_true(v$is_valid_index(1L))
  expect_true(v$is_valid_index(3L))
  expect_false(v$is_valid_index(4L))
  expect_false(v$is_valid_index(0L))
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

  # 1-based access
  expect_equal(m$get_2d(1L, 1L), 1)
  expect_equal(m$get_2d(2L, 1L), 2)
  expect_equal(m$get_2d(1L, 3L), 5)
  expect_equal(m$get_2d(2L, 3L), 6)

  # Out of bounds / non-positive index errors
  expect_error(m$get_2d(3L, 1L), "out of bounds")
  expect_error(m$get_2d(0L, 1L), "positive 1-based")
})

test_that("NdMatrix RNdSlice2D - row and col", {
  skip_if_ndarray_disabled()
  m <- NdMatrix$new(matrix(as.double(1:6), nrow = 2, ncol = 3))

  # Rows (1-based)
  expect_equal(m$row(1L), c(1, 3, 5))
  expect_equal(m$row(2L), c(2, 4, 6))
  expect_error(m$row(3L), "out of bounds")  # Out of bounds errors

  # Columns (1-based)
  expect_equal(m$col(1L), c(1, 2))
  expect_equal(m$col(2L), c(3, 4))
  expect_equal(m$col(3L), c(5, 6))
  expect_error(m$col(4L), "out of bounds")  # Out of bounds errors
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

  # 1-based access (row-major order)
  # Shape (2, 3) with data 1:6:
  # Row 1: [1, 2, 3], Row 2: [4, 5, 6]
  expect_equal(arr$get_nd(c(1L, 1L)), 1)
  expect_equal(arr$get_nd(c(1L, 2L)), 2)
  expect_equal(arr$get_nd(c(2L, 1L)), 4)
  expect_equal(arr$get_nd(c(2L, 3L)), 6)

  # Out of bounds errors
  expect_error(arr$get_nd(c(3L, 1L)), "out of bounds")

  # Wrong number of indices errors
  expect_error(arr$get_nd(c(1L)), "out of bounds")
  expect_error(arr$get_nd(c(1L, 1L, 1L)), "out of bounds")
})

test_that("NdArrayDyn RNdIndex - slice_nd", {
  skip_if_ndarray_disabled()
  # 3x3 array (row-major data):
  # Row 1: [1, 2, 3], Row 2: [4, 5, 6], Row 3: [7, 8, 9]
  arr <- NdArrayDyn$new(c(3L, 3L), as.numeric(1:9))

  # 1-based inclusive bounds (R's x[s1:e1, s2:e2] convention):
  # top-left 2x2 subarray, flattened column-major
  slice <- arr$slice_nd(c(1L, 1L), c(2L, 2L))
  expect_equal(slice, c(1, 4, 2, 5))

  # Bottom-right 2x2 subarray
  expect_equal(arr$slice_nd(c(2L, 2L), c(3L, 3L)), c(5, 8, 6, 9))

  # Single element
  expect_equal(arr$slice_nd(c(2L, 2L), c(2L, 2L)), 5)

  # Dimensionality mismatch errors
  expect_error(arr$slice_nd(c(1L), c(2L, 2L)), "one bound per dimension")

  # Out-of-bounds / invalid ranges error
  expect_error(arr$slice_nd(c(0L, 1L), c(2L, 2L)), "out of bounds")
  expect_error(arr$slice_nd(c(1L, 1L), c(4L, 2L)), "out of bounds")
  expect_error(arr$slice_nd(c(2L, 2L), c(1L, 1L)), "out of bounds")
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

  expect_true(arr$is_valid_nd(c(1L, 1L)))
  expect_true(arr$is_valid_nd(c(2L, 3L)))
  expect_false(arr$is_valid_nd(c(3L, 1L)))
  expect_false(arr$is_valid_nd(c(0L, 1L)))
})

test_that("NdArrayDyn RNdIndex - axis_slice", {
  skip_if_ndarray_disabled()
  # 2x3 array (row-major data): Row 1: [1, 2, 3], Row 2: [4, 5, 6]
  arr <- NdArrayDyn$new(c(2L, 3L), as.numeric(1:6))

  # Both arguments are 1-based; axis follows R's MARGIN convention
  # (1 = first dimension). Row 1 (axis 1, index 1):
  expect_equal(arr$axis_slice(1L, 1L), c(1, 2, 3))

  # Row 2
  expect_equal(arr$axis_slice(1L, 2L), c(4, 5, 6))

  # Column 2 (axis 2, index 2)
  expect_equal(arr$axis_slice(2L, 2L), c(2, 5))

  # Out-of-bounds axis errors
  expect_error(arr$axis_slice(3L, 1L), "axis 3 is out of bounds")
  expect_error(arr$axis_slice(0L, 1L), "axis 0 is out of bounds")

  # Out-of-bounds index along the axis errors
  expect_error(arr$axis_slice(1L, 3L), "out of bounds along axis 1")
  expect_error(arr$axis_slice(1L, 0L), "out of bounds along axis 1")
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

  # Invalid reshape (wrong total size) errors
  expect_error(arr$reshape(c(2L, 2L)), "does not match")
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

  expect_equal(v$get(1L), 10L)
  expect_equal(v$get(3L), 30L)
  # Out of bounds errors
  expect_error(v$get(4L), "out of bounds")
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

# =============================================================================
# Additional tests for methods not covered above
# =============================================================================

test_that("NdVec view_to_r returns R vector view", {
  skip_if_ndarray_disabled()
  v <- NdVec$new(c(1, 2, 3, 4, 5))

  r_view <- v$view_to_r()
  expect_equal(r_view, c(1, 2, 3, 4, 5))
})

test_that("NdMatrix view_to_r returns R matrix view", {
  skip_if_ndarray_disabled()
  m <- NdMatrix$new(matrix(as.double(1:6), nrow = 2, ncol = 3))

  r_view <- m$view_to_r()
  expect_true(is.matrix(r_view))
  expect_equal(dim(r_view), c(2, 3))
})

test_that("NdMatrix variance and std work", {
  skip_if_ndarray_disabled()
  # Matrix with known variance
  m <- NdMatrix$new(matrix(c(2, 4, 4, 4, 5, 5, 7, 9), nrow = 2, ncol = 4))
  expect_equal(m$mean(), 5)
  expect_equal(m$var(), 4)
  expect_equal(m$std(), 2)
})

test_that("NdMatrix product works", {
  skip_if_ndarray_disabled()
  m <- NdMatrix$new(matrix(as.double(1:4), nrow = 2, ncol = 2))
  expect_equal(m$product(), 24)
})

test_that("NdArrayDyn aggregations work", {
  skip_if_ndarray_disabled()
  arr <- NdArrayDyn$new(c(2L, 3L), as.numeric(1:6))

  expect_equal(arr$len(), 6L)
  expect_false(arr$is_empty())
  expect_equal(arr$sum(), 21)
  expect_equal(arr$mean(), 3.5)
  expect_equal(arr$min(), 1)
  expect_equal(arr$max(), 6)
  expect_equal(arr$product(), 720)
})

test_that("NdArrayDyn variance and std work", {
  skip_if_ndarray_disabled()
  # Array with known variance
  arr <- NdArrayDyn$new(c(2L, 4L), c(2, 4, 4, 4, 5, 5, 7, 9))
  expect_equal(arr$mean(), 5)
  expect_equal(arr$var(), 4)
  expect_equal(arr$std(), 2)
})

test_that("NdArrayDyn to_r returns R array", {
  skip_if_ndarray_disabled()
  arr <- NdArrayDyn$new(c(2L, 3L), as.numeric(1:6))

  r_arr <- arr$to_r()
  expect_equal(length(r_arr), 6)
})

test_that("NdIntVec metadata methods work", {
  skip_if_ndarray_disabled()
  v <- NdIntVec$new(1:5)

  expect_equal(v$len(), 5L)
  expect_false(v$is_empty())
  expect_equal(v$ndim(), 1L)
  expect_equal(v$shape(), 5L)

  empty <- NdIntVec$new(integer(0))
  expect_true(empty$is_empty())
})

test_that("NdIntVec first and last work", {
  skip_if_ndarray_disabled()
  v <- NdIntVec$new(c(10L, 20L, 30L))

  expect_equal(v$first(), 10L)
  expect_equal(v$last(), 30L)
})

test_that("NdIntVec slice_1d works", {
  skip_if_ndarray_disabled()
  v <- NdIntVec$new(1:5)

  # 1-based inclusive bounds (R's x[start:end] convention)
  expect_equal(v$slice_1d(2L, 4L), 2:4)
  expect_equal(v$slice_1d(1L, 5L), 1:5)

  # Out-of-bounds / invalid ranges error
  expect_error(v$slice_1d(0L, 5L), "out of bounds")
  expect_error(v$slice_1d(2L, 6L), "out of bounds")
  expect_error(v$slice_1d(4L, 2L), "out of bounds")
})

test_that("NdIntVec variance and std work", {
  skip_if_ndarray_disabled()
  v <- NdIntVec$new(c(2L, 4L, 4L, 4L, 5L, 5L, 7L, 9L))
  expect_equal(v$mean(), 5)
  expect_equal(v$var(), 4)
  expect_equal(v$std(), 2)
})
