test_that("typed_df_attr_nrow returns correct row count", {
  df <- data.frame(subject = 1:3L, weight = c(60.0, 70.0, 80.0))
  expect_equal(typed_df_attr_nrow(df), 3L)
})

test_that("typed_df_attr_weight_sum returns correct column sum", {
  df <- data.frame(subject = 1:4L, weight = c(10.0, 20.0, 30.0, 40.0))
  expect_equal(typed_df_attr_weight_sum(df), 100.0)
})

test_that("typed_df: wrong column type raises R error with batched message", {
  # subject must be integer, not double
  df <- data.frame(subject = c(1.0, 2.0), weight = c(60.0, 70.0))
  expect_error(typed_df_attr_nrow(df), regexp = "subject")
})

test_that("typed_df: missing required column raises R error", {
  df <- data.frame(subject = 1:2L)  # missing 'weight'
  expect_error(typed_df_attr_weight_sum(df), regexp = "weight")
})

test_that("typed_df: extra column rejected under @exact", {
  df <- data.frame(x = 1:3L, y = c(1.1, 2.2, 3.3), extra = 1:3L)
  expect_error(typed_df_attr_exact_optional(df), regexp = "extra")
})

test_that("typed_df: optional column absent works", {
  df <- data.frame(x = 1:3L)
  expect_equal(typed_df_attr_exact_optional(df), 3L)
})

test_that("typed_df: optional column present works", {
  df <- data.frame(x = 1:3L, y = c(1.0, 2.0, 3.0))
  expect_equal(typed_df_attr_exact_optional(df), 6L)
})

test_that("typed_df: two-param punchline combines row counts", {
  left  <- data.frame(a = 1:4L)
  right <- data.frame(b = c(1.1, 2.2))
  expect_equal(typed_df_attr_two(left, right), 6L)
})
