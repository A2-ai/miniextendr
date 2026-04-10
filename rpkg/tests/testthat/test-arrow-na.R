# Tests for NA handling in zero-copy Arrow ↔ R conversions.
#
# R stores NAs as sentinel values (NA_integer_ = -2147483648, NA_real_ = specific NaN).
# Arrow stores nulls as a separate validity bitmask, leaving data buffer untouched.
# These tests verify correctness across roundtrips, mutation, computation,
# serialization, ALTREP, and cross-session scenarios.

# region: Float64Array NA roundtrip

test_that("f64 all-NA vector survives Arrow roundtrip", {
  v <- c(NA_real_, NA_real_, NA_real_)
  result <- arrow_na_f64_roundtrip(v)
  expect_equal(arrow_na_f64_null_count(v), 3L)
  expect_true(all(is.na(result)))
  expect_equal(length(result), 3L)
})

test_that("f64 no-NA vector has null_count 0", {
  v <- c(1.0, 2.0, 3.0)
  expect_equal(arrow_na_f64_null_count(v), 0L)
  expect_equal(arrow_na_f64_roundtrip(v), v)
})

test_that("f64 single NA element", {
  expect_equal(arrow_na_f64_null_count(NA_real_), 1L)
  expect_true(is.na(arrow_na_f64_roundtrip(NA_real_)))
})

test_that("f64 alternating NA pattern", {
  v <- c(1.0, NA, 2.0, NA, 3.0)
  expect_equal(arrow_na_f64_null_count(v), 2L)
  result <- arrow_na_f64_roundtrip(v)
  expect_equal(result[1], 1.0)
  expect_true(is.na(result[2]))
  expect_equal(result[3], 2.0)
  expect_true(is.na(result[4]))
  expect_equal(result[5], 3.0)
})

test_that("f64 NA at boundaries (first and last)", {
  v <- c(NA, 1.0, 2.0, 3.0, NA)
  expect_equal(arrow_na_f64_null_count(v), 2L)
  result <- arrow_na_f64_roundtrip(v)
  expect_true(is.na(result[1]))
  expect_equal(result[2:4], c(1.0, 2.0, 3.0))
  expect_true(is.na(result[5]))
})

test_that("f64 null positions reported correctly", {
  v <- c(1.0, NA, 3.0, NA, 5.0)
  positions <- arrow_na_f64_null_positions(v)
  expect_equal(positions, c(FALSE, TRUE, FALSE, TRUE, FALSE))
})

test_that("f64 compact (remove NAs) works", {
  v <- c(1.0, NA, 3.0, NA, 5.0)
  result <- arrow_na_f64_compact(v)
  expect_equal(result, c(1.0, 3.0, 5.0))
})

test_that("f64 compact of all-NA returns empty", {
  v <- c(NA_real_, NA_real_, NA_real_)
  result <- arrow_na_f64_compact(v)
  expect_equal(result, numeric(0))
})

# endregion

# region: Float64Array NA computation

test_that("f64 Arrow computation preserves NA (null propagation)", {
  v <- c(1.0, NA, 3.0)
  result <- arrow_na_f64_add_one(v)
  expect_equal(result[1], 2.0)
  expect_true(is.na(result[2]))
  expect_equal(result[3], 4.0)
})

test_that("f64 Arrow computation on all-NA returns all-NA", {
  v <- c(NA_real_, NA_real_)
  result <- arrow_na_f64_add_one(v)
  expect_true(all(is.na(result)))
  expect_equal(length(result), 2L)
})

test_that("f64 double roundtrip preserves NA positions", {
  v <- c(1.0, NA, 3.0, NA, 5.0)
  result <- arrow_na_f64_double_roundtrip(v)
  # Values doubled (first computation)
  expect_equal(result[1], 2.0)
  expect_true(is.na(result[2]))
  expect_equal(result[3], 6.0)
  expect_true(is.na(result[4]))
  expect_equal(result[5], 10.0)
})

# endregion

# region: Int32Array NA roundtrip

test_that("i32 all-NA vector survives Arrow roundtrip", {
  v <- c(NA_integer_, NA_integer_, NA_integer_)
  result <- arrow_na_i32_roundtrip(v)
  expect_equal(arrow_na_i32_null_count(v), 3L)
  expect_true(all(is.na(result)))
  expect_equal(length(result), 3L)
})

