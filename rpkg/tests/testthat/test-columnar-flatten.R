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

# ── vec_to_dataframe_split ────────────────────────────────────────────────────

test_that("externally-tagged enum split: named list, no NA columns", {
  result <- test_columnar_ext_tagged_split()

  # outer result is a named list, not a data.frame
  expect_type(result, "list")
  expect_false(is.data.frame(result))
  expect_setequal(names(result), c("Click", "Scroll"))

  # Click: 2 rows, only x and y — no delta column
  click <- result$Click
  expect_s3_class(click, "data.frame")
  expect_equal(nrow(click), 2)
  expect_equal(sort(names(click)), sort(c("x", "y")))
  expect_equal(click$x, c(1.0, 5.0))
  expect_equal(click$y, c(2.0, 6.0))

  # Scroll: 1 row, only delta — no x or y column
  scroll <- result$Scroll
  expect_s3_class(scroll, "data.frame")
  expect_equal(nrow(scroll), 1)
  expect_equal(names(scroll), "delta")
  expect_equal(scroll$delta, -3.0)
})

test_that("internally-tagged enum split: tag column dropped, no NA columns", {
  result <- test_columnar_int_tagged_split()

  # outer result is a named list
  expect_type(result, "list")
  expect_false(is.data.frame(result))
  expect_setequal(names(result), c("Click", "Scroll"))

  # Click: 2 rows, x + y only — no kind column
  click <- result$Click
  expect_s3_class(click, "data.frame")
  expect_equal(nrow(click), 2)
  expect_false("kind" %in% names(click))
  expect_equal(sort(names(click)), sort(c("x", "y")))
  expect_equal(click$x, c(10.0, 30.0))
  expect_equal(click$y, c(20.0, 40.0))

  # Scroll: 1 row, delta only — no kind column
  scroll <- result$Scroll
  expect_s3_class(scroll, "data.frame")
  expect_equal(nrow(scroll), 1)
  expect_false("kind" %in% names(scroll))
  expect_equal(names(scroll), "delta")
  expect_equal(scroll$delta, -3.5)
})

test_that("single-variant split returns bare data.frame", {
  result <- test_columnar_single_variant_split()

  # single variant → bare data.frame, not a named list of data.frames
  expect_s3_class(result, "data.frame")
  expect_equal(nrow(result), 2)
  expect_equal(sort(names(result)), sort(c("x", "y")))
  expect_equal(result$x, c(1.0, 3.0))
  expect_equal(result$y, c(2.0, 4.0))
})

test_that("empty input split returns an empty list()", {
  result <- test_columnar_empty_split()

  # variant set is unknowable from zero rows → list(), not a data.frame
  expect_type(result, "list")
  expect_false(is.data.frame(result))
  expect_equal(length(result), 0)
})

# region: vec_to_dataframe_split with-tag + collated shapes (closes #699)

test_that("PerVariantListWithTag prepends variant column to each partition", {
  result <- test_columnar_split_with_tag()

  expect_type(result, "list")
  expect_setequal(names(result), c("Click", "Scroll"))

  click <- result$Click
  expect_s3_class(click, "data.frame")
  expect_equal(names(click)[1], "variant")
  expect_equal(click$variant, c("Click", "Click"))
  expect_true(all(c("x", "y") %in% names(click)))

  scroll <- result$Scroll
  expect_s3_class(scroll, "data.frame")
  expect_equal(names(scroll)[1], "variant")
  expect_equal(scroll$variant, "Scroll")
  expect_true("delta" %in% names(scroll))
})

test_that("Collated shape returns a single data.frame with union schema + variant column", {
  df <- test_columnar_split_collated()

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3)
  expect_equal(names(df)[1], "variant")
  expect_true(all(c("x", "y", "delta") %in% names(df)))
  expect_equal(df$variant, c("Click", "Scroll", "Click"))

  # Click rows: x and y set, delta NA
  expect_equal(df$x[c(1, 3)], c(1.0, 5.0))
  expect_true(is.na(df$delta[1]))
  expect_true(is.na(df$delta[3]))

  # Scroll row: delta set, x and y NA
  expect_equal(df$delta[2], -3.0)
  expect_true(is.na(df$x[2]))
  expect_true(is.na(df$y[2]))
})

# endregion

# region: map_to_dataframe (closes #700)

test_that("map_to_dataframe over BTreeMap returns key column first", {
  df <- test_map_to_dataframe_btreemap()

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3)
  expect_equal(names(df)[1], "subject")
  expect_true(all(c("cmax", "tmax") %in% names(df)))

  # BTreeMap is ordered, so subject column should be sorted 1,2,3
  expect_equal(df$subject, c(1L, 2L, 3L))
  expect_equal(df$cmax, c(10.5, 8.1, 12.0))
  expect_equal(df$tmax, c(2.0, 3.5, 1.0))
})

test_that("hashmap_to_dataframe sorts by key for deterministic order", {
  df <- test_hashmap_to_dataframe()

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3)
  expect_equal(names(df)[1], "subject")
  # HashMap iteration order is unspecified, but the helper sorts by key.
  expect_equal(df$subject, c(1L, 2L, 3L))
  expect_equal(df$cmax, c(10.5, 8.1, 12.0))
})

# endregion

# region: result_to_dataframe (closes #697)

test_that("result_to_dataframe(Auto) returns bare data.frame on all-Ok", {
  df <- test_result_to_dataframe_auto_all_ok()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2)
  expect_equal(sort(names(df)), sort(c("id", "value")))
})

test_that("result_to_dataframe(Auto) returns list(results, error) on mixed input", {
  res <- test_result_to_dataframe_auto_mixed()
  expect_type(res, "list")
  expect_equal(names(res), c("results", "error"))
  expect_s3_class(res$results, "data.frame")
  expect_s3_class(res$error, "data.frame")
  expect_equal(nrow(res$results), 2)
  expect_equal(nrow(res$error), 1)
  expect_equal(res$error$reason, "bad")
})

test_that("result_to_dataframe(Collated) produces single data.frame with is_error", {
  df <- test_result_to_dataframe_collated()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3)
  expect_equal(names(df)[1], "is_error")
  expect_equal(df$is_error, c(FALSE, TRUE, FALSE))
  # Union schema: id (in both variants) + value + reason
  expect_true(all(c("id", "value", "reason") %in% names(df)))
  # Ok rows: value set, reason NA
  expect_equal(df$value[c(1, 3)], c(1.0, 3.0))
  expect_true(is.na(df$reason[1]))
  # Err row: reason set, value NA
  expect_equal(df$reason[2], "bad")
  expect_true(is.na(df$value[2]))
})

test_that("result_to_dataframe(Split) with all-Err puts sentinel in results slot", {
  res <- test_result_to_dataframe_split_all_err()
  expect_type(res, "list")
  expect_equal(names(res), c("results", "error"))
  # No Ok rows → sentinel `()` lands as NULL on the R side
  expect_null(res$results)
  expect_s3_class(res$error, "data.frame")
  expect_equal(nrow(res$error), 2)
})

# endregion
