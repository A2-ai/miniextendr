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

# ── Enum align tests ──────────────────────────────────────────────────────────

test_that("DataFrameRow align works with enum variants and tag column", {
  df <- create_events_df()

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3)

  # Tag column present
  expect_true("_type" %in% names(df))
  expect_equal(df$`_type`, c("Click", "Impression", "Error"))

  # Shared field (id) has no NAs
  expect_equal(df$id, c(1L, 2L, 3L))

  # Click-only fields are NA for non-Click rows
  expect_equal(df$x, c(1.5, NA, NA))
  expect_equal(df$y, c(2.5, NA, NA))


  # Impression-only field
  expect_equal(df$slot, c(NA, "top_banner", NA))

  # Error-only fields
  expect_equal(df$code, c(NA, NA, 404L))
  expect_equal(df$message, c(NA, NA, "not found"))
})

test_that("DataFrameRow align works without tag column", {
  df <- create_shapes_df()

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3)

  # No tag column
  expect_false("_tag" %in% names(df))

  # Shared field: area is present in both variants
  expect_equal(df$area, c(78.54, 12.0, pi))

  # Circle-only field
  expect_equal(df$radius, c(5.0, NA, 1.0))

  # Rect-only fields
  expect_equal(df$width, c(NA, 3.0, NA))
  expect_equal(df$height, c(NA, 4.0, NA))
})