test_that("i32 no-NA vector has null_count 0", {
  v <- c(1L, 2L, 3L)
  expect_equal(arrow_na_i32_null_count(v), 0L)
  expect_equal(arrow_na_i32_roundtrip(v), v)
})

test_that("i32 single NA element", {
  expect_equal(arrow_na_i32_null_count(NA_integer_), 1L)
  expect_true(is.na(arrow_na_i32_roundtrip(NA_integer_)))
})

test_that("i32 alternating NA pattern", {
  v <- c(1L, NA, 2L, NA, 3L)
  expect_equal(arrow_na_i32_null_count(v), 2L)
  result <- arrow_na_i32_roundtrip(v)
  expect_equal(result[1], 1L)
  expect_true(is.na(result[2]))
  expect_equal(result[3], 2L)
  expect_true(is.na(result[4]))
  expect_equal(result[5], 3L)
})

test_that("i32 null positions reported correctly", {
  v <- c(NA, 1L, NA, 2L)
  positions <- arrow_na_i32_null_positions(v)
  expect_equal(positions, c(TRUE, FALSE, TRUE, FALSE))
})

# endregion

# region: Int32Array NA computation

test_that("i32 Arrow computation preserves NA (null propagation)", {
  v <- c(1L, NA, 3L)
  result <- arrow_na_i32_add_ten(v)
  expect_equal(result[1], 11L)
  expect_true(is.na(result[2]))
  expect_equal(result[3], 13L)
})

test_that("i32 double roundtrip preserves NA positions", {
  v <- c(1L, NA, 3L)
  result <- arrow_na_i32_double_roundtrip(v)
  # Values tripled (first computation)
  expect_equal(result[1], 3L)
  expect_true(is.na(result[2]))
  expect_equal(result[3], 9L)
})

# endregion

# region: BooleanArray NA roundtrip

test_that("bool all-NA vector survives Arrow roundtrip", {
  v <- c(NA, NA, NA)
  expect_equal(arrow_na_bool_null_count(v), 3L)
  result <- arrow_na_bool_roundtrip(v)
  expect_true(all(is.na(result)))
})

test_that("bool mixed NA pattern", {
  v <- c(TRUE, NA, FALSE, NA, TRUE)
  expect_equal(arrow_na_bool_null_count(v), 2L)
  result <- arrow_na_bool_roundtrip(v)
  expect_true(result[1])
  expect_true(is.na(result[2]))
  expect_false(result[3])
  expect_true(is.na(result[4]))
  expect_true(result[5])
})

test_that("bool null positions reported correctly", {
  v <- c(TRUE, NA, FALSE)
  positions <- arrow_na_bool_null_positions(v)
  expect_equal(positions, c(FALSE, TRUE, FALSE))
})

# endregion

# region: StringArray NA roundtrip

test_that("string all-NA vector survives Arrow roundtrip", {
  v <- c(NA_character_, NA_character_, NA_character_)
  expect_equal(arrow_na_string_null_count(v), 3L)
  result <- arrow_na_string_roundtrip(v)
  expect_true(all(is.na(result)))
})

test_that("string mixed NA pattern", {
  v <- c("hello", NA, "world", NA)
  expect_equal(arrow_na_string_null_count(v), 2L)
  result <- arrow_na_string_roundtrip(v)
  expect_equal(result[1], "hello")
  expect_true(is.na(result[2]))
  expect_equal(result[3], "world")
  expect_true(is.na(result[4]))
})

test_that("string null positions reported correctly", {
  v <- c(NA, "a", NA, "b")
  positions <- arrow_na_string_null_positions(v)
  expect_equal(positions, c(TRUE, FALSE, TRUE, FALSE))
})

test_that("string with empty strings are not confused with NA", {
  v <- c("", NA, "", "a")
  expect_equal(arrow_na_string_null_count(v), 1L)
  result <- arrow_na_string_roundtrip(v)
  expect_equal(result[1], "")
  expect_true(is.na(result[2]))
  expect_equal(result[3], "")
  expect_equal(result[4], "a")
})

# endregion

# region: RecordBatch NA roundtrip

test_that("RecordBatch with all-NA columns roundtrips", {
  df <- data.frame(
    dbl = c(NA_real_, NA_real_),
    int = c(NA_integer_, NA_integer_),
    lgl = c(NA, NA),
    chr = c(NA_character_, NA_character_)
  )
  result <- arrow_na_recordbatch_roundtrip(df)
  expect_true(all(is.na(result$dbl)))
  expect_true(all(is.na(result$int)))
  expect_true(all(is.na(result$lgl)))
  expect_true(all(is.na(result$chr)))
})

