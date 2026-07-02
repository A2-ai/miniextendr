# Tests for scatter_column with CPLXSXP and RAWSXP columns (issue #902).
#
# These cover the CPLXSXP and RAWSXP branches of scatter_column, which were
# refactored in PR #831 but not previously tested end-to-end.
#
# scatter_column is exercised by the struct-flatten path in DataFrameRow-derived
# enums: ScatterRawEvent/ScatterComplexEvent carry a DataFrameRow inner struct
# (RawPayload / ComplexPayload) as a struct-flatten field.  When Empty rows are
# present the inner struct's pre-built RAWSXP / CPLXSXP columns are scattered
# back to the full row-count with 0x00 / NA_complex_ fill respectively.
#
# Column names are prefixed by the struct-flatten field name "data_":
#   data_byte  — RAWSXP
#   data_value — CPLXSXP
#
# Complex fixtures require the num-complex feature and are only executed when
# the feature is compiled in (checked via rpkg_enabled_features()).

has_num_complex <- function() {
  "num-complex" %in% miniextendr::rpkg_enabled_features()
}

# region: RAWSXP scatter (always runs)

test_that("scatter_raw_mixed: present rows round-trip, absent rows are as.raw(0)", {
  df <- scatter_raw_mixed()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 5L)
  expect_true(is.raw(df$data_byte))

  # Present rows: check byte values.
  expect_equal(df$data_byte[1], as.raw(0xFF))
  expect_equal(df$data_byte[3], as.raw(0x01))
  expect_equal(df$data_byte[5], as.raw(0x7F))

  # Absent rows: R raw has no NA — scatter fills with 0x00.
  expect_equal(df$data_byte[2], as.raw(0x00))
  expect_equal(df$data_byte[4], as.raw(0x00))
})

test_that("scatter_raw_all_present: all rows have correct raw byte values", {
  df <- scatter_raw_all_present()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3L)
  expect_true(is.raw(df$data_byte))

  expect_equal(df$data_byte[1], as.raw(0xAB))
  expect_equal(df$data_byte[2], as.raw(0xCD))
  expect_equal(df$data_byte[3], as.raw(0xEF))
})

test_that("scatter_raw_all_absent: all data_byte cells are as.raw(0)", {
  df <- scatter_raw_all_absent()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3L)
  expect_true(is.raw(df$data_byte))
  expect_true(all(df$data_byte == as.raw(0x00)))
})

test_that("gc_stress_scatter_raw runs without error", {
  expect_no_error(gc_stress_scatter_raw())
})

# endregion

# region: CPLXSXP scatter (requires num-complex feature)

test_that("scatter_complex_mixed: present rows round-trip, absent rows are NA_complex_", {
  skip_if_not(has_num_complex(), "num-complex feature not compiled in")

  df <- scatter_complex_mixed()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 5L)
  expect_true(is.complex(df$data_value))

  # Present rows: check Re and Im parts.
  expect_equal(Re(df$data_value[1]),  1.0)
  expect_equal(Im(df$data_value[1]),  2.0)
  expect_equal(Re(df$data_value[3]), -0.5)
  expect_equal(Im(df$data_value[3]),  0.5)
  expect_equal(Re(df$data_value[5]),  0.0)
  expect_equal(Im(df$data_value[5]),  0.0)

  # Absent rows: NA_complex_ — both parts are NA_real_.
  expect_true(is.na(df$data_value[2]))
  expect_true(is.na(df$data_value[4]))
  expect_true(is.na(Re(df$data_value[2])))
  expect_true(is.na(Im(df$data_value[2])))
})

test_that("scatter_complex_all_present: all rows have correct complex values", {
  skip_if_not(has_num_complex(), "num-complex feature not compiled in")

  df <- scatter_complex_all_present()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3L)
  expect_true(is.complex(df$data_value))
  expect_false(anyNA(df$data_value))

  expect_equal(Re(df$data_value[1]),  3.0)
  expect_equal(Im(df$data_value[1]),  4.0)
  expect_equal(Re(df$data_value[2]), -1.0)
  expect_equal(Im(df$data_value[2]), -1.0)
  expect_equal(Re(df$data_value[3]),  0.0)
  expect_equal(Im(df$data_value[3]),  1.0)
})

test_that("scatter_complex_all_absent: all data_value cells are NA_complex_", {
  skip_if_not(has_num_complex(), "num-complex feature not compiled in")

  df <- scatter_complex_all_absent()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3L)
  expect_true(is.complex(df$data_value))
  expect_true(all(is.na(df$data_value)))
})

test_that("gc_stress_scatter_complex runs without error", {
  skip_if_not(has_num_complex(), "num-complex feature not compiled in")

  expect_no_error(gc_stress_scatter_complex())
})

# endregion

# region: RawSlice<T> (raw_conversions prelude item, audit A7)

test_that("RawSlice<f64> produces tagged headerless bytes and reads back", {
  skip_if_missing_feature("raw_conversions")
  r <- rawslice_produce()
  expect_true(is.raw(r))
  expect_length(r, 24) # 3 doubles x 8 bytes, headerless
  expect_equal(rawslice_sum(r), 1.0 + 2.5 - 3.75)
})

test_that("RawSlice<f64> handles the empty slice", {
  skip_if_missing_feature("raw_conversions")
  expect_equal(rawslice_sum(raw(0)), 0)
})

test_that("RawSlice<i32> round-trips native-layout bytes", {
  skip_if_missing_feature("raw_conversions")
  rr <- writeBin(c(1L, 2L), raw())
  out <- rawslice_roundtrip_i32(rr)
  attributes(out) <- NULL # drop the mx_raw_type tag for comparison
  expect_identical(out, rr)
})

# endregion
