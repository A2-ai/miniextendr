# Tests for DataFusion integration (RSessionContext + RDataFrame)

make_test_df <- function() {
  data.frame(
    x = c(1L, 2L, 3L, 4L, 5L),
    y = c(10.0, 20.0, 30.0, 40.0, 50.0),
    name = c("a", "b", "c", "d", "e")
  )
}

# region: SQL queries

test_that("DataFusion SQL query works on data.frame", {
  df <- make_test_df()
  result <- test_df_sql_query(df, "SELECT x, y FROM df WHERE x > 2")
  expect_true(is.data.frame(result))
  expect_equal(nrow(result), 3)
  expect_true(all(result$x > 2))
})

test_that("DataFusion SQL aggregation works", {
  df <- make_test_df()
  result <- test_df_sql_query(df, "SELECT SUM(y) as total FROM df")
  expect_equal(result$total, 150.0)
})

# endregion

# region: RDataFrame operations

test_that("RDataFrame select returns subset of columns", {
  df <- make_test_df()
  result <- test_df_select(df, c("x", "y"))
  expect_true(is.data.frame(result))
  expect_equal(names(result), c("x", "y"))
  expect_equal(nrow(result), 5)
})

test_that("RDataFrame sort + limit works", {
  df <- make_test_df()
  result <- test_df_sort_limit(df, "y", FALSE, 3L)
  expect_equal(nrow(result), 3)
  # Descending sort: largest y first
  expect_equal(result$y, c(50.0, 40.0, 30.0))
})

test_that("RDataFrame columns returns column names", {
  df <- make_test_df()
  cols <- test_df_columns(df)
  expect_equal(cols, c("x", "y", "name"))
})

test_that("RDataFrame chain (SQL WHERE + sort + limit)", {
  df <- make_test_df()
  result <- test_df_chain(df)
  expect_true(is.data.frame(result))
  expect_true(all(result$x > 2))
  expect_true(nrow(result) <= 3)
  # Sorted ascending by x
  expect_equal(result$x, sort(result$x))
})

# endregion

# region: Aggregate and count

test_that("RDataFrame aggregate with group_by works", {
  df <- data.frame(
    name = c("a", "a", "b", "b"),
    y = c(10.0, 20.0, 30.0, 40.0)
  )
  result <- test_df_aggregate(df)
  expect_true(is.data.frame(result))
  expect_equal(nrow(result), 2)
  # Group "a": sum=30, count=2; group "b": sum=70, count=2
  a_row <- result[result$name == "a", ]
  expect_equal(a_row$total, 30.0)
  expect_equal(a_row$cnt, 2L)
})

test_that("RDataFrame global aggregation (no group_by) works", {
  df <- make_test_df()
  result <- test_df_global_agg(df)
  expect_equal(nrow(result), 1)
  expect_equal(result$avg_y, 30.0)
  expect_equal(result$max_x, 5L)
})

test_that("RDataFrame count works", {
  df <- make_test_df()
  result <- test_df_count(df)
  expect_equal(result, 3L)  # x > 2: values 3, 4, 5
})

# endregion
