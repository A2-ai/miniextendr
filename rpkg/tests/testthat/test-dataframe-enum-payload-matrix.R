# Cardinality x payload-shape matrix for enum-derived data frames.
# Mirrors rpkg/src/rust/dataframe_enum_payload_matrix.rs.
#
# Per shape, four cardinality cells via to_dataframe_split:
#   1v1r  - one variant, one row
#   1vNr  - one variant, many rows
#   Nv1r  - many variants, one row each
#   NvNr  - many variants, many rows each
# Plus an NvNr align-path test per shape to confirm NA-fill columns.

# ── helpers ───────────────────────────────────────────────────────────────────

expect_split_partition <- function(part, expected_nrow, expected_cols) {
  expect_s3_class(part, "data.frame")
  expect_equal(nrow(part), expected_nrow)
  expect_equal(sort(names(part)), sort(expected_cols))
}

# ── 0a. Vec<i32> opaque (list-column, no expand/width) ───────────────────────

test_that("vec opaque — split 1v1r: one Items row, no_items has 0 rows", {
  res <- vec_opaque_split_1v1r()
  expect_setequal(names(res), c("items", "no_items"))
  expect_split_partition(res$items, 1, c("label", "items"))
  expect_split_partition(res$no_items, 0, "label")
  expect_equal(res$items$items[[1]], c(1L, 2L, 3L))
})

test_that("vec opaque — split 1vNr: multiple Items rows, no_items has 0 rows", {
  res <- vec_opaque_split_1vnr()
  expect_split_partition(res$items, 3, c("label", "items"))
  expect_split_partition(res$no_items, 0, "label")
  expect_equal(res$items$items[[1]], c(1L, 2L, 3L))
  expect_equal(res$items$items[[2]], c(4L, 5L))
  expect_equal(res$items$items[[3]], integer(0))
})

test_that("vec opaque — split Nv1r: one row each, no_items omits items column", {
  res <- vec_opaque_split_nv1r()
  expect_split_partition(res$items, 1, c("label", "items"))
  expect_split_partition(res$no_items, 1, "label")
  expect_equal(res$no_items$label, "b")
  expect_equal(res$items$items[[1]], c(1L, 2L, 3L))
})

test_that("vec opaque — align NvNr: present rows are integer vectors, absent rows are NULL", {
  df <- vec_opaque_align_nvnr()
  expect_equal(df$`_type`, c("Items", "NoItems", "Items", "NoItems"))
  expect_equal(df$label, c("a", "b", "c", "d"))
  # items column is a list-column
  expect_type(df$items, "list")
  expect_equal(df$items[[1]], c(1L, 2L, 3L))
  expect_null(df$items[[2]])
  expect_equal(df$items[[3]], c(4L, 5L))
  expect_null(df$items[[4]])
})

test_that("vec opaque — split NvNr: items partition has list-column, no_items partition omits it", {
  res <- vec_opaque_split_nvnr()
  expect_setequal(names(res), c("items", "no_items"))
  expect_s3_class(res$items, "data.frame")
  expect_s3_class(res$no_items, "data.frame")
  expect_equal(nrow(res$items), 2)
  expect_equal(nrow(res$no_items), 2)
  expect_true("items" %in% names(res$items))
  expect_false("items" %in% names(res$no_items))
  expect_equal(res$items$items[[1]], c(1L, 2L, 3L))
  expect_equal(res$items$items[[2]], c(4L, 5L))
})

# ── 0b. HashSet<String> opaque (list-column, unordered elements) ──────────────

test_that("hashset — split 1v1r: one Tagged row, untagged has 0 rows", {
  res <- hashset_split_1v1r()
  expect_setequal(names(res), c("tagged", "untagged"))
  expect_split_partition(res$tagged, 1, c("id", "tags"))
  expect_split_partition(res$untagged, 0, "id")
  # HashSet is unordered — use setequal
  expect_setequal(res$tagged$tags[[1]], c("a", "b"))
})

test_that("hashset — split 1vNr: multiple Tagged rows, untagged has 0 rows", {
  res <- hashset_split_1vnr()
  expect_split_partition(res$tagged, 3, c("id", "tags"))
  expect_split_partition(res$untagged, 0, "id")
  expect_setequal(res$tagged$tags[[1]], c("a", "b"))
  expect_setequal(res$tagged$tags[[2]], c("c"))
  expect_equal(length(res$tagged$tags[[3]]), 0L)
})

