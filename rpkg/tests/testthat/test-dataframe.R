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

# ── Collection expansion tests ────────────────────────────────────────────────

test_that("DataFrameRow expands [f64; 3] into suffixed columns", {
  df <- create_expanded_points_df()

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2)
  expect_equal(names(df), c("label", "coords_1", "coords_2", "coords_3"))

  expect_equal(df$label, c("A", "B"))
  expect_equal(df$coords_1, c(1.0, 4.0))
  expect_equal(df$coords_2, c(2.0, 5.0))
  expect_equal(df$coords_3, c(3.0, 6.0))

  expect_type(df$coords_1, "double")
  expect_type(df$coords_2, "double")
  expect_type(df$coords_3, "double")
})

test_that("DataFrameRow supports skip, rename, and Vec width expansion", {
  df <- create_scored_items_df()

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3)

  # "name" renamed to "item", "_internal_id" skipped
  expect_equal(names(df), c("item", "scores_1", "scores_2", "scores_3"))

  expect_equal(df$item, c("alpha", "beta", "gamma"))

  # First row: full scores [10, 20, 30]
  expect_equal(df$scores_1[1], 10.0)
  expect_equal(df$scores_2[1], 20.0)
  expect_equal(df$scores_3[1], 30.0)

  # Second row: only first score [40], rest NA
  expect_equal(df$scores_1[2], 40.0)
  expect_true(is.na(df$scores_2[2]))
  expect_true(is.na(df$scores_3[2]))

  # Third row: empty scores [], all NA
  expect_true(is.na(df$scores_1[3]))
  expect_true(is.na(df$scores_2[3]))
  expect_true(is.na(df$scores_3[3]))
})

test_that("DataFrameRow enum with array expansion in variants", {
  df <- create_sensor_readings_df()

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3)

  # Tag column
  expect_true("_type" %in% names(df))
  expect_equal(df$`_type`, c("Xyz", "Single", "Xyz"))

  # Shared field: sensor_id always present
  expect_equal(df$sensor_id, c(1L, 2L, 3L))

  # Xyz-only expanded fields: values_1, values_2, values_3
  expect_equal(df$values_1, c(1.0, NA, 7.0))
  expect_equal(df$values_2, c(2.0, NA, 8.0))
  expect_equal(df$values_3, c(3.0, NA, 9.0))

  # Single-only field
  expect_equal(df$reading, c(NA, 42.0, NA))
})

# ── to_dataframe_split tests ──────────────────────────────────────────────────

test_that("to_dataframe_split returns named list of data.frames for multi-variant enum", {
  result <- create_events_split()

  # outer result is a plain list, not a data.frame
  expect_type(result, "list")
  expect_false(is.data.frame(result))

  # names are snake_case variant names
  expect_setequal(names(result), c("click", "impression", "error"))

  # each element is a data.frame
  expect_s3_class(result$click,      "data.frame")
  expect_s3_class(result$impression, "data.frame")
  expect_s3_class(result$error,      "data.frame")

  # click: 2 rows, only Click columns (id, x, y) — no NA columns from other variants
  expect_equal(nrow(result$click), 2)
  expect_equal(sort(names(result$click)), sort(c("id", "x", "y")))
  expect_equal(result$click$id, c(1L, 4L))
  expect_equal(result$click$x,  c(1.5, 3.0))
  expect_equal(result$click$y,  c(2.5, 4.0))

  # impression: 1 row, only Impression columns (id, slot)
  expect_equal(nrow(result$impression), 1)
  expect_equal(sort(names(result$impression)), sort(c("id", "slot")))
  expect_equal(result$impression$id,   2L)
  expect_equal(result$impression$slot, "top_banner")

  # error: 1 row, only Error columns (id, code, message)
  expect_equal(nrow(result$error), 1)
  expect_equal(sort(names(result$error)), sort(c("id", "code", "message")))
  expect_equal(result$error$code,    404L)
  expect_equal(result$error$message, "not found")
})

test_that("to_dataframe_split single variant returns data.frame directly", {
  result <- create_single_event_split()

  # single variant -> data.frame, not a named list of data.frames
  expect_s3_class(result, "data.frame")
  expect_equal(nrow(result), 2)
  expect_equal(sort(names(result)), sort(c("x", "y")))
  expect_equal(result$x, c(1.0, 3.0))
  expect_equal(result$y, c(2.0, 4.0))
})

test_that("to_dataframe_split shared field appears in both partitions non-optionally", {
  result <- create_shapes_split()

  expect_type(result, "list")
  expect_false(is.data.frame(result))
  expect_setequal(names(result), c("circle", "rect"))

  # circle: 2 rows, only Circle columns — area non-optional (no NAs)
  expect_s3_class(result$circle, "data.frame")
  expect_equal(nrow(result$circle), 2)
  expect_equal(sort(names(result$circle)), sort(c("radius", "area")))
  expect_false(anyNA(result$circle$area))
  expect_false(anyNA(result$circle$radius))
  expect_equal(result$circle$radius, c(5.0, 1.0))
  expect_equal(result$circle$area,   c(78.54, pi))

  # rect: 1 row, only Rect columns — area non-optional, no rect-specific NA
  expect_s3_class(result$rect, "data.frame")
  expect_equal(nrow(result$rect), 1)
  expect_equal(sort(names(result$rect)), sort(c("width", "height", "area")))
  expect_false(anyNA(result$rect$area))
  expect_false(anyNA(result$rect$width))
  expect_false(anyNA(result$rect$height))
  expect_equal(result$rect$width,  3.0)
  expect_equal(result$rect$height, 4.0)
  expect_equal(result$rect$area,   12.0)
})
