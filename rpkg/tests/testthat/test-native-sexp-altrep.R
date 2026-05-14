# Tests for native-SEXP AltrepExtract PoC
#
# Verifies that NativeSexpIntAltrep — an ALTREP integer vector whose backing
# data lives directly in data1 as a plain INTSXP (no ExternalPtr) — behaves
# identically to a regular integer vector from the R side.

test_that("native-SEXP AltrepExtract returns a working integer vector", {
  v <- native_sexp_altrep_new(c(10L, 20L, 30L))
  expect_identical(length(v), 3L)
  expect_identical(v[1L], 10L)
  expect_identical(v[2L], 20L)
  expect_identical(v[3L], 30L)
  expect_identical(as.integer(v), c(10L, 20L, 30L))
})

test_that("native-SEXP AltrepExtract roundtrips via serialize/unserialize", {
  v <- native_sexp_altrep_new(c(1L, 2L, 3L))
  raw <- serialize(v, NULL)
  w <- unserialize(raw)
  expect_identical(as.integer(w), c(1L, 2L, 3L))
})

test_that("native-SEXP AltrepExtract handles empty vector", {
  v <- native_sexp_altrep_new(integer(0))
  expect_identical(length(v), 0L)
  expect_identical(as.integer(v), integer(0))
})

test_that("native-SEXP AltrepExtract handles single element", {
  v <- native_sexp_altrep_new(42L)
  expect_identical(length(v), 1L)
  expect_identical(v[1L], 42L)
  expect_identical(as.integer(v), 42L)
})

test_that("native-SEXP AltrepExtract preserves NA_integer_", {
  v <- native_sexp_altrep_new(c(1L, NA_integer_, 3L))
  expect_identical(length(v), 3L)
  expect_identical(v[1L], 1L)
  expect_true(is.na(v[2L]))
  expect_identical(v[3L], 3L)
  expect_identical(as.integer(v), c(1L, NA_integer_, 3L))
})
