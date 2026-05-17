# Tests for Apache Arrow integration (zero-copy R <-> Arrow)

# region: Float64Array (zero-copy)

test_that("Float64Array roundtrip preserves values", {
  v <- c(1.5, 2.5, 3.5)
  expect_equal(arrow_f64_roundtrip(v), v)
})

test_that("Float64Array sum computes correctly", {
  expect_equal(arrow_f64_sum(c(1, 2, 3, 4)), 10)
})

test_that("Float64Array preserves length", {
  expect_equal(arrow_f64_len(c(1, 2, 3)), 3L)
})

test_that("Float64Array handles NA -> null", {
  v <- c(1.0, NA, 3.0)
  expect_equal(arrow_f64_null_count(v), 1L)
  result <- arrow_f64_roundtrip(v)
  expect_equal(result[1], 1.0)
  expect_true(is.na(result[2]))
  expect_equal(result[3], 3.0)
})

test_that("Float64Array empty roundtrip", {
  expect_equal(arrow_f64_empty_roundtrip(numeric(0)), numeric(0))
})

# endregion

# region: Int32Array (zero-copy)

test_that("Int32Array roundtrip preserves values", {
  v <- 1:5
  expect_equal(arrow_i32_roundtrip(v), v)
})

test_that("Int32Array sum computes correctly", {
  expect_equal(arrow_i32_sum(1:10), 55L)
})

test_that("Int32Array handles NA -> null", {
  v <- c(1L, NA, 3L)
  expect_equal(arrow_i32_null_count(v), 1L)
  result <- arrow_i32_roundtrip(v)
  expect_equal(result[1], 1L)
  expect_true(is.na(result[2]))
  expect_equal(result[3], 3L)
})

test_that("Int32Array empty roundtrip", {
  expect_equal(arrow_i32_empty_roundtrip(integer(0)), integer(0))
})

# endregion

# region: UInt8Array (zero-copy)

test_that("UInt8Array roundtrip preserves values", {
  v <- as.raw(c(0x01, 0x02, 0xFF))
  expect_equal(arrow_u8_roundtrip(v), v)
})

test_that("UInt8Array length correct", {
  expect_equal(arrow_u8_len(as.raw(1:5)), 5L)
})

# endregion

# region: BooleanArray (copy)

test_that("BooleanArray roundtrip preserves values", {
  v <- c(TRUE, FALSE, TRUE)
  expect_equal(arrow_bool_roundtrip(v), v)
})

test_that("BooleanArray handles NA -> null", {
  v <- c(TRUE, NA, FALSE)
  expect_equal(arrow_bool_null_count(v), 1L)
  result <- arrow_bool_roundtrip(v)
  expect_true(result[1])
  expect_true(is.na(result[2]))
  expect_false(result[3])
})

# endregion

# region: StringArray (copy)

test_that("StringArray roundtrip preserves values", {
  v <- c("hello", "world")
  expect_equal(arrow_string_roundtrip(v), v)
})

test_that("StringArray handles NA -> null", {
  v <- c("hello", NA, "world")
  expect_equal(arrow_string_null_count(v), 1L)
  result <- arrow_string_roundtrip(v)
  expect_equal(result[1], "hello")
  expect_true(is.na(result[2]))
  expect_equal(result[3], "world")
})

test_that("StringArray handles UTF-8", {
  v <- c("\u00e9", "\u00fc", "\U0001f600")
  expect_equal(arrow_string_roundtrip(v), v)
})

# endregion

# region: RecordBatch (data.frame)

test_that("RecordBatch roundtrip preserves data.frame", {
  df <- data.frame(x = c(1.0, 2.0, 3.0), y = c(10L, 20L, 30L))
  result <- arrow_recordbatch_roundtrip(df)
  expect_equal(result$x, df$x)
  expect_equal(result$y, df$y)
  expect_s3_class(result, "data.frame")
})

test_that("RecordBatch nrow/ncol correct", {
  df <- data.frame(a = 1:3, b = c(1.0, 2.0, 3.0), c = c("x", "y", "z"))
  expect_equal(arrow_recordbatch_nrow(df), 3L)
  expect_equal(arrow_recordbatch_ncol(df), 3L)
})

test_that("RecordBatch column names preserved", {
  df <- data.frame(alpha = 1.0, beta = 2L, gamma = "c")
  expect_equal(arrow_recordbatch_column_names(df), c("alpha", "beta", "gamma"))
})

test_that("RecordBatch handles mixed types", {
  df <- data.frame(
    dbl = c(1.0, NA),
    int = c(1L, NA),
    lgl = c(TRUE, NA),
    chr = c("a", NA)
  )
  result <- arrow_recordbatch_roundtrip(df)
  expect_equal(result$dbl[1], 1.0)
  expect_true(is.na(result$dbl[2]))
  expect_equal(result$int[1], 1L)
  expect_true(is.na(result$int[2]))
  expect_true(result$lgl[1])
  expect_true(is.na(result$lgl[2]))
  expect_equal(result$chr[1], "a")
  expect_true(is.na(result$chr[2]))
})

test_that("RecordBatch rejects ragged columns with offending name in error", {
  # Pass a named list (VECSXP) with columns of unequal lengths.
  # The pre-validation in TryFromSexp for RecordBatch should catch this and
  # name the offending column before RecordBatch::try_new sees it.
  ragged <- list(x = c(1.0, 2.0, 3.0), y = c(10L, 20L))
  expect_error(
    arrow_recordbatch_roundtrip(ragged),
    regexp = "'y'",
    fixed = FALSE
  )
})

