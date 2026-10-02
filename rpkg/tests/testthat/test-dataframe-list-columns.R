# DataFrameRow list columns of wide-integer vectors: an
# `#[dataframe(as_list)] starts: Vec<usize>` field needs `Vec<Vec<usize>>: IntoR`.
# Each cell converts exactly like a returned `Vec<usize>`: integer when every
# value fits, double otherwise (decided per row); strict returns error instead.

big_start <- 3e9

test_that("as_list Vec<usize> field is a list column of integer vectors", {
  df <- list_column_starts_df()
  expect_s3_class(df, "data.frame")
  expect_identical(colnames(df), "starts")
  expect_identical(nrow(df), 4L)
  expect_true(is.list(df$starts))
  expect_identical(df$starts, list(c(1L, 5L, 9L), integer(0), 42L, integer(0)))
})

test_that("empty start rows are integer(0), not NULL or logical", {
  df <- list_column_starts_df()
  for (i in c(2L, 4L)) {
    expect_identical(typeof(df$starts[[i]]), "integer")
    expect_length(df$starts[[i]], 0L)
  }
})

test_that("a row past .Machine$integer.max widens to double, other rows stay integer", {
  df <- list_column_starts_wide_df()
  expect_true(is.list(df$starts))
  expect_identical(df$starts[[1L]], c(1L, 2L))
  expect_identical(df$starts[[2L]], c(big_start, 7))
  expect_identical(typeof(df$starts[[2L]]), "double")
  expect_identical(df$starts[[3L]], integer(0))
})

test_that("as_list Option<Vec<usize>> field gives NULL cells for None", {
  df <- list_column_maybe_starts_df()
  expect_true(is.list(df$starts))
  expect_identical(df$starts, list(c(3L, 4L), NULL, integer(0)))
})

test_that("enum Vec<usize> / Box<[usize]> fields are list columns with NULL for other variants", {
  df <- list_column_starts_event_df()
  expect_identical(df[["_type"]], c("Hit", "Miss", "Hit"))
  expect_identical(df$starts, list(c(1, big_start), NULL, integer(0)))
  expect_identical(df$offsets, list(0L, NULL, integer(0)))
  expect_identical(df$id, c(NA, 7L, NA))
})

test_that("Vec<Vec<usize>> returned directly matches the column cells", {
  expect_identical(
    list_column_starts_rows_wide(),
    list(c(1L, 2L), big_start, integer(0))
  )
})

test_that("strict Vec<Vec<usize>> in range returns integer rows", {
  expect_identical(
    strict_list_column_starts_rows(),
    list(c(1L, 2L), integer(0), .Machine$integer.max)
  )
})

test_that("strict Vec<Vec<usize>> errors on every out-of-range element at once", {
  e <- tryCatch(strict_list_column_starts_rows_overflow(), error = identity)
  expect_s3_class(e, "rust_error")
  expect_identical(e$kind, "panic")
  msg <- conditionMessage(e)
  expect_match(msg, "strict conversion failed for Vec<Vec<usize>>", fixed = TRUE)
  expect_match(msg, "at inner position 1 (element 2)", fixed = TRUE)
  expect_match(msg, "at inner position 2 (element 4)", fixed = TRUE)
})

test_that("strict Option<Vec<usize>> checks Some and returns NULL for None", {
  expect_error(strict_maybe_starts_overflow(), "strict conversion failed for Vec<usize>", fixed = TRUE)
  expect_null(strict_maybe_starts_none())
})
