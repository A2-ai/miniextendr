# Test thread safety utilities

# Thread tests are disabled because they demonstrate an unsafe pattern.
#
# The tests try to call R memory allocation APIs (Rf_ScalarInteger) from spawned
# threads. R's memory manager is NOT thread-safe - it can only be called from
# R's main thread. Even with stack checking disabled (nonapi feature), calling
# R allocation APIs from spawned threads leads to crashes or undefined behavior.
#
# RThreadBuilder is designed for CPU-intensive pure Rust work on other threads,
# NOT for calling R APIs. R API calls belong on R's main thread: collect the
# thread's results and convert them to R objects after it returns.

# test_that("RThreadBuilder with custom stack size works", {
#   result <- miniextendr:::unsafe_C_test_r_thread_builder()
#   expect_equal(result, 123L)
# })

# test_that("RThreadBuilder::spawn_join works", {
#   result <- miniextendr:::unsafe_C_test_r_thread_builder_spawn_join()
#   expect_equal(result, 456L)
# })
