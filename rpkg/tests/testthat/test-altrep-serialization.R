# Test ALTREP serialization/deserialization (saveRDS/readRDS)
#
# These tests verify that ALTREP objects can be serialized and deserialized
# correctly. The AltrepSerialize trait implementations convert ALTREP data
# to native R vectors for serialization, ensuring data survives even if
# the Rust package isn't loaded when deserializing.

test_that("Vec<i32> ALTREP survives serialization round-trip", {
  original <- altrep_from_integers(c(10L, 20L, 30L, 40L, 50L))

  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(original, tmp)
  restored <- readRDS(tmp)

  expect_equal(restored, original)
  expect_equal(length(restored), 5L)
  expect_equal(sum(restored), 150L)
  expect_equal(restored[3], 30L)
})

test_that("Vec<f64> ALTREP survives serialization round-trip", {
  original <- altrep_from_doubles(c(1.5, 2.5, 3.5, 4.5))
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(original, tmp)
  restored <- readRDS(tmp)

  expect_equal(restored, original)
  expect_equal(length(restored), 4L)
  expect_equal(sum(restored), 12)
  expect_equal(restored[2], 2.5)
})

test_that("Vec<String> ALTREP survives serialization round-trip", {
  original <- altrep_from_strings(c("hello", "world", "test"))
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(original, tmp)
  restored <- readRDS(tmp)

  expect_equal(restored, original)
  expect_equal(length(restored), 3L)
  expect_equal(restored[1], "hello")
  expect_equal(restored[3], "test")
})

# Vec<bool> and Vec<u8> use iterator-backed ALTREP which doesn't have DATAPTR.
# They serialize via the Serialized_state ALTREP method.
# NOTE: Can't use expect_equal() with vector comparisons because testthat's
# waldo::compare → identical() triggers DATAPTR access. Use element-by-element only.
test_that("Vec<bool> ALTREP survives serialization round-trip", {
  original <- altrep_from_logicals(c(TRUE, FALSE, TRUE, FALSE, TRUE))

  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(original, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 5L)
  expect_equal(sum(restored), 3L)
  # Element-by-element checks only (no vector comparisons)
  expect_equal(restored[1], TRUE)
  expect_equal(restored[2], FALSE)
  expect_equal(restored[3], TRUE)
  expect_equal(restored[4], FALSE)
  expect_equal(restored[5], TRUE)
})

test_that("Vec<u8> (raw) ALTREP survives serialization round-trip", {
  original <- altrep_from_raw(as.raw(c(0x01, 0x02, 0x03, 0xFF)))

  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(original, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 4L)
  # Element-by-element checks only (no vector comparisons)
  expect_equal(restored[1], as.raw(0x01))
  expect_equal(restored[2], as.raw(0x02))
  expect_equal(restored[3], as.raw(0x03))
  expect_equal(restored[4], as.raw(0xFF))
})

