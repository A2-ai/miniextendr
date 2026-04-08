# Serialization tests for iterator/streaming ALTREP types (#61)
#
# Without explicit serialization support, iterator/streaming ALTREP types
# that lack DATAPTR error on saveRDS because R's default serialization
# calls DATAPTR() which has no fallback for ALTREP types without it.
#
# The serialize_materialize macro variant provides materializing serialization:
# serialized_state collects all elements into a plain R vector, and
# unserialize returns it as-is (ALTREP class is lost, data is preserved).

# ===========================================================================
# Streaming ALTREP
# ===========================================================================

test_that("streaming integer ALTREP survives saveRDS/readRDS", {
  v <- streaming_int_range(10L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 10L)
  expect_equal(restored[1], 1L)
  expect_equal(restored[5], 5L)
  expect_equal(restored[10], 10L)
  expect_equal(sum(restored), 55L)
})

test_that("streaming real ALTREP survives saveRDS/readRDS", {
  v <- streaming_real_squares(5L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 5L)
  expect_equal(restored[1], 1.0)
  expect_equal(restored[2], 4.0)
  expect_equal(restored[3], 9.0)
  expect_equal(restored[4], 16.0)
  expect_equal(restored[5], 25.0)
})

# ===========================================================================
# Sparse iterator ALTREP
# ===========================================================================

test_that("sparse integer iterator ALTREP survives saveRDS/readRDS", {
  v <- sparse_iter_int(1L, 11L)  # 1..11 = [1,2,...,10]
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 10L)
  expect_equal(restored[1], 1L)
  expect_equal(restored[5], 5L)
  expect_equal(restored[10], 10L)
  expect_equal(sum(restored), 55L)
})

test_that("sparse integer squares ALTREP survives saveRDS/readRDS", {
  v <- sparse_iter_int_squares(5L)  # [0, 1, 4, 9, 16]
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 5L)
  expect_equal(restored[1], 0L)
  expect_equal(restored[2], 1L)
  expect_equal(restored[3], 4L)
  expect_equal(restored[4], 9L)
  expect_equal(restored[5], 16L)
})

test_that("sparse real iterator ALTREP survives saveRDS/readRDS", {
  v <- sparse_iter_real(0.0, 0.5, 5L)  # [0.0, 0.5, 1.0, 1.5, 2.0]
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 5L)
  expect_equal(restored[1], 0.0)
  expect_equal(restored[2], 0.5)
  expect_equal(restored[3], 1.0)
  expect_equal(restored[5], 2.0)
})

test_that("sparse logical iterator ALTREP survives saveRDS/readRDS", {
  v <- sparse_iter_logical(5L)  # [TRUE, FALSE, TRUE, FALSE, TRUE]
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 5L)
  # Element-by-element checks (no vector comparisons for logical ALTREP)
  expect_equal(restored[1], TRUE)
  expect_equal(restored[2], FALSE)
  expect_equal(restored[3], TRUE)
  expect_equal(restored[4], FALSE)
  expect_equal(restored[5], TRUE)
})

test_that("sparse raw iterator ALTREP survives saveRDS/readRDS", {
  v <- sparse_iter_raw(5L)  # [0x00, 0x01, 0x02, 0x03, 0x04]
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 5L)
  # Element-by-element checks (no vector comparisons for raw ALTREP)
  expect_equal(restored[1], as.raw(0x00))
  expect_equal(restored[2], as.raw(0x01))
  expect_equal(restored[3], as.raw(0x02))
  expect_equal(restored[4], as.raw(0x03))
  expect_equal(restored[5], as.raw(0x04))
})

# ===========================================================================
# Constant/compute ALTREP types (also without DATAPTR)
# ===========================================================================

test_that("constant integer ALTREP survives saveRDS/readRDS", {
  v <- constant_int()  # 10 elements, all 42
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 10L)
  expect_equal(restored[1], 42L)
  expect_equal(restored[10], 42L)
  expect_equal(sum(restored), 420L)
})

test_that("constant real ALTREP survives saveRDS/readRDS", {
  v <- constant_real()  # 10 elements, all pi
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 10L)
  expect_equal(restored[1], pi)
  expect_equal(restored[10], pi)
})

test_that("arith_seq ALTREP survives saveRDS/readRDS", {
  v <- arith_seq(1.0, 0.5, 5L)  # [1.0, 1.5, 2.0, 2.5, 3.0]
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 5L)
  expect_equal(restored[1], 1.0)
  expect_equal(restored[2], 1.5)
  expect_equal(restored[5], 3.0)
})

# ===========================================================================
# Edge cases
# ===========================================================================

test_that("empty streaming integer ALTREP survives saveRDS/readRDS", {
  v <- streaming_int_range(0L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 0L)
})

test_that("large streaming integer ALTREP survives saveRDS/readRDS", {
  v <- streaming_int_range(10000L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 10000L)
  expect_equal(restored[1], 1L)
  expect_equal(restored[10000], 10000L)
  expect_equal(sum(restored), sum(1:10000))
})

test_that("deserialized iterator ALTREP is a plain vector (not ALTREP)", {
  v <- streaming_int_range(5L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  # After deserialization, the result should be a plain vector
  # (the ALTREP class is lost, which is expected)
  expect_equal(restored, 1:5)
})
