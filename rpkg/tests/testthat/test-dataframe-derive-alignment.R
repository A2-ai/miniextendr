# Tests that standalone #[derive(DataFrameRow)] produces a companion type
# that can be returned from #[miniextendr] functions (has IntoR impl).

test_that("standalone DataFrameRow derive returns data.frame from Rust", {
  result <- standalone_dataframe_roundtrip()
  expect_s3_class(result, "data.frame")
  expect_equal(nrow(result), 2L)
  expect_equal(result$name, c("a", "b"))
  expect_equal(result$value, c(1.0, 2.0))
})