test_that("RecordBatch null counts per column", {
  df <- data.frame(
    none = c(1.0, 2.0, 3.0),     # 0 NAs
    some = c(1L, NA, 3L),         # 1 NA
    all_na = c(NA, NA, NA)        # 3 NAs (logical)
  )
  counts <- arrow_na_recordbatch_null_counts(df)
  expect_equal(counts, c(0L, 1L, 3L))
})

test_that("RecordBatch mixed NA patterns across columns", {
  df <- data.frame(
    x = c(1.0, NA, 3.0, NA),
    y = c(NA, 2L, NA, 4L),
    z = c("a", NA, NA, "d")
  )
  result <- arrow_na_recordbatch_roundtrip(df)
  # Verify NA positions are independent per column
  expect_equal(result$x[1], 1.0)
  expect_true(is.na(result$x[2]))
  expect_true(is.na(result$y[1]))
  expect_equal(result$y[2], 2L)
  expect_equal(result$z[1], "a")
  expect_true(is.na(result$z[2]))
  expect_true(is.na(result$z[3]))
  expect_equal(result$z[4], "d")
})

# endregion

# region: R-side mutation after Arrow roundtrip

test_that("f64: R-side assignment after Arrow roundtrip works", {
  v <- c(1.0, NA, 3.0)
  result <- arrow_na_f64_roundtrip(v)
  # Mutate: set NA position to a value
  result[2] <- 99.0
  expect_equal(result, c(1.0, 99.0, 3.0))
  # Mutate: set a value position to NA
  result[1] <- NA
  expect_true(is.na(result[1]))
  expect_equal(result[2], 99.0)
})

test_that("i32: R-side assignment after Arrow roundtrip works", {
  v <- c(1L, NA, 3L)
  result <- arrow_na_i32_roundtrip(v)
  result[2] <- 42L
  expect_equal(result, c(1L, 42L, 3L))
  result[3] <- NA
  expect_true(is.na(result[3]))
})

test_that("bool: R-side assignment after Arrow roundtrip works", {
  v <- c(TRUE, NA, FALSE)
  result <- arrow_na_bool_roundtrip(v)
  result[2] <- TRUE
  expect_equal(result, c(TRUE, TRUE, FALSE))
  result[1] <- NA
  expect_true(is.na(result[1]))
})

test_that("string: R-side assignment after Arrow roundtrip works", {
  v <- c("hello", NA, "world")
  result <- arrow_na_string_roundtrip(v)
  result[2] <- "replaced"
  expect_equal(result, c("hello", "replaced", "world"))
  result[1] <- NA
  expect_true(is.na(result[1]))
})

# endregion

# region: Zero-copy identity with NAs

test_that("f64 zero-copy identity preserved even with NAs", {
  # The NA sentinels are in the data buffer but the buffer pointer
  # should still match for recovery
  x <- c(1.0, NA, 3.0)
  expect_true(arrow_na_f64_zero_copy_identity(x))
})

test_that("f64 zero-copy identity with all NAs", {
  x <- c(NA_real_, NA_real_, NA_real_)
  expect_true(arrow_na_f64_zero_copy_identity(x))
})

test_that("i32 zero-copy identity preserved even with NAs", {
  x <- c(1L, NA, 3L)
  expect_true(arrow_na_i32_zero_copy_identity(x))
})

test_that("i32 zero-copy identity with all NAs", {
  x <- c(NA_integer_, NA_integer_)
  expect_true(arrow_na_i32_zero_copy_identity(x))
})

# endregion

# region: Serialization (saveRDS/readRDS) with NAs — same session

test_that("f64 Arrow roundtrip result survives saveRDS/readRDS (same session)", {
  v <- c(1.0, NA, 3.0, NA, 5.0)
  result <- arrow_na_f64_roundtrip(v)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  saveRDS(result, tmp)
  loaded <- readRDS(tmp)
  expect_equal(loaded[1], 1.0)
  expect_true(is.na(loaded[2]))
  expect_equal(loaded[3], 3.0)
  expect_true(is.na(loaded[4]))
  expect_equal(loaded[5], 5.0)
})

