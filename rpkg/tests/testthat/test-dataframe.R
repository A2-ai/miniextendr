test_that("DataFrameRow works with homogeneous types", {
  df <- create_points_df()

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2)
  expect_equal(ncol(df), 2)
  expect_equal(names(df), c("x", "y"))

  expect_equal(df$x, c(1.0, 3.0))
  expect_equal(df$y, c(2.0, 4.0))
  expect_type(df$x, "double")
  expect_type(df$y, "double")
})

test_that("DataFrameRow works with heterogeneous types", {
  df <- create_people_df()

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3)
  expect_equal(ncol(df), 4)
  expect_equal(names(df), c("name", "age", "height", "is_student"))

  # Check data types
  expect_type(df$name, "character")
  expect_type(df$age, "integer")
  expect_type(df$height, "double")
  expect_type(df$is_student, "logical")

  # Check values
  expect_equal(df$name, c("Alice", "Bob", "Charlie"))
  expect_equal(df$age, c(25L, 30L, 28L))
  expect_equal(df$height, c(165.5, 180.0, 175.2))
  expect_equal(df$is_student, c(TRUE, FALSE, TRUE))
})