# endregion

# region: ArrayRef (dynamic dispatch)

test_that("ArrayRef roundtrip works for numeric", {
  v <- c(1.0, 2.0, 3.0)
  expect_equal(arrow_arrayref_roundtrip(v), v)
})

test_that("ArrayRef roundtrip works for integer", {
  v <- 1:5
  expect_equal(arrow_arrayref_roundtrip(v), v)
})

test_that("ArrayRef roundtrip works for logical", {
  v <- c(TRUE, FALSE, TRUE)
  expect_equal(arrow_arrayref_roundtrip(v), v)
})

test_that("ArrayRef roundtrip works for character", {
  v <- c("hello", "world")
  expect_equal(arrow_arrayref_roundtrip(v), v)
})

test_that("ArrayRef length correct", {
  expect_equal(arrow_arrayref_len(c(1, 2, 3)), 3L)
  expect_equal(arrow_arrayref_len(1:5), 5L)
})

# endregion

# region: Factor (DictionaryArray)

test_that("Factor roundtrip preserves levels and values", {
  f <- factor(c("a", "b", "a", "c"))
  result <- arrow_factor_roundtrip(f)
  expect_s3_class(result, "factor")
  expect_equal(levels(result), levels(f))
  expect_equal(as.character(result), as.character(f))
})

test_that("Factor handles NA", {
  f <- factor(c("x", NA, "y"))
  result <- arrow_factor_roundtrip(f)
  expect_equal(as.character(result[1]), "x")
  expect_true(is.na(result[2]))
  expect_equal(as.character(result[3]), "y")
})

test_that("Factor length correct", {
  expect_equal(arrow_factor_len(factor(c("a", "b", "c"))), 3L)
})

# endregion

# region: Date (Date32Array)

test_that("Date roundtrip preserves values", {
  d <- as.Date(c("2024-01-01", "2024-06-15", "2024-12-31"))
  result <- arrow_date_roundtrip(d)
  expect_s3_class(result, "Date")
  expect_equal(result, d)
})

test_that("Date handles NA", {
  d <- as.Date(c("2024-01-01", NA, "2024-12-31"))
  result <- arrow_date_roundtrip(d)
  expect_equal(result[1], d[1])
  expect_true(is.na(result[2]))
  expect_equal(result[3], d[3])
})

test_that("Date length correct", {
  expect_equal(arrow_date_len(Sys.Date() + 0:4), 5L)
})

# endregion

# region: POSIXct (TimestampSecondArray)

test_that("POSIXct roundtrip preserves values (truncated to seconds)", {
  t <- as.POSIXct(c("2024-01-01 12:00:00", "2024-06-15 18:30:00"), tz = "UTC")
  result <- arrow_posixct_roundtrip(t)
  expect_s3_class(result, "POSIXct")
  # Timestamps are truncated to seconds
  expect_equal(as.numeric(result), as.numeric(t), tolerance = 1)
})

test_that("POSIXct handles NA", {
  t <- as.POSIXct(c("2024-01-01 00:00:00", NA), tz = "UTC")
  result <- arrow_posixct_roundtrip(t)
  expect_false(is.na(result[1]))
  expect_true(is.na(result[2]))
})

test_that("POSIXct length correct", {
  t <- as.POSIXct(c("2024-01-01", "2024-01-02", "2024-01-03"), tz = "UTC")
  expect_equal(arrow_posixct_len(t), 3L)
})

test_that("POSIXct preserves timezone", {
  t <- as.POSIXct("2024-01-01 12:00:00", tz = "America/New_York")
  result <- arrow_posixct_roundtrip(t)
  expect_equal(attr(result, "tzone"), "America/New_York")
})

# endregion

# region: RecordBatch with typed columns

test_that("RecordBatch handles factor columns", {
  df <- data.frame(name = factor(c("a", "b", "c")), value = c(1.0, 2.0, 3.0))
  result <- arrow_recordbatch_typed_roundtrip(df)
  expect_s3_class(result$name, "factor")
  expect_equal(as.character(result$name), c("a", "b", "c"))
  expect_equal(result$value, c(1.0, 2.0, 3.0))
})

test_that("RecordBatch handles Date columns", {
  df <- data.frame(
    date = as.Date(c("2024-01-01", "2024-06-15")),
    value = c(10L, 20L)
  )
  result <- arrow_recordbatch_typed_roundtrip(df)
  expect_s3_class(result$date, "Date")
  expect_equal(result$date, df$date)
  expect_equal(result$value, df$value)
})

# endregion

# region: Upstream example-derived fixtures

test_that("arrow_f64_filter_non_null removes nulls", {

  v <- c(1.0, NA, 3.0, NA, 5.0)
  result <- arrow_f64_filter_non_null(v)
  expect_equal(result, c(1.0, 3.0, 5.0))
})

test_that("arrow_f64_mean computes mean ignoring nulls", {

  v <- c(2.0, NA, 4.0, NA, 6.0)
  result <- arrow_f64_mean(v)
  expect_equal(result, 4.0)
})

test_that("arrow_f64_mean returns NA for empty input", {

  result <- arrow_f64_mean(numeric(0))
  expect_true(is.na(result))
})

test_that("arrow_arrayref_type_name returns type string", {

  result <- arrow_arrayref_type_name(c(1.0, 2.0))
  expect_type(result, "character")
  expect_true(nchar(result) > 0)
})

# endregion
