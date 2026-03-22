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
