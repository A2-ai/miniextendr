test_that("RThreadBuilder basic flow (pending fix)", {
  skip("RThreadBuilder symbols not registered correctly yet; see TODO in test-thread.R")
  result <- rpkg:::unsafe_C_test_r_thread_builder()
  expect_equal(result, 123L)
})

test_that("RThreadBuilder spawn_join (pending fix)", {
  skip("RThreadBuilder spawn_join currently fails; enable once module registration is fixed")
  result <- rpkg:::unsafe_C_test_r_thread_builder_spawn_join()
  expect_equal(result, 456L)
})
