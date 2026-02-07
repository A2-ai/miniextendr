# RThreadBuilder attempts to register a new thread with R's threading system so
# it can safely call R API functions. Currently this crashes R because R's
# internal thread registration (via R_CStackStart and related mechanisms) is not
# designed for arbitrary threads -- it corrupts R's stack checking state.
#
# Resolution path: Either use R's own R_BeginDeferred/R_EndDeferred (if exposed
# in future R versions), or implement a message-passing approach where the
# worker thread sends requests to the main R thread. See docs/GAPS.md and
# docs/THREADS.md for design discussion.
test_that("RThreadBuilder basic flow works", {
  skip("Crashes current R runtime; needs safer registration before enabling")
  result <- miniextendr:::unsafe_C_test_r_thread_builder()
  expect_equal(result, 123L)
})

test_that("RThreadBuilder spawn_join works", {
  skip("Crashes current R runtime; needs safer registration before enabling")
  result <- miniextendr:::unsafe_C_test_r_thread_builder_spawn_join()
  expect_equal(result, 456L)
})
