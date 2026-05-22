# Tests for dataframe_to_vec_borrowed + BorrowedRows<'a, T> (#671b).
#
# The Rust-side integration tests (miniextendr-api/tests/dataframe_de_borrowed.rs)
# cover round-trip correctness, NA handling, empty data.frame, and non-data.frame
# rejection. This R test verifies GC safety under gctorture(TRUE) for the no-arg
# fixture.

test_that("gc_stress_borrowed_rows returns expected row count", {
  skip_if_not(
    exists("gc_stress_borrowed_rows", mode = "function"),
    "gc_stress_borrowed_rows not compiled (serde feature missing)"
  )
  n <- gc_stress_borrowed_rows()
  expect_equal(n, 10L)
})

test_that("gc_stress_borrowed_rows survives gctorture(TRUE)", {
  skip_if_not(
    exists("gc_stress_borrowed_rows", mode = "function"),
    "gc_stress_borrowed_rows not compiled (serde feature missing)"
  )
  old <- gctorture(TRUE)
  on.exit(gctorture(old), add = TRUE)

  n <- gc_stress_borrowed_rows()
  expect_equal(n, 10L)
})