test_that("i32 Arrow roundtrip result survives saveRDS/readRDS (same session)", {
  v <- c(NA, 2L, NA, 4L)
  result <- arrow_na_i32_roundtrip(v)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  saveRDS(result, tmp)
  loaded <- readRDS(tmp)
  expect_true(is.na(loaded[1]))
  expect_equal(loaded[2], 2L)
  expect_true(is.na(loaded[3]))
  expect_equal(loaded[4], 4L)
})

test_that("ALTREP f64 with NAs: same-session saveRDS/readRDS", {
  v <- c(1.0, NA, 3.0)
  altrep <- arrow_na_f64_altrep(v)
  expect_equal(altrep[1], 10.0)
  expect_true(is.na(altrep[2]))
  expect_equal(altrep[3], 30.0)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  saveRDS(altrep, tmp)
  loaded <- readRDS(tmp)
  expect_equal(loaded[1], 10.0)
  expect_true(is.na(loaded[2]))
  expect_equal(loaded[3], 30.0)
})

test_that("ALTREP i32 with NAs: same-session saveRDS/readRDS", {
  v <- c(NA, 2L, NA)
  altrep <- arrow_na_i32_altrep(v)
  expect_true(is.na(altrep[1]))
  expect_equal(altrep[2], 102L)
  expect_true(is.na(altrep[3]))
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  saveRDS(altrep, tmp)
  loaded <- readRDS(tmp)
  expect_true(is.na(loaded[1]))
  expect_equal(loaded[2], 102L)
  expect_true(is.na(loaded[3]))
})

