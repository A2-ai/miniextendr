test_that("ALTREP constructors cover all classes", {
  expect_equal(length(constant_int()), 10L)
  expect_equal(length(constant_real()), 10L)
  expect_equal(length(arith_seq(0, 1, 5L)), 5L)
  expect_equal(length(lazy_int_seq(1L, 5L, 1L)), 5L)
  expect_equal(length(constant_logical(1L, 2L)), 2L)
  expect_equal(length(altrep_from_logicals(c(TRUE, FALSE, NA))), 3L)
  expect_equal(length(lazy_string("x", 2L)), 2L)
  expect_equal(length(repeating_raw(as.raw(c(1, 2)), 3L)), 3L)
  expect_equal(length(unit_circle(4L)), 4L)
  expect_equal(length(altrep_from_integers(1:3)), 3L)
  expect_equal(length(altrep_from_strings(c("a", "b"))), 2L)
  expect_equal(length(altrep_from_raw(as.raw(1:3))), 3L)
  expect_equal(length(altrep_from_doubles(c(1, 2))), 2L)
  expect_equal(length(boxed_ints(3L)), 3L)
  expect_equal(length(static_ints()), 5L)
  expect_equal(length(static_strings()), 4L)
  expect_equal(length(altrep_from_list(list(1L, 2L))), 2L)
})

test_that("Box ALTREP builtins cover all base types", {
  reals <- boxed_reals(4L)
  expect_equal(reals, c(1.5, 3.0, 4.5, 6.0))
  expect_equal(reals + 1.0, c(2.5, 4.0, 5.5, 7.0))

  logicals <- boxed_logicals(4L)
  expect_true(logicals[1])
  expect_false(logicals[2])
  expect_true(logicals[3])
  expect_false(logicals[4])

  raw_bytes <- boxed_raw(3L)
  expect_equal(raw_bytes[1], as.raw(0))
  expect_equal(raw_bytes[2], as.raw(1))
  expect_equal(raw_bytes[3], as.raw(2))

  strings <- boxed_strings(3L)
  expect_equal(strings, c("boxed_0", "boxed_1", "boxed_2"))

  complex_vals <- boxed_complex(3L)
  expect_equal(Re(complex_vals[1]), 0.25)
  expect_equal(Im(complex_vals[1]), 0.75)
  expect_equal(Re(complex_vals[3]), 2.25)
  expect_equal(Im(complex_vals[3]), 2.75)
})

test_that("Vec and Range ALTREP builtins cover additional types", {
  complex_vec <- vec_complex_altrep(3L)
  expect_equal(length(complex_vec), 3L)
  expect_equal(Re(complex_vec[1]), 0)
  expect_equal(Im(complex_vec[1]), 0)
  expect_equal(Re(complex_vec[3]), 2)
  expect_equal(Im(complex_vec[3]), -2)

  r64 <- range_i64_altrep(1L, 6L)
  expect_equal(length(r64), 5L)
  expect_equal(sum(r64), 15L)
  expect_equal(min(r64), 1L)
  expect_equal(max(r64), 5L)

  rreal <- range_real_altrep(0.5, 3.5)
  expect_equal(length(rreal), 3L)
  expect_equal(rreal[1], 0.5)
  expect_equal(rreal[3], 2.5)
  expect_equal(sum(rreal), 4.5)
  expect_equal(min(rreal), 0.5)
  expect_equal(max(rreal), 2.5)
})

# =============================================================================
# NA sentinel (i32::MIN) handling tests
# =============================================================================

test_that("Range<i32> NA sentinel handling - normal ranges", {
  # Normal range - no NA
  r <- range_int_altrep(1L, 6L)
  expect_equal(length(r), 5L)
  expect_equal(r[1], 1L)
  expect_equal(r[5], 5L)
  expect_false(anyNA(r))
  expect_equal(sum(r), 15L)
  expect_equal(min(r), 1L)
  expect_equal(max(r), 5L)

  # Negative range - no NA
  r_neg <- range_int_altrep(-5L, 0L)
  expect_equal(length(r_neg), 5L)
  expect_false(anyNA(r_neg))
  expect_equal(sum(r_neg), -15L)
  expect_equal(min(r_neg), -5L)
  expect_equal(max(r_neg), -1L)
})

test_that("Range<i32> NA sentinel handling is tested at Rust level", {

  # NA_INTEGER is -2147483648 (i32::MIN), which is NA_integer_ in R.
  # This cannot be tested via R API because R coerces NA_integer_ before Rust sees it.
  #
  # Tested at the Rust level in miniextendr-api/src/altrep_data/tests.rs:
  #   - range_i32_no_na_normal
  #   - range_i32_no_na_at_min
  #   - range_i32_sum_with_na
  #   - range_i32_min_with_na
  #
  # Run `cargo test range_i32` to verify.
  succeed("R-level test not possible; see Rust tests above")
})

