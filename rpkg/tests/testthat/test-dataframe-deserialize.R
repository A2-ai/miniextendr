# Tests for dataframe_to_vec + with_dataframe_rows (#671a).
#
# The Rust-side integration tests (miniextendr-api/tests/dataframe_de.rs) cover
# round-trip correctness, NA handling, type-mismatch errors, and missing-column
# errors using the embedded R engine.  These R tests verify:
#   - GC safety under gctorture(TRUE) for both public entry points
#   - that the exported no-arg fixtures produce the correct return values

# region: gc_stress_dataframe_to_vec -------------------------------------------

test_that("gc_stress_dataframe_to_vec — returns expected row count", {
  skip_if_not(
    exists("gc_stress_dataframe_to_vec", mode = "function"),
    "gc_stress_dataframe_to_vec not compiled (serde feature missing)"
  )
  n <- gc_stress_dataframe_to_vec()
  expect_equal(n, 10L)
})

test_that("gc_stress_dataframe_to_vec — survives gctorture(TRUE)", {
  skip_if_not(
    exists("gc_stress_dataframe_to_vec", mode = "function"),
    "gc_stress_dataframe_to_vec not compiled (serde feature missing)"
  )
  old <- gctorture(TRUE)
  on.exit(gctorture(old), add = TRUE)

  n <- gc_stress_dataframe_to_vec()
  expect_equal(n, 10L)
})

# endregion

# region: gc_stress_dataframe_to_vec_nested ------------------------------------

test_that("gc_stress_dataframe_to_vec_nested — returns expected row count", {
  skip_if_not(
    exists("gc_stress_dataframe_to_vec_nested", mode = "function"),
    "gc_stress_dataframe_to_vec_nested not compiled (serde feature missing)"
  )
  n <- gc_stress_dataframe_to_vec_nested()
  expect_equal(n, 10L)
})

test_that("gc_stress_dataframe_to_vec_nested — survives gctorture(TRUE)", {
  skip_if_not(
    exists("gc_stress_dataframe_to_vec_nested", mode = "function"),
    "gc_stress_dataframe_to_vec_nested not compiled (serde feature missing)"
  )
  old <- gctorture(TRUE)
  on.exit(gctorture(old), add = TRUE)

  n <- gc_stress_dataframe_to_vec_nested()
  expect_equal(n, 10L)
})

# endregion

# region: gc_stress_with_dataframe_rows ----------------------------------------

test_that("gc_stress_with_dataframe_rows — returns expected sum", {
  skip_if_not(
    exists("gc_stress_with_dataframe_rows", mode = "function"),
    "gc_stress_with_dataframe_rows not compiled (serde feature missing)"
  )
  # rows 0..9, value = i * 2.0; sum = 2*(0+1+...+9) = 2*45 = 90
  total <- gc_stress_with_dataframe_rows()
  expect_equal(total, 90.0)
})

test_that("gc_stress_with_dataframe_rows — survives gctorture(TRUE)", {
  skip_if_not(
    exists("gc_stress_with_dataframe_rows", mode = "function"),
    "gc_stress_with_dataframe_rows not compiled (serde feature missing)"
  )
  old <- gctorture(TRUE)
  on.exit(gctorture(old), add = TRUE)

  total <- gc_stress_with_dataframe_rows()
  expect_equal(total, 90.0)
})

# endregion

# region: gc_stress_factor_labels (issue #689) ---------------------------------

test_that("gc_stress_factor_labels — round-trips factor column to labels", {
  skip_if_not(
    exists("gc_stress_factor_labels", mode = "function"),
    "gc_stress_factor_labels not compiled (serde feature missing)"
  )
  # Fixture builds 30 rows cycling 1..3 over levels c("active","pending","archived")
  # with NA cells at i %% 7 == 0.
  n <- gc_stress_factor_labels()
  expect_equal(n, 30L)
})

test_that("gc_stress_factor_labels — survives gctorture(TRUE)", {
  skip_if_not(
    exists("gc_stress_factor_labels", mode = "function"),
    "gc_stress_factor_labels not compiled (serde feature missing)"
  )
  old <- gctorture(TRUE)
  on.exit(gctorture(old), add = TRUE)

  n <- gc_stress_factor_labels()
  expect_equal(n, 30L)
})

# endregion
