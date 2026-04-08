# ALTREP iterator/streaming serialization round-trip tests
#
# Issue #61: ALTREP types without explicit AltrepSerialize implementations
# still preserve data through saveRDS/readRDS. R's serialize falls through
# to default behavior (DATAPTR materialization) when serialized_state returns
# NULL. The data survives but the ALTREP class is lost — readRDS returns a
# plain vector.
#
# These tests verify that data is preserved for all iterator/streaming/
# computed ALTREP types that lack explicit serialization support.

# =============================================================================
# Streaming ALTREP types (StreamingIntRangeData, StreamingRealSquaresData)
# =============================================================================

test_that("streaming integer range ALTREP data survives saveRDS/readRDS", {
  v <- streaming_int_range(100L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  # Verify original values

  expect_equal(length(v), 100L)
  expect_equal(v[1], 1L)
  expect_equal(v[100], 100L)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  # Data preserved as plain vector
  expect_equal(length(restored), 100L)
  expect_equal(restored[1], 1L)
  expect_equal(restored[50], 50L)
  expect_equal(restored[100], 100L)
  expect_equal(sum(restored), sum(1:100))
})

test_that("streaming real squares ALTREP data survives saveRDS/readRDS", {
  v <- streaming_real_squares(10L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  expected <- (1:10)^2

  expect_equal(length(v), 10L)
  expect_equal(v[1], 1.0)
  expect_equal(v[10], 100.0)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 10L)
  for (i in 1:10) {
    expect_equal(restored[i], expected[i])
  }
  expect_equal(sum(restored), sum(expected))
})

# =============================================================================
# Constant ALTREP types (ConstantRealData without serialize)
# =============================================================================

test_that("constant real ALTREP data survives saveRDS/readRDS", {
  v <- constant_real()
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  expect_equal(length(v), 10L)
  expect_equal(v[1], pi)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 10L)
  for (i in 1:10) {
    expect_equal(restored[i], pi)
  }
})

# =============================================================================
# ArithSeq ALTREP (computed arithmetic sequence)
# =============================================================================

test_that("arith_seq ALTREP data survives saveRDS/readRDS", {
  v <- arith_seq(1.0, 0.5, 20L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  expected <- 1.0 + (0:19) * 0.5

  expect_equal(length(v), 20L)
  expect_equal(v[1], 1.0)
  expect_equal(v[20], 1.0 + 19 * 0.5)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 20L)
  for (i in 1:20) {
    expect_equal(restored[i], expected[i])
  }
})

# =============================================================================
# LazyString ALTREP (lazy string generation)
# =============================================================================