test_that("Vec<Rcomplex> ALTREP survives serialization round-trip", {
  original <- vec_complex_altrep(5L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(original, tmp)
  restored <- readRDS(tmp)

  expect_equal(restored, original)
  expect_equal(length(restored), 5L)
  expect_equal(Re(restored[1]), 0)
  expect_equal(Im(restored[1]), 0)
  expect_equal(Re(restored[3]), 2)
  expect_equal(Im(restored[3]), -2)
})

test_that("List ALTREP survives serialization round-trip", {
  original <- integer_sequence_list(3L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(original, tmp)
  restored <- readRDS(tmp)

  expect_equal(restored, original)
  expect_equal(length(restored), 3L)
  expect_equal(restored[[1]], 1L)
  expect_equal(restored[[2]], c(1L, 2L))
  expect_equal(restored[[3]], c(1L, 2L, 3L))
})

test_that("Box<[i32]> ALTREP survives serialization round-trip", {
  original <- boxed_ints(4L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(original, tmp)
  restored <- readRDS(tmp)

  expect_equal(restored, original)
  expect_equal(length(restored), 4L)
})

test_that("Box<[f64]> ALTREP survives serialization round-trip", {
  original <- boxed_reals(5L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(original, tmp)
  restored <- readRDS(tmp)

  expect_equal(restored, original)
  expect_equal(length(restored), 5L)
  expect_equal(restored[1], 1.5)
  expect_equal(restored[5], 7.5)
})

# NOTE: Box<[bool]> and Box<[u8]> also don't have DATAPTR support.
# Use element-by-element checks only (no vector comparisons).
test_that("Box<[bool]> ALTREP survives serialization round-trip", {
  original <- boxed_logicals(5L)

  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(original, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 5L)
  # boxed_logicals generates alternating TRUE/FALSE pattern
  # Element-by-element checks only
  expect_equal(restored[1], TRUE)
  expect_equal(restored[2], FALSE)
  expect_equal(restored[3], TRUE)
  expect_equal(restored[4], FALSE)
  expect_equal(restored[5], TRUE)
})

test_that("Box<[u8]> ALTREP survives serialization round-trip", {
  original <- boxed_raw(5L)

  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(original, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 5L)
  # boxed_raw generates sequential bytes starting at 0
  # Element-by-element checks only
  expect_equal(restored[1], as.raw(0x00))
  expect_equal(restored[2], as.raw(0x01))
  expect_equal(restored[3], as.raw(0x02))
  expect_equal(restored[4], as.raw(0x03))
  expect_equal(restored[5], as.raw(0x04))
})

test_that("Box<[String]> ALTREP survives serialization round-trip", {
  original <- boxed_strings(3L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(original, tmp)
  restored <- readRDS(tmp)

  expect_equal(restored, original)
  expect_equal(length(restored), 3L)
  expect_equal(restored[1], "boxed_0")
})

test_that("Box<[Rcomplex]> ALTREP survives serialization round-trip", {
  original <- boxed_complex(4L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(original, tmp)
  restored <- readRDS(tmp)

  expect_equal(restored, original)
  expect_equal(length(restored), 4L)
  expect_equal(Re(restored[1]), 0.25)
  expect_equal(Im(restored[1]), 0.75)
})

# Range types don't have DATAPTR support by design (they're lazy).
# They serialize via the Serialized_state ALTREP method.
# NOTE: c() also triggers DATAPTR, so use element-by-element checks only.
test_that("Range<i32> ALTREP survives serialization round-trip", {
  original <- range_int_altrep(1L, 11L)  # 1..11 = [1,2,3,4,5,6,7,8,9,10]

  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(original, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 10L)
  expect_equal(sum(restored), 55L)
  # Element-by-element checks
  expect_equal(restored[1], 1L)
  expect_equal(restored[5], 5L)
  expect_equal(restored[10], 10L)
})

test_that("Range<i64> ALTREP survives serialization round-trip", {
  original <- range_i64_altrep(1, 6)  # 1..6 = [1,2,3,4,5]

  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(original, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 5L)
  # Element-by-element checks
  expect_equal(restored[1], 1L)
  expect_equal(restored[3], 3L)
  expect_equal(restored[5], 5L)
})

test_that("Range<f64> ALTREP survives serialization round-trip", {
  original <- range_real_altrep(0, 5)  # 0..5 = [0.0, 1.0, 2.0, 3.0, 4.0]

  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(original, tmp)
  restored <- readRDS(tmp)

  expect_equal(length(restored), 5L)
  expect_equal(sum(restored), 10.0)
  # Element-by-element checks
  expect_equal(restored[1], 0.0)
  expect_equal(restored[3], 2.0)
  expect_equal(restored[5], 4.0)
})

test_that("Empty ALTREP vectors survive serialization", {
  # Empty integer
  empty_int <- altrep_from_integers(integer(0))

  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(empty_int, tmp)
  restored <- readRDS(tmp)
  expect_equal(length(restored), 0L)
  expect_equal(restored, integer(0))

  # Empty strings
  empty_str <- altrep_from_strings(character(0))

  saveRDS(empty_str, tmp)
  restored <- readRDS(tmp)
  expect_equal(length(restored), 0L)
  expect_equal(restored, character(0))
})

test_that("Large ALTREP vectors survive serialization", {
  n <- 10000L

  # Large integer vector
  large_int <- boxed_ints(n)

  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(large_int, tmp)
  restored <- readRDS(tmp)
  expect_equal(length(restored), n)
  expect_equal(restored[1], 1L)
  expect_equal(restored[n], n)

  # Large real vector
  large_real <- boxed_reals(n)

  saveRDS(large_real, tmp)
  restored <- readRDS(tmp)
  expect_equal(length(restored), n)
  expect_equal(restored[1], 1.5)
})

test_that("ALTREP serialization works with compress options", {
  original <- altrep_from_integers(1:1000)
  tmp_xz <- tempfile(fileext = ".rds")
  tmp_gz <- tempfile(fileext = ".rds")
  tmp_bz <- tempfile(fileext = ".rds")
  on.exit(unlink(c(tmp_xz, tmp_gz, tmp_bz)))

  # Test different compression methods
  saveRDS(original, tmp_xz, compress = "xz")
  saveRDS(original, tmp_gz, compress = "gzip")
  saveRDS(original, tmp_bz, compress = "bzip2")

  restored_xz <- readRDS(tmp_xz)
  restored_gz <- readRDS(tmp_gz)
  restored_bz <- readRDS(tmp_bz)

  expect_equal(restored_xz, original)
  expect_equal(restored_gz, original)
  expect_equal(restored_bz, original)
})

test_that("ALTREP serialization preserves special values", {
  # Doubles with Inf, -Inf, NaN
  special_vals <- c(1.0, Inf, -Inf, NaN, 0.0)
  original <- altrep_from_doubles(special_vals)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp))

  saveRDS(original, tmp)
  restored <- readRDS(tmp)

  expect_equal(restored[1], 1.0)
  expect_true(is.infinite(restored[2]) && restored[2] > 0)
  expect_true(is.infinite(restored[3]) && restored[3] < 0)
  expect_true(is.nan(restored[4]))
  expect_equal(restored[5], 0.0)
})
