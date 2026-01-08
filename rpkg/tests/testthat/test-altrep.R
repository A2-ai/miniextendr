# ALTREP tests for miniextendr
#
# These tests verify ALTREP functionality across all supported vector types
# and backing data structures (iterators, Vec, Range, static slices, Box<[T]>).

# =============================================================================
# Proc-macro ALTREP tests (ConstantInt)
# =============================================================================

test_that("proc-macro ALTREP (ConstantIntClass) works", {
  c42 <- constant_int()
  expect_equal(length(c42), 10L)
  # Use element access instead of all() which requires dataptr
  expect_equal(c42[1], 42L)
  expect_equal(c42[10], 42L)
  expect_equal(sum(c42), 420L)
  expect_equal(mean(c42), 42)
})

test_that("constant real ALTREP works", {
  cr <- constant_real()
  expect_equal(length(cr), 10L)
  expect_equal(cr[1], pi, tolerance = 1e-15)
  expect_equal(cr[10], pi, tolerance = 1e-15)
  expect_equal(sum(cr), 10 * pi, tolerance = 1e-14)
})

# =============================================================================
# Complex number ALTREP tests
# =============================================================================

test_that("complex ALTREP (UnitCircle) works", {
  circle <- unit_circle(4L)  # 4th roots of unity
  expect_equal(length(circle), 4L)

  # First root is 1+0i
  expect_equal(Re(circle[1]), 1.0, tolerance = 1e-10)
  expect_equal(Im(circle[1]), 0.0, tolerance = 1e-10)

  # Second root is 0+1i (i)
  expect_equal(Re(circle[2]), 0.0, tolerance = 1e-10)
  expect_equal(Im(circle[2]), 1.0, tolerance = 1e-10)

  # Third root is -1+0i
  expect_equal(Re(circle[3]), -1.0, tolerance = 1e-10)
  expect_equal(Im(circle[3]), 0.0, tolerance = 1e-10)

  # Fourth root is 0-1i (-i)
  expect_equal(Re(circle[4]), 0.0, tolerance = 1e-10)
  expect_equal(Im(circle[4]), -1.0, tolerance = 1e-10)

  # Check modulus of each root individually
  expect_equal(Mod(circle[1]), 1.0, tolerance = 1e-10)
  expect_equal(Mod(circle[2]), 1.0, tolerance = 1e-10)
  expect_equal(Mod(circle[3]), 1.0, tolerance = 1e-10)
  expect_equal(Mod(circle[4]), 1.0, tolerance = 1e-10)
})

# =============================================================================
# Lazy materialization tests
# =============================================================================

test_that("lazy materialization element access works", {
  lazy <- lazy_int_seq(1L, 100L, 1L)

  # Check initial state is NOT materialized
  expect_false(lazy_int_seq_is_materialized(lazy))

  # Element access should work without materialization
  expect_equal(lazy[1], 1L)
  expect_equal(lazy[50], 50L)
  expect_equal(lazy[100], 100L)

  # Still should not be materialized after element access
  expect_false(lazy_int_seq_is_materialized(lazy))
})

test_that("lazy materialization sum uses O(1) formula", {
  big_lazy <- lazy_int_seq(1L, 1000000L, 1L)

  # sum should be O(1) via formula, not materialize
  expect_equal(sum(big_lazy), 500000500000)

  # Still not materialized
  expect_false(lazy_int_seq_is_materialized(big_lazy))
})

test_that("lazy materialization triggers on dataptr", {
  lazy <- lazy_int_seq(1L, 10L, 1L)

  # Not materialized initially
  expect_false(lazy_int_seq_is_materialized(lazy))

  # Force materialization via arithmetic that requires dataptr
  y <- lazy + 0L

  # Now it should be materialized
  expect_true(lazy_int_seq_is_materialized(lazy))

  # Check values are still correct
  expect_equal(y, 1:10)
})

# =============================================================================
# Static slice ALTREP tests
# =============================================================================

test_that("static slice ALTREP works", {
  s <- static_ints()

  expect_equal(length(s), 5L)
  expect_equal(s[1], 10L)
  expect_equal(s[5], 50L)
  # Cannot use expect_equal(s, vec) or as.integer(s) because both try to get
  # writable DATAPTR which fails for static ALTREP data.
  # Compare element-by-element using indexing instead.
  expected <- c(10L, 20L, 30L, 40L, 50L)
  for (i in seq_along(expected)) {
    expect_equal(s[i], expected[i])
  }
  expect_equal(sum(s), 150L)
  expect_equal(min(s), 10L)
  expect_equal(max(s), 50L)
})

test_that("leaked heap ALTREP works", {
  leaked <- leaked_ints(10L)

  expect_equal(length(leaked), 10L)
  expect_equal(leaked[1], 1L)
  expect_equal(leaked[10], 10L)
  expect_equal(sum(leaked), 55L)
})

