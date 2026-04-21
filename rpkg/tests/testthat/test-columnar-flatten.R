test_that("nested struct flattening produces prefixed columns", {
  df <- test_columnar_nested()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2)
  expect_equal(names(df), c("label", "point_x", "point_y"))
  expect_equal(df$label, c("a", "b"))
  expect_equal(df$point_x, c(1.0, 3.0))
  expect_equal(df$point_y, c(2.0, 4.0))
})

test_that("Option<Struct> with skip_serializing_if produces NA rows", {
  df <- test_columnar_optional_struct()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3)
  expect_equal(names(df), c("name", "extra_x", "extra_y"))
  expect_equal(df$name, c("has", "none", "also"))
  expect_equal(df$extra_x, c(1.0, NA, 5.0))
  expect_equal(df$extra_y, c(2.0, NA, 6.0))
})

test_that("deep nesting (3 levels) flattens fully", {
  df <- test_columnar_deep_nesting()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2)
  expect_equal(names(df), c("a", "mid_b", "mid_leaf_c"))
  expect_equal(df$a, c("x", "y"))
  expect_equal(df$mid_b, c(1L, 2L))
  expect_equal(df$mid_leaf_c, c(10.0, 20.0))
})

test_that("serde flatten removes prefix", {
  df <- test_columnar_serde_flatten()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2)
  expect_equal(names(df), c("id", "x", "y"))
  expect_equal(df$id, c(1L, 2L))
  expect_equal(df$x, c(10.0, 30.0))
  expect_equal(df$y, c(20.0, 40.0))
})

test_that("scalar skip_serializing_if produces NA", {
  df <- test_columnar_skip_serializing_if()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3)
  expect_equal(names(df), c("name", "tag", "value"))
  expect_equal(df$tag, c("t1", NA, "t3"))
  expect_equal(df$value, c(1.0, 2.0, 3.0))
})

test_that("rename API changes column names", {
  df <- test_columnar_rename()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2)
  expect_equal(names(df), c("label", "px", "py"))
  expect_equal(df$px, c(1.0, 3.0))
  expect_equal(df$py, c(2.0, 4.0))
})

test_that("rename nonexistent column is a silent no-op", {
  df <- test_columnar_rename_noop()
  expect_s3_class(df, "data.frame")
  expect_equal(names(df), c("x", "y"))
  expect_equal(df$x, 1.0)
  expect_equal(df$y, 2.0)
})

test_that("empty vec produces empty data.frame", {
  df <- test_columnar_empty()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 0)
  expect_equal(ncol(df), 0)
})

test_that("drop removes a column", {
  df <- test_columnar_drop()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2)
  expect_equal(names(df), c("label", "point_x"))
  expect_equal(df$label, c("a", "b"))
  expect_equal(df$point_x, c(1.0, 3.0))
})

test_that("select keeps only named columns in order", {
  df <- test_columnar_select()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2)
  expect_equal(names(df), c("point_y", "label"))
  expect_equal(df$point_y, c(2.0, 4.0))
  expect_equal(df$label, c("a", "b"))
})

test_that("with_column replaces an existing column", {
  df <- test_columnar_with_column_replace()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3)
  expect_equal(names(df), c("id", "value"))
  # id column was an integer column, replaced with a character vector
  expect_type(df$id, "character")
  expect_equal(df$id, c("a", "b", "c"))
  # value column is untouched
  expect_equal(df$value, c(10.0, 20.0, 30.0))
})

test_that("with_column appends a new column when the name is absent", {
  df <- test_columnar_with_column_append()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2)
  # Original columns first, new "label" appended at the end.
  expect_equal(names(df), c("x", "y", "label"))
  expect_equal(df$x, c(1.0, 3.0))
  expect_equal(df$y, c(2.0, 4.0))
  expect_type(df$label, "character")
  expect_equal(df$label, c("first", "second"))
})

test_that("strip_prefix removes prefix from matching columns", {
  df <- test_columnar_strip_prefix()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2)
  # "point_x" -> "x", "point_y" -> "y", "label" unchanged (no prefix match)
  expect_equal(names(df), c("label", "x", "y"))
  expect_equal(df$x, c(1.0, 3.0))
  expect_equal(df$y, c(2.0, 4.0))
})

test_that("untagged enum: multi-row discovery unions variant fields", {
  df <- test_columnar_untagged_enum()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3)
  # Ok rows have status+size, Err rows have error — all columns present
  expect_true("path" %in% names(df))
  expect_true("status" %in% names(df))
  expect_true("size" %in% names(df))
  expect_true("error" %in% names(df))
  # Ok rows: status+size filled, error NA
  expect_equal(df$path, c("a.txt", "b.txt", "c.txt"))
  expect_equal(df$status, c("current", NA, "absent"))
  expect_equal(df$size, c(100, NA, 200))
  # Err rows: error filled, status+size NA
  expect_equal(df$error, c(NA, "not found", NA))
})

test_that("internally tagged enum: kind column + variant-specific fields", {
  df <- test_columnar_tagged_enum()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3)
  # Tag column
  expect_true("kind" %in% names(df))
  expect_equal(df$kind, c("Click", "Scroll", "Click"))
  # Click fields
  expect_true("x" %in% names(df))
  expect_true("y" %in% names(df))
  expect_equal(df$x, c(10.0, NA, 30.0))
  expect_equal(df$y, c(20.0, NA, 40.0))
  # Scroll fields
  expect_true("delta" %in% names(df))
  expect_equal(df$delta, c(NA, -3.5, NA))
})
