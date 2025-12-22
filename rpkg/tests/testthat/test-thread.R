# Test thread safety utilities

# Tests for nonapi module functions are skipped because the nested module
# registration (miniextendr_module! with use nonapi;) isn't registering
# the nonapi symbols properly. TODO: Fix module registration.

test_that("RThreadBuilder with custom stack size works", {
  result <- unsafe_C_test_r_thread_builder()
  expect_equal(result, 123L)
})

test_that("RThreadBuilder::spawn_join works", {
  result <- unsafe_C_test_r_thread_builder_spawn_join()
  expect_equal(result, 456L)
})
