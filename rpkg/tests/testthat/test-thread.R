# Test thread safety utilities (nonapi feature)

test_that("spawn_with_r allows R API calls from spawned thread", {
  # This would segfault without StackCheckGuard
  result <- unsafe_C_test_spawn_with_r_simple()
  expect_equal(result, 42L)
})

test_that("spawn_with_r supports computation with R API calls", {
  # Sum 1..100 = 5050
  result <- unsafe_C_test_spawn_with_r_computation()
  expect_equal(result, 5050L)
})

test_that("RThreadBuilder with custom stack size works", {
  result <- unsafe_C_test_r_thread_builder()
  expect_equal(result, 123L)
})

test_that("RThreadBuilder::spawn_join works", {
  result <- unsafe_C_test_r_thread_builder_spawn_join()
  expect_equal(result, 456L)
})

test_that("StackCheckGuard with std::thread::spawn works", {
  result <- unsafe_C_test_stack_check_guard()
  expect_equal(result, 789L)
})

test_that("with_stack_checking_disabled closure wrapper works", {
  result <- unsafe_C_test_with_stack_checking_disabled()
  expect_equal(result, 999L)
})

test_that("multiple R API calls from spawned thread work", {
  # 10 + 20 + 30 = 60
  result <- unsafe_C_test_spawn_multiple_r_calls()
  expect_equal(result, 60L)
})

test_that("creating R vectors from spawned thread works", {
  # 10 + 20 + 30 + 40 + 50 = 150
  result <- unsafe_C_test_spawn_create_vector()
  expect_equal(result, 150L)
})

test_that("stack checking is disabled inside spawn_with_r", {
  # Returns 1 if is_stack_checking_disabled() is true
  result <- unsafe_C_test_stack_check_status()
  expect_equal(result, 1L)
})