test_that("static string slice ALTREP works", {
  s <- static_strings()

  expect_equal(length(s), 4L)
  expect_equal(s[1], "alpha")
  expect_equal(s[2], "beta")
  expect_equal(s[3], "gamma")
  expect_equal(s[4], "delta")
})

test_that("constant logical ALTREP works", {
  true_vec <- constant_logical(1L, 3L)
  expect_equal(length(true_vec), 3L)
  expect_true(all(true_vec))
  expect_true(true_vec[1])
  expect_true(true_vec[3])

  false_vec <- constant_logical(0L, 2L)
  expect_true(all(!false_vec))
  expect_false(false_vec[1])
  expect_false(false_vec[2])

  na_vec <- constant_logical(NA_integer_, 2L)
  expect_true(is.na(na_vec[1]))
  expect_true(is.na(na_vec[2]))
})

test_that("lazy string ALTREP yields NA", {
  lazy <- lazy_string("prefix", 4L)

  expect_equal(length(lazy), 4L)
  expect_true(all(is.na(lazy)))
})

test_that("repeating raw ALTREP cycles pattern", {
  pattern <- as.raw(c(0xAA, 0x00))
  repeated <- repeating_raw(pattern, 5L)

  expect_equal(length(repeated), 5L)
  expect_equal(repeated[1:2], pattern)
  expect_equal(repeated[3], pattern[1])
  expect_equal(repeated[4], pattern[2])
  expect_equal(repeated[5], pattern[1])
})

# =============================================================================
# Box<[T]> ALTREP tests
# =============================================================================

test_that("Box<[i32]> ALTREP works", {
  boxed <- boxed_ints(5L)

  expect_equal(length(boxed), 5L)
  expect_equal(boxed[1], 1L)
  expect_equal(boxed[5], 5L)
  expect_equal(boxed, 1:5)
  expect_equal(sum(boxed), 15L)
})

test_that("Box<[i32]> has dataptr support", {
  boxed <- boxed_ints(10L)

  # Arithmetic should work via dataptr
  y <- boxed * 2L
  expect_equal(y, seq(2L, 20L, 2L))
})

# =============================================================================
# Iterator-backed ALTREP tests
# =============================================================================

test_that("iterator integer ALTREP works", {
  x <- iter_int_range(1L, 11L)

  expect_equal(length(x), 10L)
  expect_equal(x[1], 1L)
  expect_equal(x[10], 10L)
  expect_equal(sum(x), 55L)
})

test_that("iterator real ALTREP works", {
  x <- iter_real_squares(5L)

  expect_equal(length(x), 5L)
  expect_equal(x[1], 0)   # 0^2
  expect_equal(x[2], 1)   # 1^2
  expect_equal(x[3], 4)   # 2^2
  expect_equal(x[4], 9)   # 3^2
  expect_equal(x[5], 16)  # 4^2
  expect_equal(sum(x), 30)
})

test_that("iterator logical ALTREP works", {
  x <- iter_logical_alternating(6L)

  expect_equal(length(x), 6L)
  expect_true(x[1])   # 0 % 2 == 0 -> TRUE
  expect_false(x[2])  # 1 % 2 == 0 -> FALSE
  expect_true(x[3])
  expect_false(x[4])
  expect_true(x[5])
  expect_false(x[6])
  expect_equal(sum(x), 3L)  # 3 TRUEs
})

test_that("iterator raw ALTREP works", {
  x <- iter_raw_bytes(10L)

  expect_equal(length(x), 10L)
  expect_equal(x[1], as.raw(0))
  expect_equal(x[2], as.raw(1))
  expect_equal(x[10], as.raw(9))

  # Test cycling behavior
  big <- iter_raw_bytes(300L)
  expect_equal(big[1], as.raw(0))
  expect_equal(big[256], as.raw(255))
  expect_equal(big[257], as.raw(0))  # Wraps around
})

test_that("iterator string ALTREP works", {
  x <- iter_string_items(5L)

  expect_equal(length(x), 5L)
  expect_equal(x[1], "item_0")
  expect_equal(x[2], "item_1")
  expect_equal(x[5], "item_4")
})

test_that("complex ALTREP works (unit circle)", {
  x <- unit_circle(8L)

  expect_equal(length(x), 8L)

  # First point: angle=0 -> (1, 0)
  expect_equal(Re(x[1]), 1.0, tolerance = 1e-10)
  expect_equal(Im(x[1]), 0.0, tolerance = 1e-10)

  # All points should be on the unit circle (modulus = 1)
  for (i in seq_along(x)) {
    expect_equal(Mod(x[i]), 1.0, tolerance = 1e-10)
  }
})

test_that("iterator integer coercion ALTREP works", {
  x <- iter_int_from_u16(5L)

  expect_equal(length(x), 5L)
  expect_equal(x[1], 0L)      # 0 * 100
  expect_equal(x[2], 100L)    # 1 * 100
  expect_equal(x[3], 200L)    # 2 * 100
  expect_equal(x[5], 400L)    # 4 * 100
})

