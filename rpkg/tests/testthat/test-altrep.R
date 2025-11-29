# ALTREP tests - compact integer sequences and optimizations

test_that("compact integer sequence basics work", {
  x <- rpkg:::altrep_compact_int(5L, 10L, 2L)  # 10, 12, 14, 16, 18
  expect_equal(length(x), 5L)
  expect_equal(x[1], 10L)
  expect_equal(x[5], 18L)
  expect_equal(sum(x), 70L)
  expect_equal(x, c(10L, 12L, 14L, 16L, 18L))
})

test_that("descending compact sequence works", {
  x <- rpkg:::altrep_compact_int(4L, 100L, -10L)  # 100, 90, 80, 70

expect_equal(length(x), 4L)
  expect_equal(x[1], 100L)
  expect_equal(x[4], 70L)
  expect_equal(sum(x), 340L)
})

test_that("real ALTREP from doubles works", {
  y <- rpkg:::altrep_from_doubles(c(1.5, 2.5, 3.5, NA_real_))
  expect_equal(length(y), 4L)
  expect_equal(y[1], 1.5)
  expect_equal(y[3], 3.5)
  expect_true(is.na(y[4]))
  expect_true(is.na(sum(y)))
  expect_equal(sum(y, na.rm = TRUE), 7.5)
})

test_that("string ALTREP works", {
  z <- rpkg:::altrep_from_strings(c("hello", NA_character_, "world", ""))
  expect_equal(length(z), 4L)
  expect_equal(z[1], "hello")
  expect_true(is.na(z[2]))
  expect_equal(z[3], "world")
  expect_equal(z[4], "")
})

test_that("logical ALTREP works", {
  l <- rpkg:::altrep_from_logicals(c(TRUE, FALSE, NA, TRUE))
  expect_equal(length(l), 4L)
  expect_true(l[1])
  expect_false(l[2])
  expect_true(is.na(l[3]))
  expect_equal(sum(l, na.rm = TRUE), 2L)
})

test_that("raw ALTREP works", {
  r <- rpkg:::altrep_from_raw(as.raw(c(0, 127, 255)))
  expect_equal(length(r), 3L)
  expect_equal(r[1], as.raw(0))
  expect_equal(r[2], as.raw(127))
  expect_equal(r[3], as.raw(255))
})

test_that("list ALTREP works", {
  w <- rpkg:::altrep_from_list(list(a = 1L, b = "two", c = 3.0))
  expect_equal(length(w), 3L)
  expect_equal(w[[1]], 1L)
  expect_equal(w[[2]], "two")
  expect_equal(w[[3]], 3.0)
})

test_that("empty vectors work", {
  empty_int <- rpkg:::altrep_compact_int(0L, 0L, 1L)
  expect_equal(length(empty_int), 0L)
  expect_equal(sum(empty_int), 0L)

  empty_real <- rpkg:::altrep_from_doubles(double(0))
  expect_equal(length(empty_real), 0L)

  empty_str <- rpkg:::altrep_from_strings(character(0))
  expect_equal(length(empty_str), 0L)
})

test_that("O(1) sum uses arithmetic series formula", {
  big_seq <- rpkg:::altrep_compact_int(1000000L, 1L, 1L)  # 1:1000000
  expected_sum <- 1000000 * 1000001 / 2
  expect_equal(sum(big_seq), expected_sum)
})

test_that("O(1) min/max work", {
  big_seq <- rpkg:::altrep_compact_int(1000000L, 1L, 1L)
  expect_equal(min(big_seq), 1L)
  expect_equal(max(big_seq), 1000000L)

  # Descending sequence
  desc_seq <- rpkg:::altrep_compact_int(100L, 100L, -1L)
  expect_equal(min(desc_seq), 1L)
  expect_equal(max(desc_seq), 100L)
})

test_that("serialization round-trip works", {
  compact_ser <- rpkg:::altrep_compact_int(100L, 1L, 1L)
  expect_equal(compact_ser, 1:100)

  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  saveRDS(compact_ser, tmp)
  reloaded <- readRDS(tmp)

  expect_equal(length(reloaded), 100L)
  expect_equal(reloaded, 1:100)
  expect_equal(sum(reloaded), 5050L)
})

test_that("large sequence serialization preserves compactness", {
  big_compact <- rpkg:::altrep_compact_int(1000000L, 1L, 1L)

  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  saveRDS(big_compact, tmp)
  big_reloaded <- readRDS(tmp)

  expect_equal(length(big_reloaded), 1000000L)
  expect_equal(sum(big_reloaded), 500000500000)  # O(1) sum should work
})

test_that("extract_subset optimization works for contiguous ranges", {
  big_seq <- rpkg:::altrep_compact_int(100L, 1L, 1L)
  subset_range <- big_seq[10:20]

  expect_equal(length(subset_range), 11L)
  expect_equal(subset_range[1], 10L)
  expect_equal(subset_range[11], 20L)
  expect_equal(sum(subset_range), sum(10:20))
})

test_that("extract_subset works on descending sequences", {
  desc_seq <- rpkg:::altrep_compact_int(50L, 100L, -2L)  # 100, 98, 96, ..., 2
  desc_subset <- desc_seq[10:20]

  expect_equal(length(desc_subset), 11L)
  expect_equal(desc_subset[1], 100L - 2L * 9L)  # 82
  expect_equal(desc_subset[11], 100L - 2L * 19L)  # 62
})

test_that("proc-macro ALTREP (ConstantIntClass) works", {
  c42 <- rpkg:::altrep_constant_int()
  expect_equal(length(c42), 10L)
  expect_true(all(c42 == 42L))
  expect_equal(sum(c42), 420L)
  expect_equal(mean(c42), 42)
})
