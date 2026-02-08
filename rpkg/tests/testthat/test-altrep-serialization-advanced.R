# Advanced ALTREP serialization round-trip tests
#
# Tests for edge cases in ALTREP vector serialization that aren't covered
# by the main test-altrep-serialization.R file.

# =============================================================================
# NA values in ALTREP vectors through serialization
# =============================================================================

test_that("Vec<i32> ALTREP with NA survives saveRDS/readRDS", {
  # Create a vector with NA values via Rust
  v <- into_sexp_altrep(c(1L, NA_integer_, 3L, NA_integer_, 5L))
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(restored, c(1L, NA_integer_, 3L, NA_integer_, 5L))
  expect_true(is.na(restored[2]))
  expect_true(is.na(restored[4]))
})

test_that("Vec<String> ALTREP with NA survives saveRDS/readRDS", {
  v <- into_sexp_altrep(c("hello", NA_character_, "world"))
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(restored, c("hello", NA_character_, "world"))
  expect_true(is.na(restored[2]))
})

test_that("Vec<String> ALTREP empty vector roundtrips", {
  v <- into_sexp_altrep(character(0))
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(restored, character(0))
  expect_equal(length(restored), 0L)
})

test_that("Vec<String> ALTREP identical() works with materialized dataptr", {
  v <- into_sexp_altrep(c("a", "b", "c"))
  expect_equal(v, c("a", "b", "c"))
  # After accessing elements, the materialized cache should be consistent
  expect_equal(v[1], "a")
  expect_equal(v[3], "c")
})

# =============================================================================
# Single-element vector boundaries
# =============================================================================

test_that("single-element integer ALTREP serializes correctly", {
  v <- into_sexp_altrep(42L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(restored, 42L)
  expect_length(restored, 1)
})

test_that("single-element double ALTREP serializes correctly", {
  v <- into_sexp_altrep(3.14)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(restored, 3.14)
  expect_length(restored, 1)
})

test_that("single-element string ALTREP serializes correctly", {
  v <- into_sexp_altrep("solo")
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(restored, "solo")
  expect_length(restored, 1)
})

test_that("single NA integer ALTREP serializes correctly", {
  v <- into_sexp_altrep(NA_integer_)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_true(is.na(restored))
  expect_length(restored, 1)
})

# =============================================================================
# Special floating-point values
# =============================================================================

test_that("ALTREP vector with NaN serializes correctly", {
  v <- into_sexp_altrep(c(1.0, NaN, 3.0))
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(restored[1], 1.0)
  expect_true(is.nan(restored[2]))
  expect_equal(restored[3], 3.0)
})

test_that("ALTREP vector with NA_real_ serializes correctly", {
  v <- into_sexp_altrep(c(1.0, NA_real_, 3.0))
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)

  saveRDS(v, tmp)
  restored <- readRDS(tmp)

  expect_equal(restored[1], 1.0)
  expect_true(is.na(restored[2]))
  expect_equal(restored[3], 3.0)
})
