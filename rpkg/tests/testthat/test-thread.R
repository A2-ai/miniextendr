# Test thread safety utilities

# Thread tests are disabled because they demonstrate an unsafe pattern.
#
# The tests try to call R memory allocation APIs (Rf_ScalarInteger) from worker
# threads. R's memory manager is NOT thread-safe - it can only be called from
# R's main thread. Even with stack checking disabled (nonapi feature), calling
# R allocation APIs from worker threads leads to crashes or undefined behavior.
#
# RThreadBuilder is designed for CPU-intensive pure Rust work on worker threads,
# NOT for calling R APIs. The proper pattern for R API calls is to use the
# worker thread infrastructure in #[miniextendr] which routes R calls back to
# the main thread via the with_r_thread mechanism.
#
# TODO: Rewrite these tests to demonstrate valid RThreadBuilder usage (pure Rust work)
# TODO: Add tests for actual main-thread R API patterns

# test_that("RThreadBuilder with custom stack size works", {
#   result <- unsafe_C_test_r_thread_builder()
#   expect_equal(result, 123L)
# })

# test_that("RThreadBuilder::spawn_join works", {
#   result <- unsafe_C_test_r_thread_builder_spawn_join()
#   expect_equal(result, 456L)
# })