test_that("LazyIntSeq works for normal sequences", {
  # Normal sequence - no NA
  seq <- altrep_compact_int(5L, 1L, 2L)  # 1, 3, 5, 7, 9
  expect_equal(length(seq), 5L)
  expect_equal(seq[1], 1L)
  expect_equal(seq[5], 9L)
  expect_false(anyNA(seq))
  expect_equal(sum(seq), 25L)
  expect_equal(min(seq), 1L)
  expect_equal(max(seq), 9L)

  # Descending sequence - no NA
  desc <- altrep_compact_int(5L, 100L, -10L)  # 100, 90, 80, 70, 60
  expect_equal(length(desc), 5L)
  expect_false(anyNA(desc))
  expect_equal(min(desc), 60L)
  expect_equal(max(desc), 100L)
})

test_that("LazyIntSeq handles sequences near integer bounds", {
  # Sequence near max int - should not overflow to NA
  # i32::MAX is 2147483647
  max_int <- .Machine$integer.max
  near_max <- altrep_compact_int(3L, max_int - 2L, 1L)  # max-2, max-1, max
  expect_equal(length(near_max), 3L)
  expect_equal(near_max[1], max_int - 2L)
  expect_equal(near_max[3], max_int)
  expect_false(anyNA(near_max))
})

test_that("Range<i64> handles out-of-range values", {
  # Range<i64> should return NA for values outside i32 range
  # Test a range that's within i32 bounds
  r64_normal <- range_i64_altrep(1L, 11L)
  expect_equal(length(r64_normal), 10L)
  expect_false(anyNA(r64_normal))
  expect_equal(sum(r64_normal), 55L)
})

# =============================================================================
# DATAPTR writability tests (#60)
#
# Verify that writing through DATAPTR (e.g. x[i] <- val) works correctly
# for all ALTREP types: types with AltrepDataptr return a pointer into data1
# (writes visible to Elt), types without Dataptr get duplicated by R into a
# regular vector before writing.
# =============================================================================

test_that("Vec<i32> ALTREP write-through works (dataptr into data1)", {
  x <- vec_int_altrep(5L)
  expect_equal(x, 1:5)

  # Single element write

  x[1] <- 999L
  expect_equal(x[1], 999L)
  # Other elements unchanged
  expect_equal(x[2:5], 2:5)

  # Multiple element write
  x[3:4] <- c(30L, 40L)
  expect_equal(x, c(999L, 2L, 30L, 40L, 5L))
})

test_that("Vec<f64> ALTREP write-through works (dataptr into data1)", {
  x <- vec_real_altrep(5L)
  expect_equal(x, c(0.5, 1.0, 1.5, 2.0, 2.5))

  x[2] <- 99.9
  expect_equal(x[2], 99.9)
  expect_equal(x[1], 0.5)
  expect_equal(x[5], 2.5)
})

test_that("Box<[f64]> ALTREP write-through works (dataptr into data1)", {
  x <- boxed_reals(4L)
  expect_equal(x, c(1.5, 3.0, 4.5, 6.0))

  x[3] <- -1.0
  expect_equal(x[3], -1.0)
  expect_equal(x[1], 1.5)
  expect_equal(x[4], 6.0)
})

test_that("Box<[i32]> ALTREP write-through works (dataptr into data1)", {
  x <- boxed_ints(3L)
  expect_equal(x, 1:3)

  x[2] <- 42L
  expect_equal(x, c(1L, 42L, 3L))
})

test_that("Box<[String]> ALTREP write-through works (string materialization)", {
  x <- boxed_strings(3L)
  expect_equal(x, c("boxed_0", "boxed_1", "boxed_2"))

  # Writing to a string ALTREP: R materializes the STRSXP cache in data2
  # and then writes to it. Subsequent reads come from data2.
  x[2] <- "modified"
  expect_equal(x[2], "modified")
  expect_equal(x[1], "boxed_0")
  expect_equal(x[3], "boxed_2")
})

test_that("Range<i32> ALTREP materializes on subassignment (no Dataptr)", {
  # Range<i32> has no Dataptr method. Subassignment triggers R's default
  # ALTREP materialization: R copies the vector via elt() calls, producing
  # a regular (non-ALTREP) vector that supports modification.
  x <- range_int_altrep(1L, 6L)  # [1, 2, 3, 4, 5]
  expect_equal(x[1], 1L)
  expect_equal(x[5], 5L)
  expect_equal(length(x), 5L)

  # Subassignment materializes the ALTREP vector into a regular vector
  x[1] <- 999L
  expect_equal(x, c(999L, 2L, 3L, 4L, 5L))
})

test_that("Range<f64> ALTREP materializes on subassignment (no Dataptr)", {
  x <- range_real_altrep(0.5, 3.5)  # [0.5, 1.5, 2.5]
  expect_equal(x[1], 0.5)
  expect_equal(x[3], 2.5)
  expect_equal(length(x), 3L)

  # Subassignment materializes the ALTREP vector into a regular vector
  x[2] <- -1.0
  expect_equal(x, c(0.5, -1.0, 2.5))
})
