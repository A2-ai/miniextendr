test_that("cli_active_progress_bars returns integer", {
  result <- cli_active_progress_bars()
  expect_type(result, "integer")
  expect_length(result, 1)
  expect_gte(result, 0L)
})
