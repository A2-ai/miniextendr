# Tests for #485 — struct-in-struct flattening on top-level `DataFrameRow`.
#
# Mirrors the enum-variant struct-field flattening tests in
# test-dataframe-enum-payload-matrix.R (the #477 follow-up).
#
# DO NOT relax these assertions to make a partial fix pass. Talk to the author
# first — column names and column ordering are part of the public contract.

# region: basic — 1 row -------------------------------------------------------

test_that("struct-in-struct flatten — 1 row: prefixed inner columns", {
  df <- flat_basic_1row()
  expect_s3_class(df, "data.frame")
  expect_identical(colnames(df), c("id", "origin_x", "origin_y"))
  expect_equal(nrow(df), 1L)
  expect_equal(df$id, 1L)
  expect_equal(df$origin_x, 1.0)
  expect_equal(df$origin_y, 2.0)
})

# region: basic — N rows ------------------------------------------------------

test_that("struct-in-struct flatten — N rows: values align by row", {
  df <- flat_basic_nrow()
  expect_identical(colnames(df), c("id", "origin_x", "origin_y"))
  expect_equal(nrow(df), 3L)
  expect_equal(df$id, c(1L, 2L, 3L))
  expect_equal(df$origin_x, c(1.0, 3.0, 5.0))
  expect_equal(df$origin_y, c(2.0, 4.0, 6.0))
})

# region: basic — 0 rows ------------------------------------------------------

test_that("struct-in-struct flatten — 0 rows: column shape preserved", {
  df <- flat_basic_zero_rows()
  expect_s3_class(df, "data.frame")
  expect_identical(colnames(df), c("id", "origin_x", "origin_y"))
  expect_equal(nrow(df), 0L)
  expect_equal(length(df$id), 0L)
  expect_equal(length(df$origin_x), 0L)
  expect_equal(length(df$origin_y), 0L)
})

# region: multiple struct fields ---------------------------------------------

test_that("struct-in-struct flatten — two struct fields on the same outer", {
  df <- flat_two_struct_fields()
  expect_identical(
    colnames(df),
    c("id", "a_x", "a_y", "b_x", "b_y")
  )
  expect_equal(nrow(df), 2L)
  expect_equal(df$id, c(10L, 20L))
  expect_equal(df$a_x, c(1.0, 5.0))
  expect_equal(df$a_y, c(2.0, 6.0))
  expect_equal(df$b_x, c(3.0, 7.0))
  expect_equal(df$b_y, c(4.0, 8.0))
})

# region: mixed inner column types -------------------------------------------

test_that("struct-in-struct flatten — inner has mixed scalar types", {
  df <- flat_mixed_inner_types()
  expect_identical(colnames(df), c("id", "owner_name", "owner_age"))
  expect_equal(nrow(df), 2L)
  expect_equal(df$id, c(1L, 2L))
  expect_equal(df$owner_name, c("Ada", "Linus"))
  expect_equal(df$owner_age, c(30L, 50L))
})

# region: rename --------------------------------------------------------------

test_that("struct-in-struct flatten — `rename` controls the column prefix", {
  df <- flat_renamed_inner()
  # Should be `loc_x`/`loc_y`, NOT `origin_x`/`origin_y`.
  expect_identical(colnames(df), c("id", "loc_x", "loc_y"))
  expect_false("origin_x" %in% colnames(df))
  expect_false("origin_y" %in% colnames(df))
  expect_equal(df$loc_x, 1.0)
  expect_equal(df$loc_y, 2.0)
})

# region: skip ----------------------------------------------------------------

test_that("struct-in-struct flatten — `skip` drops the struct field entirely", {
  df <- flat_skip_inner()
  expect_identical(colnames(df), "id")
  expect_false("origin" %in% colnames(df))
  expect_false("origin_x" %in% colnames(df))
  expect_false("origin_y" %in% colnames(df))
  expect_equal(df$id, c(1L, 2L))
})