test_that("hashset — split Nv1r: one row each, untagged omits tags column", {
  res <- hashset_split_nv1r()
  expect_split_partition(res$tagged, 1, c("id", "tags"))
  expect_split_partition(res$untagged, 1, "id")
  expect_equal(res$untagged$id, 2L)
  expect_setequal(res$tagged$tags[[1]], c("a", "b"))
})

test_that("hashset — align NvNr: present rows are character vectors, absent rows are NULL", {
  df <- hashset_align_nvnr()
  expect_equal(df$`_type`, c("Tagged", "Untagged", "Tagged", "Untagged"))
  expect_equal(df$id, c(1L, 2L, 3L, 4L))
  expect_type(df$tags, "list")
  # HashSet is unordered — use setequal
  expect_setequal(df$tags[[1]], c("a", "b"))
  expect_null(df$tags[[2]])
  expect_setequal(df$tags[[3]], c("c"))
  expect_null(df$tags[[4]])
})

test_that("hashset — split NvNr: tagged partition has tags list-column, untagged does not", {
  res <- hashset_split_nvnr()
  expect_setequal(names(res), c("tagged", "untagged"))
  expect_equal(nrow(res$tagged), 2)
  expect_equal(nrow(res$untagged), 2)
  expect_true("tags" %in% names(res$tagged))
  expect_false("tags" %in% names(res$untagged))
  expect_setequal(res$tagged$tags[[1]], c("a", "b"))
  expect_setequal(res$tagged$tags[[2]], c("c"))
})

# ── 0c. BTreeSet<i32> opaque (list-column, sorted elements) ──────────────────

test_that("btreeset — split 1v1r: one Cats row, no_cats has 0 rows, sorted order", {
  res <- btreeset_split_1v1r()
  expect_setequal(names(res), c("cats", "no_cats"))
  expect_split_partition(res$cats, 1, c("label", "cats"))
  expect_split_partition(res$no_cats, 0, "label")
  # BTreeSet sorts input [3,1,2] → [1,2,3]
  expect_equal(res$cats$cats[[1]], c(1L, 2L, 3L))
})

test_that("btreeset — split 1vNr: multiple Cats rows, no_cats has 0 rows, sorted order", {
  res <- btreeset_split_1vnr()
  expect_split_partition(res$cats, 3, c("label", "cats"))
  expect_split_partition(res$no_cats, 0, "label")
  # BTreeSet sorts [3,1,2] → [1,2,3] and [5,4] → [4,5]
  expect_equal(res$cats$cats[[1]], c(1L, 2L, 3L))
  expect_equal(res$cats$cats[[2]], c(4L, 5L))
  expect_equal(res$cats$cats[[3]], integer(0))
})

test_that("btreeset — split Nv1r: one row each, no_cats omits cats column", {
  res <- btreeset_split_nv1r()
  expect_split_partition(res$cats, 1, c("label", "cats"))
  expect_split_partition(res$no_cats, 1, "label")
  expect_equal(res$no_cats$label, "b")
  expect_equal(res$cats$cats[[1]], c(1L, 2L, 3L))
})

test_that("btreeset — align NvNr: present rows are sorted integer vectors, absent rows are NULL", {
  df <- btreeset_align_nvnr()
  expect_equal(df$`_type`, c("Cats", "NoCats", "Cats", "NoCats"))
  expect_equal(df$label, c("a", "b", "c", "d"))
  expect_type(df$cats, "list")
  # BTreeSet is sorted — order is guaranteed
  expect_equal(df$cats[[1]], c(1L, 2L, 3L))
  expect_null(df$cats[[2]])
  expect_equal(df$cats[[3]], c(4L, 5L))
  expect_null(df$cats[[4]])
})

test_that("btreeset — split NvNr: cats partition has cats list-column, no_cats does not", {
  res <- btreeset_split_nvnr()
  expect_setequal(names(res), c("cats", "no_cats"))
  expect_equal(nrow(res$cats), 2)
  expect_equal(nrow(res$no_cats), 2)
  expect_true("cats" %in% names(res$cats))
  expect_false("cats" %in% names(res$no_cats))
  expect_equal(res$cats$cats[[1]], c(1L, 2L, 3L))
  expect_equal(res$cats$cats[[2]], c(4L, 5L))
})

# ── Vec<T> width = N ─────────────────────────────────────────────────────────

test_that("vec width — split 1v1r emits pinned scores_1..scores_3", {
  res <- vec_width_split_1v1r()
  expect_setequal(names(res), c("score", "no_score"))
  expect_split_partition(res$score, 1, c("label", "scores_1", "scores_2", "scores_3"))
  expect_split_partition(res$no_score, 0, "label")
  expect_equal(res$score$scores_1, 1.0)
  expect_equal(res$score$scores_2, 2.0)
  expect_equal(res$score$scores_3, 3.0)
})

