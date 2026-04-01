test_that("ExternalSlice length is correct", {
  expect_equal(external_slice_len(c(1.0, 2.0, 3.0)), 3L)
  expect_equal(external_slice_len(numeric(0)), 0L)
})

test_that("ExternalSlice sum via as_slice", {
  expect_equal(external_slice_sum(c(1.0, 2.0, 3.0, 4.0)), 10.0)
})