test_that("ALTREP all-null f64: same-session saveRDS/readRDS", {
  altrep <- arrow_na_f64_all_null_altrep(3L)
  expect_true(all(is.na(altrep)))
  expect_equal(length(altrep), 3L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  saveRDS(altrep, tmp)
  loaded <- readRDS(tmp)
  expect_true(all(is.na(loaded)))
  expect_equal(length(loaded), 3L)
})

test_that("ALTREP all-null i32: same-session saveRDS/readRDS", {
  altrep <- arrow_na_i32_all_null_altrep(4L)
  expect_true(all(is.na(altrep)))
  expect_equal(length(altrep), 4L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  saveRDS(altrep, tmp)
  loaded <- readRDS(tmp)
  expect_true(all(is.na(loaded)))
  expect_equal(length(loaded), 4L)
})

# endregion

# region: Serialization double-roundtrip

test_that("f64 double save/load preserves NAs", {
  v <- c(NA, 1.0, NA, 2.0)
  result <- arrow_na_f64_roundtrip(v)
  tmp1 <- tempfile(fileext = ".rds")
  tmp2 <- tempfile(fileext = ".rds")
  on.exit(unlink(c(tmp1, tmp2)), add = TRUE)
  saveRDS(result, tmp1)
  loaded1 <- readRDS(tmp1)
  saveRDS(loaded1, tmp2)
  loaded2 <- readRDS(tmp2)
  expect_true(is.na(loaded2[1]))
  expect_equal(loaded2[2], 1.0)
  expect_true(is.na(loaded2[3]))
  expect_equal(loaded2[4], 2.0)
})

# endregion

# region: Cross-session (callr) with NAs

test_that("f64 ALTREP with NAs: cross-session readRDS (with package)", {
  skip_on_os("windows")
  v <- c(1.0, NA, 3.0, NA, 5.0)
  altrep <- arrow_na_f64_altrep(v)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  saveRDS(altrep, tmp)

  lib <- .libPaths()
  loaded <- callr::r(function(path, lib) {
    .libPaths(lib)
    library(miniextendr)
    readRDS(path)
  }, args = list(path = tmp, lib = lib))

  expect_equal(loaded[1], 10.0)
  expect_true(is.na(loaded[2]))
  expect_equal(loaded[3], 30.0)
  expect_true(is.na(loaded[4]))
  expect_equal(loaded[5], 50.0)
})

test_that("f64 ALTREP with NAs: cross-session readRDS (without package)", {
  skip_on_os("windows")
  v <- c(1.0, NA, 3.0)
  altrep <- arrow_na_f64_altrep(v)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  saveRDS(altrep, tmp)

  loaded <- callr::r(function(path) {
    readRDS(path)
  }, args = list(path = tmp))

  expect_equal(loaded[1], 10.0)
  expect_true(is.na(loaded[2]))
  expect_equal(loaded[3], 30.0)
})

test_that("i32 ALTREP with NAs: cross-session readRDS", {
  skip_on_os("windows")
  v <- c(NA, 2L, 3L)
  altrep <- arrow_na_i32_altrep(v)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  saveRDS(altrep, tmp)

  lib <- .libPaths()
  loaded <- callr::r(function(path, lib) {
    .libPaths(lib)
    library(miniextendr)
    readRDS(path)
  }, args = list(path = tmp, lib = lib))

  expect_true(is.na(loaded[1]))
  expect_equal(loaded[2], 102L)
  expect_equal(loaded[3], 103L)
})

test_that("f64 all-null ALTREP: cross-session readRDS", {
  skip_on_os("windows")
  altrep <- arrow_na_f64_all_null_altrep(5L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  saveRDS(altrep, tmp)

  lib <- .libPaths()
  loaded <- callr::r(function(path, lib) {
    .libPaths(lib)
    library(miniextendr)
    readRDS(path)
  }, args = list(path = tmp, lib = lib))

  expect_true(all(is.na(loaded)))
  expect_equal(length(loaded), 5L)
})

test_that("i32 all-null ALTREP: cross-session readRDS", {
  skip_on_os("windows")
  altrep <- arrow_na_i32_all_null_altrep(3L)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  saveRDS(altrep, tmp)

  lib <- .libPaths()
  loaded <- callr::r(function(path, lib) {
    .libPaths(lib)
    library(miniextendr)
    readRDS(path)
  }, args = list(path = tmp, lib = lib))

  expect_true(all(is.na(loaded)))
  expect_equal(length(loaded), 3L)
})

# endregion

# region: Cross-session roundtrip (Arrow compute + serialize + load + re-convert)

test_that("f64 cross-session: compute → serialize → load → re-convert to Arrow", {
  skip_on_os("windows")
  v <- c(1.0, NA, 3.0)
  # Compute in Arrow (multiply by 10), get ALTREP back
  altrep <- arrow_na_f64_altrep(v)
  tmp <- tempfile(fileext = ".rds")
  on.exit(unlink(tmp), add = TRUE)
  saveRDS(altrep, tmp)

  # In a fresh session: load, then pass through Arrow again
  lib <- .libPaths()
  result <- callr::r(function(path, lib) {
    .libPaths(lib)
    library(miniextendr)
    loaded <- readRDS(path)
    # Re-convert through Arrow (R → Arrow → compute → R)
    miniextendr:::arrow_na_f64_add_one(loaded)
  }, args = list(path = tmp, lib = lib))

  expect_equal(result[1], 11.0)   # 1.0 * 10 + 1
  expect_true(is.na(result[2]))   # NA stays NA
  expect_equal(result[3], 31.0)   # 3.0 * 10 + 1
})

# endregion

# region: R-side operations on ALTREP Arrow with NAs

test_that("ALTREP f64 with NAs: R sum/mean handle NAs", {
  v <- c(1.0, NA, 3.0)
  altrep <- arrow_na_f64_altrep(v)
  expect_true(is.na(sum(altrep)))
  expect_equal(sum(altrep, na.rm = TRUE), 40.0)  # 10 + 30
  expect_true(is.na(mean(altrep)))
  expect_equal(mean(altrep, na.rm = TRUE), 20.0)  # (10 + 30) / 2
})

test_that("ALTREP f64 with NAs: is.na works element-wise", {
  v <- c(1.0, NA, 3.0, NA)
  altrep <- arrow_na_f64_altrep(v)
  expect_equal(is.na(altrep), c(FALSE, TRUE, FALSE, TRUE))
})

test_that("ALTREP f64 with NAs: subsetting works", {
  v <- c(1.0, NA, 3.0, NA, 5.0)
  altrep <- arrow_na_f64_altrep(v)
  expect_equal(altrep[1], 10.0)
  expect_true(is.na(altrep[2]))
  expect_equal(altrep[c(1, 3, 5)], c(10.0, 30.0, 50.0))
})

test_that("ALTREP i32 with NAs: R sum/mean handle NAs", {
  v <- c(1L, NA, 3L)
  altrep <- arrow_na_i32_altrep(v)
  expect_true(is.na(sum(altrep)))
  expect_equal(sum(altrep, na.rm = TRUE), 204L)  # 101 + 103
})

# endregion