test_that("vec width — split 1vNr trailing NAs from short / empty rows", {
  res <- vec_width_split_1vnr()
  expect_split_partition(res$score, 3, c("label", "scores_1", "scores_2", "scores_3"))
  expect_equal(res$score$scores_1, c(1.0, 4.0, NA))
  expect_equal(res$score$scores_2, c(2.0, NA, NA))
  expect_equal(res$score$scores_3, c(3.0, NA, NA))
})

test_that("vec width — split Nv1r drops non-payload columns from no-score partition", {
  res <- vec_width_split_nv1r()
  expect_split_partition(res$score, 1, c("label", "scores_1", "scores_2", "scores_3"))
  expect_split_partition(res$no_score, 1, "label")
  expect_equal(res$no_score$label, "b")
})

test_that("vec width — split NvNr partitions multi-row groups", {
  res <- vec_width_split_nvnr()
  expect_split_partition(res$score, 2, c("label", "scores_1", "scores_2", "scores_3"))
  expect_split_partition(res$no_score, 2, "label")
  expect_equal(res$score$label, c("a", "c"))
  expect_equal(res$score$scores_1, c(1.0, 4.0))
  expect_equal(res$score$scores_3, c(3.0, NA))
})

test_that("vec width — align NvNr fills both NoScore rows and short payloads with NA", {
  df <- vec_width_align_nvnr()
  expect_equal(df$`_type`, c("Score", "NoScore", "Score"))
  expect_equal(df$scores_1, c(1.0, NA, 4.0))
  expect_equal(df$scores_2, c(2.0, NA, NA))
  expect_equal(df$scores_3, c(3.0, NA, NA))
  expect_equal(df$label, c("a", "b", "c"))
})

# ── 3. Vec<T> expand (auto-expand) ───────────────────────────────────────────

test_that("vec expand — split 1v1r emits runtime-sized vals_* columns", {
  res <- vec_expand_split_1v1r()
  expect_setequal(names(res), c("vals", "no_vals"))
  expect_split_partition(res$vals, 1, c("label", "vals_1", "vals_2"))
  expect_split_partition(res$no_vals, 0, "label")
  expect_equal(res$vals$vals_1, 1.0)
  expect_equal(res$vals$vals_2, 2.0)
})

test_that("vec expand — split 1vNr widens to longest row, fills shorter with NA", {
  res <- vec_expand_split_1vnr()
  expect_split_partition(res$vals, 3, c("label", "vals_1", "vals_2"))
  expect_equal(res$vals$vals_1, c(1.0, 3.0, NA))
  expect_equal(res$vals$vals_2, c(2.0, NA, NA))
})

test_that("vec expand — split Nv1r non-payload partition has only label", {
  res <- vec_expand_split_nv1r()
  expect_split_partition(res$vals, 1, c("label", "vals_1", "vals_2"))
  expect_split_partition(res$no_vals, 1, "label")
  expect_equal(res$no_vals$label, "b")
})

test_that("vec expand — split NvNr partitions multi-row groups", {
  res <- vec_expand_split_nvnr()
  expect_split_partition(res$vals, 2, c("label", "vals_1", "vals_2"))
  expect_split_partition(res$no_vals, 2, "label")
  expect_equal(res$vals$vals_1, c(1.0, 3.0))
  expect_equal(res$vals$vals_2, c(2.0, NA))
})

test_that("vec expand — align NvNr columns emit vals_1, vals_2 with NA for NoVals rows", {
  df <- vec_expand_align_nvnr()
  expect_equal(df$`_type`, c("Vals", "NoVals", "Vals"))
  expect_equal(df$label, c("a", "b", "c"))
  expect_equal(df$vals_1, c(1.0, NA, 3.0))
  expect_equal(df$vals_2, c(2.0, NA, NA))
})

# ── 4. [T; N] auto-expand ─────────────────────────────────────────────────────

test_that("array — split 1v1r expands [f64; 2] into coords_1, coords_2", {
  res <- array_split_1v1r()
  expect_setequal(names(res), c("coords", "no_coords"))
  expect_split_partition(res$coords, 1, c("id", "coords_1", "coords_2"))
  expect_split_partition(res$no_coords, 0, "id")
  expect_equal(res$coords$coords_1, 10.0)
  expect_equal(res$coords$coords_2, 20.0)
})