# region: as_list opt-out -----------------------------------------------------

test_that("struct-in-struct flatten — `as_list` keeps the opaque list-column", {
  df <- flat_as_list_inner()
  # `as_list` must continue to suppress flattening — this is the documented
  # workaround in the #485 issue body. Column name is the field name (`origin`),
  # not prefixed.
  expect_identical(colnames(df), c("id", "origin"))
  expect_false("origin_x" %in% colnames(df))
  expect_false("origin_y" %in% colnames(df))
  expect_true(is.list(df$origin))
  expect_equal(length(df$origin), 2L)
  # Each list cell is the R rep of FlatPoint (from IntoList derive).
  expect_equal(df$origin[[1L]]$x, 1.0)
  expect_equal(df$origin[[1L]]$y, 2.0)
  expect_equal(df$origin[[2L]]$x, 3.0)
  expect_equal(df$origin[[2L]]$y, 4.0)
})

# region: column ordering ----------------------------------------------------

test_that("struct-in-struct flatten — scalar/struct/scalar order is preserved", {
  df <- flat_mixed_order()
  # Outer field order: id, p, label. Struct field `p` expands inline.
  expect_identical(colnames(df), c("id", "p_x", "p_y", "label"))
  expect_equal(df$id, c(1L, 2L))
  expect_equal(df$p_x, c(1.0, 3.0))
  expect_equal(df$p_y, c(2.0, 4.0))
  expect_equal(df$label, c("first", "second"))
})

# region: tuple struct outer -------------------------------------------------

test_that("struct-in-struct flatten — tuple-struct outer uses `_<i>` prefixes", {
  df <- flat_tuple_struct()
  expect_identical(colnames(df), c("_0_x", "_0_y", "_1_x", "_1_y"))
  expect_equal(nrow(df), 2L)
  expect_equal(df$`_0_x`, c(1.0, 5.0))
  expect_equal(df$`_0_y`, c(2.0, 6.0))
  expect_equal(df$`_1_x`, c(3.0, 7.0))
  expect_equal(df$`_1_y`, c(4.0, 8.0))
})

# region: nested struct-in-struct --------------------------------------------

test_that("struct-in-struct flatten — recurses through nested struct fields", {
  df <- flat_nested_struct()
  # FlatNested { id, inner: FlatInner { a, sub: FlatSubInner { depth } } }
  # → id, inner_a, inner_sub_depth (cascading flatten).
  expect_identical(colnames(df), c("id", "inner_a", "inner_sub_depth"))
  expect_equal(nrow(df), 2L)
  expect_equal(df$id, c(1L, 2L))
  expect_equal(df$inner_a, c(10.0, 20.0))
  expect_equal(df$inner_sub_depth, c(100.0, 200.0))
})

# region: gctorture -----------------------------------------------------------

test_that("struct-in-struct flatten — survives gctorture (basic)", {
  # The fixture itself returns a built data.frame. gctorture is the harness's
  # job in nightly; here we at least verify the SEXP shape is sane under
  # normal GC.
  res <- gc_stress_struct_flatten()
  expect_s3_class(res, "data.frame")
  expect_identical(colnames(res), c("id", "origin_x", "origin_y"))
  expect_equal(nrow(res), 32L)
  expect_equal(res$id, 0:31)
  expect_equal(res$origin_x, as.numeric(0:31))
  expect_equal(res$origin_y, as.numeric(0:31) * 2)
})

test_that("struct-in-struct flatten — survives gctorture (nested)", {
  res <- gc_stress_struct_flatten_nested()
  expect_s3_class(res, "data.frame")
  expect_identical(colnames(res), c("id", "inner_a", "inner_sub_depth"))
  expect_equal(nrow(res), 16L)
  expect_equal(res$id, 0:15)
  expect_equal(res$inner_a, as.numeric(0:15))
  expect_equal(res$inner_sub_depth, as.numeric(0:15) * 10)
})
