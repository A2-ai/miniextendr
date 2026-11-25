test_that("RMatrix dims returns nrow and ncol", {
  m <- matrix(1:6, nrow = 2, ncol = 3)
  expect_equal(rarray_matrix_dims(m * 1.0), c(2L, 3L))
})

test_that("RMatrix len returns total elements", {
  m <- matrix(1:12, nrow = 3, ncol = 4)
  expect_identical(rarray_matrix_len(m * 1.0), 12L)
})

test_that("RVector sum returns correct total", {
  expect_equal(rarray_vector_sum(c(1.0, 2.0, 3.0, 4.0)), 10.0)
  expect_equal(rarray_vector_sum(numeric(0)), 0.0)
})

test_that("RMatrix column extraction works", {
  m <- matrix(c(1, 2, 3, 4, 5, 6), nrow = 2, ncol = 3)
  # 1-based access
  expect_equal(rarray_matrix_column(m, 1L), c(1, 2))
  expect_equal(rarray_matrix_column(m, 2L), c(3, 4))
  expect_equal(rarray_matrix_column(m, 3L), c(5, 6))

  # Out of bounds / non-positive index errors
  expect_error(rarray_matrix_column(m, 0L), "positive 1-based")
  expect_error(rarray_matrix_column(m, 4L), "out of bounds")
})

test_that("matrix metadata, a cell and a column can leave compact real input unexpanded", {
  has_buffer <- miniextendr:::rarray_matrix_has_buffer
  for (direct in c(FALSE, TRUE)) {
    # A compact REALSXP with its dim set in place. structure() returns a wrapper,
    # and from R 4.6.2 the buffer probe expands a wrapper's shared compact data.
    x <- miniextendr:::rarray_compact_matrix(1025L, 3L, TRUE)
    expect_false(has_buffer(x))
    if (direct) {
      dims <- .Call(miniextendr:::C_miniextendr_rarray_matrix_dims, NULL, x)
      size <- .Call(miniextendr:::C_miniextendr_rarray_matrix_len, NULL, x)
      cell <- .Call(miniextendr:::C_miniextendr_rarray_matrix_cell, NULL, x, 1000L, 3L)
    } else {
      dims <- rarray_matrix_dims(x)
      size <- rarray_matrix_len(x)
      cell <- miniextendr:::rarray_matrix_cell(x, 1000L, 3L)
    }
    expect_identical(dims, c(1025L, 3L))
    expect_identical(size, 3075L)
    expect_identical(cell, 3050)
    expect_false(has_buffer(x))

    column <- if (direct) {
      .Call(miniextendr:::C_miniextendr_rarray_matrix_column, NULL, x, 2L)
    } else {
      rarray_matrix_column(x, 2L)
    }
    expect_identical(column, as.double(1025L + seq_len(1025L)))
    expect_false(has_buffer(x))
    expect_error(miniextendr:::rarray_matrix_cell(x, 1026L, 1L), "out of bounds")
    expect_error(miniextendr:::rarray_matrix_cell(x, 1L, 4L), "out of bounds")
    expect_error(rarray_matrix_column(x, 4L), "out of bounds")
    expect_false(has_buffer(x))
  }
})

test_that("a compact matrix with an existing buffer yields the same column", {
  x <- miniextendr:::rarray_compact_matrix(1025L, 3L, TRUE)
  expect_false(miniextendr:::rarray_matrix_has_buffer(x))
  # The slice conversion deliberately requests contiguous data from this provider.
  expect_identical(miniextendr:::test_f64_slice_len(x), 3075L)
  expect_true(miniextendr:::rarray_matrix_has_buffer(x))
  expect_identical(rarray_matrix_column(x, 3L), as.double(2050L + seq_len(1025L)))
})

test_that("a returned column preserves double values and drops matrix attributes", {
  values <- c(NA_real_, NaN, -0, Inf, -Inf)
  x <- matrix(values, ncol = 1L, dimnames = list(letters[1:5], "column"))
  expect_true(miniextendr:::rarray_matrix_has_buffer(x))
  result <- rarray_matrix_column(x, 1L)
  expect_identical(result, values)
  expect_identical(1 / result[3L], -Inf)
  expect_null(attributes(result))
  expect_identical(dimnames(x), list(letters[1:5], "column"))
  expect_identical(rarray_matrix_column(matrix(numeric(), 0L, 2L), 2L), numeric())
  expect_error(rarray_matrix_column(matrix(numeric(), 3L, 0L), 1L), "out of bounds")
  expect_error(rarray_matrix_column(as.double(1:5), 1L), "length 1")
  expect_error(rarray_matrix_column(matrix(1:5), 1L), "integer")
})