test_that("lazy_string ALTREP data survives saveRDS/readRDS", {
  v <- lazy_string("item_", 5L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  expect_equal(length(v), 5L)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  # LazyStringData.elt() returns None (NA) for all elements
  # (it's a demonstration stub), so the materialized result is all NAs
  expect_equal(length(restored), 5L)
  for (i in 1:5) {
    expect_true(is.na(restored[i]))
  }
})

# =============================================================================
# RepeatingRaw ALTREP (repeating byte pattern)
# =============================================================================

test_that("repeating_raw ALTREP data survives saveRDS/readRDS", {
  pattern <- as.raw(c(0x01, 0x02, 0x03))
  v <- repeating_raw(pattern, 9L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  expect_equal(length(v), 9L)
  expect_equal(v[1], as.raw(0x01))
  expect_equal(v[4], as.raw(0x01))  # pattern repeats

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 9L)
  expected <- rep(pattern, 3)
  for (i in 1:9) {
    expect_equal(restored[i], expected[i])
  }
})

# =============================================================================
# UnitCircle ALTREP (complex vector on unit circle)
# =============================================================================

test_that("unit_circle ALTREP data survives saveRDS/readRDS", {
  v <- unit_circle(4L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  expect_equal(length(v), 4L)
  # First point: e^(i*0) = 1+0i
  expect_equal(Re(v[1]), 1.0, tolerance = 1e-10)
  expect_equal(Im(v[1]), 0.0, tolerance = 1e-10)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 4L)
  # Check all 4 points: 1+0i, 0+1i, -1+0i, 0-1i
  expect_equal(Re(restored[1]), 1.0, tolerance = 1e-10)
  expect_equal(Im(restored[1]), 0.0, tolerance = 1e-10)
  expect_equal(Re(restored[2]), 0.0, tolerance = 1e-10)
  expect_equal(Im(restored[2]), 1.0, tolerance = 1e-10)
  expect_equal(Re(restored[3]), -1.0, tolerance = 1e-10)
  expect_equal(Im(restored[3]), 0.0, tolerance = 1e-10)
  expect_equal(Re(restored[4]), 0.0, tolerance = 1e-10)
  expect_equal(Im(restored[4]), -1.0, tolerance = 1e-10)
})

# =============================================================================
# IntegerSequenceList ALTREP (list where element i is 1:i)
# =============================================================================

test_that("integer_sequence_list ALTREP data survives saveRDS/readRDS", {
  v <- integer_sequence_list(5L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  expect_equal(length(v), 5L)
  expect_equal(v[[1]], 1L)
  expect_equal(v[[3]], 1:3)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 5L)
  for (i in 1:5) {
    expect_equal(restored[[i]], seq_len(i))
  }
})

# =============================================================================
# Sparse iterator ALTREP types
# =============================================================================

test_that("sparse integer iterator ALTREP data survives saveRDS/readRDS", {
  # Access elements sequentially so none are skipped (avoids NAs)
  v <- sparse_iter_int(1L, 11L)  # 1..10
  # Access all elements to cache them
  vals <- vapply(1:10, function(i) v[i], integer(1))
  expect_equal(vals, 1:10)

  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 10L)
  expect_equal(restored[1], 1L)
  expect_equal(restored[10], 10L)
  expect_equal(sum(restored), 55L)
})

test_that("sparse integer iterator with skipped elements preserves NAs", {
  v <- sparse_iter_int(1L, 11L)
  # Access element 5 first, skipping 1-4
  val5 <- v[5]
  expect_equal(val5, 5L)
  # Access remaining sequentially
  for (i in 6:10) v[i]

  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 10L)
  # Elements 1-4 were skipped -> NA
  for (i in 1:4) {
    expect_true(is.na(restored[i]))
  }
  # Elements 5-10 preserved
  expect_equal(restored[5], 5L)
  expect_equal(restored[10], 10L)
})

test_that("sparse real iterator ALTREP data survives saveRDS/readRDS", {
  v <- sparse_iter_real(0, 1, 5L)  # 0, 1, 2, 3, 4
  # Access sequentially
  for (i in 1:5) v[i]

  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 5L)
  expect_equal(restored[1], 0)
  expect_equal(restored[3], 2)
  expect_equal(restored[5], 4)
})

test_that("sparse logical iterator ALTREP data survives saveRDS/readRDS", {
  v <- sparse_iter_logical(6L)  # TRUE, FALSE, TRUE, FALSE, TRUE, FALSE
  # Access sequentially
  for (i in 1:6) v[i]

  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 6L)
  expect_true(restored[1])
  expect_false(restored[2])
  expect_true(restored[3])
  expect_false(restored[4])
})

test_that("sparse raw iterator ALTREP data survives saveRDS/readRDS", {
  v <- sparse_iter_raw(10L)  # 0, 1, 2, ..., 9
  # Access sequentially
  for (i in 1:10) v[i]

  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 10L)
  expect_equal(restored[1], as.raw(0))
  expect_equal(restored[5], as.raw(4))
  expect_equal(restored[10], as.raw(9))
})

# =============================================================================
# Edge cases
# =============================================================================

test_that("empty streaming ALTREP survives saveRDS/readRDS", {
  v <- streaming_int_range(0L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 0L)
})

test_that("large streaming ALTREP survives saveRDS/readRDS", {
  n <- 10000L
  v <- streaming_int_range(n)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), n)
  expect_equal(restored[1], 1L)
  expect_equal(restored[n], n)
  expect_equal(sum(restored), as.double(n) * (n + 1) / 2)
})
