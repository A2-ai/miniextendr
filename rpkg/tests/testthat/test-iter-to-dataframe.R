# Tests for iter_to_dataframe + DataFrameBuilder (#674).
#
# The Rust-side integration tests (miniextendr-api/tests/serde_streaming.rs)
# cover round-trip correctness, strict-schema rejection, NA-pad semantics,
# and DataFrameBuilder surface. These R tests verify GC safety under
# gctorture(TRUE) for the no-arg fixture.

test_that("gc_stress_iter_to_dataframe returns a valid data.frame", {
  skip_if_not(
    exists("gc_stress_iter_to_dataframe", envir = getNamespace("miniextendr"), mode = "function", inherits = FALSE),
    "gc_stress_iter_to_dataframe not compiled (serde feature missing)"
  )
  df <- miniextendr:::gc_stress_iter_to_dataframe()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 50L)
  expect_equal(ncol(df), 4L)
})

test_that("gc_stress_iter_to_dataframe survives gctorture(TRUE)", {
  skip_if_not(
    exists("gc_stress_iter_to_dataframe", envir = getNamespace("miniextendr"), mode = "function", inherits = FALSE),
    "gc_stress_iter_to_dataframe not compiled (serde feature missing)"
  )
  old <- gctorture(TRUE)
  on.exit(gctorture(old), add = TRUE)

  df <- miniextendr:::gc_stress_iter_to_dataframe()
  expect_equal(nrow(df), 50L)
  expect_equal(ncol(df), 4L)
})

# DataFrameBuilder::with_schema (#693) — pre-declared schema skips first-row
# discovery. Optional(Character) keeps the tag column character-typed even
# when the first row's tag is None.
test_that("gc_stress_builder_with_schema returns a valid data.frame", {
  skip_if_not(
    exists("gc_stress_builder_with_schema", envir = getNamespace("miniextendr"), mode = "function", inherits = FALSE),
    "gc_stress_builder_with_schema not compiled (serde feature missing)"
  )
  df <- miniextendr:::gc_stress_builder_with_schema()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 50L)
  expect_equal(ncol(df), 3L)
  expect_equal(names(df), c("id", "ratio", "tag"))
  expect_type(df$id, "integer")
  expect_type(df$ratio, "double")
  # Optional(Character) preserved despite first row's NA value.
  expect_type(df$tag, "character")
})

test_that("gc_stress_builder_with_schema survives gctorture(TRUE)", {
  skip_if_not(
    exists("gc_stress_builder_with_schema", envir = getNamespace("miniextendr"), mode = "function", inherits = FALSE),
    "gc_stress_builder_with_schema not compiled (serde feature missing)"
  )
  old <- gctorture(TRUE)
  on.exit(gctorture(old), add = TRUE)

  df <- miniextendr:::gc_stress_builder_with_schema()
  expect_equal(nrow(df), 50L)
  expect_equal(ncol(df), 3L)
})

# DataFrameBuilder::grow_schema (#692) — new fields from later rows are
# added on the fly and back-filled with NA on prior rows.
test_that("gc_stress_builder_grow_schema returns a valid data.frame", {
  skip_if_not(
    exists("gc_stress_builder_grow_schema", envir = getNamespace("miniextendr"), mode = "function", inherits = FALSE),
    "gc_stress_builder_grow_schema not compiled (serde feature missing)"
  )
  df <- miniextendr:::gc_stress_builder_grow_schema()
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 30L)
  # All grown columns share the same nrow.
  for (col in df) {
    expect_equal(length(col), 30L)
  }
})

test_that("gc_stress_builder_grow_schema survives gctorture(TRUE)", {
  skip_if_not(
    exists("gc_stress_builder_grow_schema", envir = getNamespace("miniextendr"), mode = "function", inherits = FALSE),
    "gc_stress_builder_grow_schema not compiled (serde feature missing)"
  )
  old <- gctorture(TRUE)
  on.exit(gctorture(old), add = TRUE)

  df <- miniextendr:::gc_stress_builder_grow_schema()
  expect_equal(nrow(df), 30L)
})