test_that("iterator real coercion ALTREP works", {
  x <- iter_real_from_f32(5L)

  expect_equal(length(x), 5L)
  expect_equal(x[1], 0.0, tolerance = 1e-6)
  expect_equal(x[2], 1.5, tolerance = 1e-6)
  expect_equal(x[3], 3.0, tolerance = 1e-6)
  expect_equal(x[5], 6.0, tolerance = 1e-6)
})

# =============================================================================
# Vec-backed ALTREP tests
# =============================================================================

test_that("Vec<i32> ALTREP works with dataptr", {
  x <- vec_int_altrep(10L)

  expect_equal(length(x), 10L)
  expect_equal(x[1], 1L)
  expect_equal(x[10], 10L)
  expect_equal(sum(x), 55L)

  # Vec has dataptr support, so arithmetic works
  y <- x * 2L
  expect_equal(y, seq(2L, 20L, 2L))
})

test_that("Vec<f64> ALTREP works with dataptr", {
  x <- vec_real_altrep(5L)

  expect_equal(length(x), 5L)
  expect_equal(x[1], 0.5)
  expect_equal(x[5], 2.5)
  expect_equal(sum(x), 7.5)

  # Vec has dataptr support
  y <- x + 1.0
  expect_equal(y, c(1.5, 2.0, 2.5, 3.0, 3.5))
})

# =============================================================================
# Range-backed ALTREP tests
# =============================================================================

test_that("Range<i32> ALTREP works", {
  x <- range_int_altrep(1L, 11L)

  expect_equal(length(x), 10L)
  expect_equal(x[1], 1L)
  expect_equal(x[10], 10L)
  expect_equal(sum(x), 55L)
  expect_equal(min(x), 1L)
  expect_equal(max(x), 10L)
})

test_that("Range ALTREP has O(1) operations", {
  # Large range - sum/min/max should be O(1) via formulas
  big <- range_int_altrep(1L, 1000001L)

  expect_equal(length(big), 1000000L)
  expect_equal(sum(big), 500000500000)  # n*(n+1)/2
  expect_equal(min(big), 1L)
  expect_equal(max(big), 1000000L)
})

test_that("Range ALTREP handles empty and single-element cases", {
  # Empty range
  empty <- range_int_altrep(5L, 5L)
  expect_equal(length(empty), 0L)

  # Single element
  single <- range_int_altrep(42L, 43L)
  expect_equal(length(single), 1L)
  expect_equal(single[1], 42L)
})

# =============================================================================
# Arithmetic sequence ALTREP tests
# =============================================================================

test_that("arithmetic sequence ALTREP works", {
  seq5 <- arith_seq(0, 1, 5L)

  expect_equal(length(seq5), 5L)
  expect_equal(seq5[1], 0.0)
  expect_equal(seq5[5], 1.0)
  expect_equal(seq5[3], 0.5)

  # Non-integer endpoints
  seq_neg <- arith_seq(-1, 1, 3L)
  expect_equal(seq_neg[1], -1.0)
  expect_equal(seq_neg[2], 0.0)
  expect_equal(seq_neg[3], 1.0)
})

# =============================================================================
# Edge cases
# =============================================================================

test_that("ALTREP handles zero-length vectors", {
  # Iterator
  empty_iter <- iter_int_range(0L, 0L)
  expect_equal(length(empty_iter), 0L)

  # Range
  empty_range <- range_int_altrep(5L, 5L)
  expect_equal(length(empty_range), 0L)

  # Vec
  empty_vec <- vec_int_altrep(0L)
  expect_equal(length(empty_vec), 0L)
})

test_that("ALTREP handles single-element vectors", {
  single_iter <- iter_int_range(42L, 43L)
  expect_equal(length(single_iter), 1L)
  expect_equal(single_iter[1], 42L)

  single_real <- iter_real_squares(1L)
  expect_equal(length(single_real), 1L)
  expect_equal(single_real[1], 0)
})

test_that("ALTREP vectors work with R subsetting", {
  x <- iter_int_range(1L, 11L)

  # Positive indices
  expect_equal(x[c(1, 3, 5)], c(1L, 3L, 5L))

  # Negative indices
  expect_equal(length(x[-1]), 9L)

  # Logical subsetting
  expect_equal(x[x > 5], 6:10)

  # head/tail
  expect_equal(head(x, 3), 1:3)
  expect_equal(tail(x, 3), 8:10)
})

test_that("ALTREP vectors work with R aggregate functions", {
  x <- iter_int_range(1L, 101L)

  expect_equal(mean(x), 50.5)
  expect_equal(median(x), 50.5)
  expect_equal(var(x), 100 * 101 / 12)  # var of 1:n is n(n+1)/12
  expect_equal(sd(x), sqrt(100 * 101 / 12))
  expect_equal(range(x), c(1L, 100L))
})