test_that("direct matrix entry validates the current shape and returns a tagged Result error", {
  x <- miniextendr:::rarray_compact_matrix(1025L, 3L, TRUE)
  direct <- function(x, col) .Call(miniextendr:::C_miniextendr_rarray_matrix_column, NULL, x, col)
  expect_identical(direct(x, 3L), as.double(2050L + seq_len(1025L)))
  # The same compact values in another shape, which has no column 3.
  x <- miniextendr:::rarray_compact_matrix(3075L, 1L, TRUE)
  expect_false(miniextendr:::rarray_matrix_has_buffer(x))
  bad <- direct(x, 3L)
  expect_s3_class(bad, "rust_condition_value")
  expect_identical(bad$kind, "result_err")
  expect_match(bad$error, "out of bounds")
  expect_false(miniextendr:::rarray_matrix_has_buffer(x))
})

test_that("a matrix column consumes short reads and reports an incomplete provider", {
  x <- miniextendr:::rarray_region_matrix(FALSE)
  expect_false(miniextendr:::rarray_matrix_has_buffer(x))
  expect_identical(miniextendr:::rarray_matrix_cell(x, 3L, 2L), 7)
  expect_identical(rarray_matrix_column(x, 2L), c(5, 6, 7, 8, 9))
  expect_false(miniextendr:::rarray_matrix_has_buffer(x))

  stalled <- miniextendr:::rarray_region_matrix(TRUE)
  expect_error(rarray_matrix_column(stalled, 2L), "returned 0 values")
  expect_identical(rarray_matrix_column(stalled, 1L), c(0, 1, 2, 3, 4))
  expect_false(miniextendr:::rarray_matrix_has_buffer(stalled))
})

test_that("RMatrix attribute getters return both dimname axes", {
  m <- miniextendr:::rarray_compact_matrix(1025L, 3L, TRUE, dimnames = list(
    paste0("row-", seq_len(1025L)),
    c("col-a", "col-b", "col-c")
  ))

  expect_false(miniextendr:::rarray_matrix_has_buffer(m))
  expect_identical(miniextendr:::rarray_matrix_rownames(m), rownames(m))
  expect_identical(miniextendr:::rarray_matrix_colnames(m), colnames(m))
  expect_false(miniextendr:::rarray_matrix_has_buffer(m))
})

test_that("List attribute getters return both dimname axes", {
  m <- matrix(as.list(1:6), nrow = 2L, dimnames = list(
    c("row-a", "row-b"),
    c("col-a", "col-b", "col-c")
  ))

  expect_identical(miniextendr:::list_matrix_rownames(m), rownames(m))
  expect_identical(miniextendr:::list_matrix_colnames(m), colnames(m))
})

test_that("row-name getters report absent row names", {
  numeric_matrix <- matrix(
    as.double(1:6),
    nrow = 2L,
    dimnames = list(NULL, c("col-a", "col-b", "col-c"))
  )
  list_matrix <- matrix(
    as.list(1:6),
    nrow = 2L,
    dimnames = list(NULL, c("col-a", "col-b", "col-c"))
  )

  expect_error(miniextendr:::rarray_matrix_rownames(numeric_matrix), "expected row names")
  expect_error(miniextendr:::list_matrix_rownames(list_matrix), "expected row names")
})

test_that("a column keeps the matrix's native type", {
  column <- miniextendr:::rarray_native_matrix_column
  expect_identical(column(matrix(1:6, 3L), 2L), 4:6)
  expect_identical(column(matrix(c(TRUE, NA, FALSE, TRUE), 2L), 1L), c(TRUE, NA))
  expect_identical(column(matrix(as.raw(1:6), 2L), 3L), as.raw(5:6))
  z <- complex(real = 1:4, imaginary = -(1:4))
  expect_identical(column(matrix(z, 2L), 2L), z[3:4])
  expect_identical(column(matrix(c(1.5, NA), 1L), 2L), NA_real_)
  expect_identical(column(matrix(integer(), 0L, 2L), 2L), integer())
  expect_error(column(matrix(1:6, 3L), 3L), "out of bounds")
  expect_error(column(matrix(1:6, 3L), 0L), "start at 1")
})

test_that("a compact integer matrix gives a column without a buffer", {
  x <- miniextendr:::rarray_compact_matrix(300L, 2L, FALSE)
  has_buffer <- miniextendr:::rarray_int_matrix_has_buffer
  expect_false(has_buffer(x))
  expect_identical(miniextendr:::rarray_int_matrix_column(x, 2L), 301:600)
  expect_false(has_buffer(x))
})
