# Tests for iter_to_dataframe + DataFrameBuilder (#674).
#
# The Rust-side integration tests (miniextendr-api/tests/serde_streaming.rs)
# cover round-trip correctness, strict-schema rejection, NA-pad semantics,
# and DataFrameBuilder surface. These R tests verify GC safety under
# gctorture(TRUE) for the no-arg fixture.

test_that("gc_stress_iter_to_dataframe returns a valid data.frame", {
  skip_if_not(
    exists("gc_stress_iter_to_dataframe", mode = "function"),
    "gc_stress_iter_to_dataframe not compiled (serde feature missing)"
  )
  df <- gc_stress_iter_to_dataframe()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 50L)
  expect_equal(ncol(df), 4L)
})

test_that("gc_stress_iter_to_dataframe survives gctorture(TRUE)", {
  skip_if_not(
    exists("gc_stress_iter_to_dataframe", mode = "function"),
    "gc_stress_iter_to_dataframe not compiled (serde feature missing)"
  )
  old <- gctorture(TRUE)
  on.exit(gctorture(old), add = TRUE)

  df <- gc_stress_iter_to_dataframe()
  expect_equal(nrow(df), 50L)
  expect_equal(ncol(df), 4L)
})
