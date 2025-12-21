# ALTREP tests for miniextendr

# =============================================================================
# Proc-macro ALTREP tests (ConstantInt)
# =============================================================================

test_that("proc-macro ALTREP (ConstantIntClass) works", {
  c42 <- rpkg:::altrep_constant_int()
  expect_equal(length(c42), 10L)
  # Use element access instead of all() which requires dataptr
  expect_equal(c42[1], 42L)
  expect_equal(c42[10], 42L)
  expect_equal(sum(c42), 420L)
  expect_equal(mean(c42), 42)
})

# =============================================================================
# Complex number ALTREP tests
# =============================================================================

test_that("complex ALTREP (UnitCircle) works", {
  circle <- rpkg:::unit_circle(4L)  # 4th roots of unity
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
  lazy <- rpkg:::lazy_int_seq(1L, 100L, 1L)

  # Check initial state is NOT materialized
  expect_false(rpkg:::altrep_lazy_int_seq_is_materialized(lazy))

  # Element access should work without materialization
  expect_equal(lazy[1], 1L)
  expect_equal(lazy[50], 50L)
  expect_equal(lazy[100], 100L)

  # Still should not be materialized after element access
  expect_false(rpkg:::altrep_lazy_int_seq_is_materialized(lazy))
})

test_that("lazy materialization sum uses O(1) formula", {
  big_lazy <- rpkg:::lazy_int_seq(1L, 1000000L, 1L)

  # sum should be O(1) via formula, not materialize
  expect_equal(sum(big_lazy), 500000500000)

  # Still not materialized
  expect_false(rpkg:::altrep_lazy_int_seq_is_materialized(big_lazy))
})

test_that("lazy materialization triggers on dataptr", {
  lazy <- rpkg:::lazy_int_seq(1L, 10L, 1L)

  # Not materialized initially
  expect_false(rpkg:::altrep_lazy_int_seq_is_materialized(lazy))

  # Force materialization via arithmetic that requires dataptr
  y <- lazy + 0L

  # Now it should be materialized
  expect_true(rpkg:::altrep_lazy_int_seq_is_materialized(lazy))

  # Check values are still correct
  expect_equal(y, 1:10)
})

# =============================================================================
# Static slice ALTREP tests
# =============================================================================

test_that("static slice ALTREP works", {
  s <- rpkg:::static_ints()

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
  leaked <- rpkg:::leaked_ints(10L)

  expect_equal(length(leaked), 10L)
  expect_equal(leaked[1], 1L)
  expect_equal(leaked[10], 10L)
  expect_equal(sum(leaked), 55L)
})

test_that("static string slice ALTREP works", {
  s <- rpkg:::static_strings()

  expect_equal(length(s), 4L)
  expect_equal(s[1], "alpha")
  expect_equal(s[2], "beta")
  expect_equal(s[3], "gamma")
  expect_equal(s[4], "delta")
})

# =============================================================================
# Box<[T]> ALTREP tests
# =============================================================================

test_that("Box<[i32]> ALTREP works", {
  boxed <- rpkg:::boxed_ints(5L)

  expect_equal(length(boxed), 5L)
  expect_equal(boxed[1], 1L)
  expect_equal(boxed[5], 5L)
  expect_equal(boxed, 1:5)
  expect_equal(sum(boxed), 15L)
})

test_that("Box<[i32]> has dataptr support", {
  boxed <- rpkg:::boxed_ints(10L)

  # Arithmetic should work via dataptr
  y <- boxed * 2L
  expect_equal(y, seq(2L, 20L, 2L))
})