test_that("array — split 1vNr stacks rows under one variant", {
  res <- array_split_1vnr()
  expect_split_partition(res$coords, 2, c("id", "coords_1", "coords_2"))
  expect_split_partition(res$no_coords, 0, "id")
  expect_equal(res$coords$coords_1, c(10.0, 30.0))
  expect_equal(res$coords$coords_2, c(20.0, 40.0))
})

test_that("array — split Nv1r non-payload partition keeps only shared scalar", {
  res <- array_split_nv1r()
  expect_split_partition(res$coords, 1, c("id", "coords_1", "coords_2"))
  expect_split_partition(res$no_coords, 1, "id")
  expect_equal(res$no_coords$id, 2L)
})

test_that("array — split NvNr partitions multi-row groups", {
  res <- array_split_nvnr()
  expect_split_partition(res$coords, 2, c("id", "coords_1", "coords_2"))
  expect_split_partition(res$no_coords, 2, "id")
  expect_equal(res$coords$coords_1, c(10.0, 30.0))
  expect_equal(res$no_coords$id, c(2L, 4L))
})

test_that("array — align NvNr fills array columns with NA for NoCoords rows", {
  df <- array_align_nvnr()
  expect_equal(df$`_type`, c("Coords", "NoCoords", "Coords"))
  expect_equal(df$coords_1, c(10.0, NA, 30.0))
  expect_equal(df$coords_2, c(20.0, NA, 40.0))
  expect_equal(df$id, c(1L, 2L, 3L))
})

# ── 5. Box<[T]> with expand ──────────────────────────────────────────────────

test_that("boxed slice expand — split 1v1r emits runtime-sized data_* columns", {
  res <- boxed_slice_split_1v1r()
  expect_setequal(names(res), c("buffer", "no_buffer"))
  expect_split_partition(res$buffer, 1, c("name", "data_1", "data_2", "data_3"))
  expect_split_partition(res$no_buffer, 0, "name")
  expect_equal(res$buffer$data_1, 1.0)
  expect_equal(res$buffer$data_2, 2.0)
  expect_equal(res$buffer$data_3, 3.0)
})

test_that("boxed slice expand — split 1vNr widens to longest, NA-fills shorter", {
  res <- boxed_slice_split_1vnr()
  expect_split_partition(res$buffer, 3, c("name", "data_1", "data_2", "data_3"))
  expect_equal(res$buffer$data_1, c(1.0, 4.0, NA))
  expect_equal(res$buffer$data_2, c(2.0, NA, NA))
  expect_equal(res$buffer$data_3, c(3.0, NA, NA))
})

test_that("boxed slice expand — split Nv1r non-payload partition has only name", {
  res <- boxed_slice_split_nv1r()
  expect_split_partition(res$buffer, 1, c("name", "data_1", "data_2", "data_3"))
  expect_split_partition(res$no_buffer, 1, "name")
  expect_equal(res$no_buffer$name, "b")
})

test_that("boxed slice expand — split NvNr partitions multi-row groups", {
  res <- boxed_slice_split_nvnr()
  expect_split_partition(res$buffer, 2, c("name", "data_1", "data_2", "data_3"))
  expect_split_partition(res$no_buffer, 2, "name")
  expect_equal(res$buffer$name, c("a", "c"))
  expect_equal(res$buffer$data_1, c(1.0, 4.0))
})

test_that("boxed slice expand — align NvNr fills NA for NoBuffer rows", {
  df <- boxed_slice_align_nvnr()
  expect_equal(df$`_type`, c("Buffer", "NoBuffer", "Buffer"))
  expect_equal(df$name, c("a", "b", "c"))
  expect_equal(df$data_1, c(1.0, NA, 4.0))
  expect_equal(df$data_2, c(2.0, NA, NA))
  expect_equal(df$data_3, c(3.0, NA, NA))
})

# ── Single-variant enum returns a bare data.frame from split ─────────────────

test_that("singleton — split 1v1r returns a bare 1-row data.frame, not a list", {
  res <- singleton_split_1v1r()
  expect_s3_class(res, "data.frame")
  expect_equal(nrow(res), 1)
  expect_equal(sort(names(res)), sort(c("id", "label")))
  expect_equal(res$id, 1L)
  expect_equal(res$label, "alpha")
})

test_that("singleton — split 1vNr returns a bare N-row data.frame", {
  res <- singleton_split_1vnr()
  expect_s3_class(res, "data.frame")
  expect_equal(nrow(res), 3)
  expect_equal(res$id, c(1L, 2L, 3L))
  expect_equal(res$label, c("alpha", "beta", "gamma"))
})
