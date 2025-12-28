test_that("RThreadBuilder basic flow works", {
  skip("Crashes current R runtime; needs safer registration before enabling")
  result <- rpkg:::unsafe_C_test_r_thread_builder()
  expect_equal(result, 123L)
})

test_that("RThreadBuilder spawn_join works", {
  skip("Crashes current R runtime; needs safer registration before enabling")
  result <- rpkg:::unsafe_C_test_r_thread_builder_spawn_join()
  expect_equal(result, 456L)
})
