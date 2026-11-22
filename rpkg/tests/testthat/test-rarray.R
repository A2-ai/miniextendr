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
    # structure() attaches dimensions without expanding R's compact REALSXP.
    x <- structure(as.double(seq_len(3075L)), dim = c(1025L, 3L))
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
  expect_error(rarray_matrix_column(as.double(1:5), 1L))
  expect_error(rarray_matrix_column(matrix(1:5), 1L))
})

test_that("a matrix column consumes short reads and reports an incomplete provider", {
  x <- miniextendr:::rarray_region_matrix(FALSE)
  expect_false(miniextendr:::rarray_matrix_has_buffer(x))
  expect_identical(rarray_matrix_column(x, 2L), c(5, 6, 7, 8, 9))
  expect_false(miniextendr:::rarray_matrix_has_buffer(x))

  stalled <- miniextendr:::rarray_region_matrix(TRUE)
  expect_error(rarray_matrix_column(stalled, 2L), "returned 0 values")
  expect_identical(rarray_matrix_column(stalled, 1L), c(0, 1, 2, 3, 4))
  expect_false(miniextendr:::rarray_matrix_has_buffer(stalled))
})

test_that("RMatrix attribute getters return both dimname axes", {
  m <- structure(as.double(seq_len(3075L)), dim = c(1025L, 3L), dimnames = list(
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
