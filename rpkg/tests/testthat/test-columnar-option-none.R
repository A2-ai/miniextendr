# Tests for ColumnarDataFrame all-None column downgrade (#307).
#
# When every row has None for an Option<T> field the column lands as a logical
# NA vector (is.logical() && all(is.na())), not list(NULL, NULL, ...).

# region: All-None → logical NA column

test_that("Option<u64> all-None single row lands as logical NA", {
  df <- test_columnar_opt_u64_all_none_single()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 1L)
  expect_true(is.logical(df$stored))
  expect_true(all(is.na(df$stored)))
  # name column is unaffected
  expect_equal(df$name, "a")
})

test_that("Option<u64> all-None multi-row lands as logical NA", {
  df <- test_columnar_opt_u64_all_none_multi()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3L)
  expect_true(is.logical(df$stored))
  expect_true(all(is.na(df$stored)))
})

test_that("Option<String> all-None lands as logical NA", {
  df <- test_columnar_opt_string_all_none()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  expect_true(is.logical(df$label))
  expect_true(all(is.na(df$label)))
})

test_that("Option<bool> all-None lands as logical NA", {
  df <- test_columnar_opt_bool_all_none()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  expect_true(is.logical(df$flag))
  expect_true(all(is.na(df$flag)))
})

test_that("Option<UserStruct> all-None: column lands as single logical NA column", {
  df <- test_columnar_opt_user_struct_all_none()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  # When every row is None for an Option<Struct>, the probe never sees any
  # fields, so struct expansion never fires.  The whole field becomes a single
  # logical NA column under the field name ("point"), not per-subfield columns.
  expect_true("point" %in% names(df))
  expect_true(is.logical(df$point))
  expect_true(all(is.na(df$point)))
})

test_that("Option<HashMap> all-None lands as logical NA", {
  df <- test_columnar_opt_hashmap_all_none()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  expect_true(is.logical(df$attrs))
  expect_true(all(is.na(df$attrs)))
})

test_that("Option<Vec<u8>> all-None lands as logical NA", {
  df <- test_columnar_opt_bytes_all_none()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  expect_true(is.logical(df$data))
  expect_true(all(is.na(df$data)))
})

# endregion

# region: Mixed Some/None — no downgrade

test_that("Option<u64> mixed Some/None: numeric column with NA", {
  df <- test_columnar_opt_u64_mixed()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3L)
  expect_true(is.numeric(df$stored))
  expect_equal(df$stored, c(42, NA, 99))
})

test_that("Option<String> mixed Some/None: character column with NA", {
  df <- test_columnar_opt_string_mixed()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  expect_true(is.character(df$label))
  expect_equal(df$label, c("hello", NA_character_))
})

# endregion

# region: Vec<u8> with values stays a list column

test_that("Vec<u8> with values is still a list column", {
  df <- test_columnar_bytes_with_values()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  expect_true(is.list(df$data))
  # Each element is a raw or integer vector of the right length
  expect_equal(length(df$data[[1]]), 3L)
  expect_equal(length(df$data[[2]]), 2L)
})

# endregion

# region: Mixed columns — bytes stays list, opt-none downgrades

test_that("Vec<u8> column stays list; adjacent all-None Option<u64> downgrades", {
  df <- test_columnar_bytes_and_opt_none()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  # raw bytes column: still a list
  expect_true(is.list(df$raw))
  # optional stored column: downgraded to logical NA
  expect_true(is.logical(df$stored))
  expect_true(all(is.na(df$stored)))
})

# endregion

# region: Enum union

test_that("enum all-variant-A with x=None: x column is logical NA", {
  df <- test_columnar_enum_all_none()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  expect_true(is.logical(df$x))
  expect_true(all(is.na(df$x)))
})

test_that("enum with one variant-B x=Some(42): x column is a list (schema-discovery limit)", {
  # Schema-discovery limitation: when the first row is variant-A with x = None,
  # the probe calls serialize_none → column type stays Generic.  Subsequent rows
  # with x = Some(42) push a real SEXP into the Generic list buffer.  The
  # assembly-time all-None downgrade does NOT fire here (one entry is non-null),
  # so the column comes back as a list, not numeric.  Mixed-variant enum rows
  # where an optional field flips from None to Some are a known probe-order
  # limitation; users who need typed columns should put Some-valued rows first or
  # reassemble the column in R.
  df <- test_columnar_enum_some_flips_type()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  expect_true(is.list(df$x))
  expect_null(df$x[[1]])
  expect_equal(df$x[[2]], 42L)
})

# endregion

# region: Flatten with all-None inner field

test_that("serde(flatten) with all-None inner Option field: column is logical NA", {
  df <- test_columnar_flatten_all_none()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 2L)
  expect_equal(df$id, c(1L, 2L))
  # The flattened 'size' field is always None and is NOT skip_serializing_if,
  # so it appears in the schema and lands as logical NA.
  expect_true("size" %in% names(df))
  expect_true(is.logical(df$size))
  expect_true(all(is.na(df$size)))
  # The 'name' field should be character
  expect_equal(df$name, c("a", "b"))
})

# endregion

# region: R coercion smoke tests

test_that("logical NA column coerces to numeric when combined with numeric values", {
  df_none <- test_columnar_opt_u64_all_none_multi()
  df_some <- test_columnar_opt_u64_mixed()

  # bind_rows coerces the logical NA column to the numeric column type
  if (requireNamespace("dplyr", quietly = TRUE)) {
    combined <- dplyr::bind_rows(df_none, df_some)
    expect_true(is.numeric(combined$stored))
    # The three all-NA rows and one more NA from the mixed set
    expect_equal(sum(is.na(combined$stored)), 4L)
  } else {
    # base R fallback: c() coerces logical to numeric
    coerced <- c(df_none$stored, df_some$stored)
    expect_true(is.numeric(coerced))
    expect_equal(sum(is.na(coerced)), 4L)
  }
})

test_that("logical NA column coerces to character when combined with character values", {
  df_none <- test_columnar_opt_string_all_none()
  df_some <- test_columnar_opt_string_mixed()

  if (requireNamespace("dplyr", quietly = TRUE)) {
    combined <- dplyr::bind_rows(df_none, df_some)
    expect_true(is.character(combined$label))
    expect_equal(sum(is.na(combined$label)), 3L) # 2 all-None + 1 mixed-None
  } else {
    coerced <- c(df_none$label, df_some$label)
    expect_true(is.character(coerced))
    expect_equal(sum(is.na(coerced)), 3L)
  }
})

# endregion
