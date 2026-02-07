# These tests verify that R API calls from the worker thread produce clear error
# messages instead of crashing. The functions attempt to call R API functions
# (e.g., Rf_error, R_MakeExternalPtr) from the worker thread, which is not the
# main R thread. Currently miniextendr runs all Rust code on a worker thread, so
# R API calls fail with "non-main thread" errors.
#
# Resolution path: Implement main-thread dispatch so R API calls from the worker
# thread are forwarded to the main R thread and executed there. This is tracked
# in docs/GAPS.md under "Incomplete Features > Main-thread dispatch".
# Once resolved, these tests should be updated to expect successful results.
test_that("add_r_error signals clear thread error (worker thread rejects R API calls)", {
  expect_error(add_r_error(1L, 2L), "non-main thread")
})

test_that("add_r_error_heap signals clear thread error (worker thread rejects R API calls)", {
  expect_error(add_r_error_heap(1L, 2L), "non-main thread")
})

test_that("drop_on_panic_with_move signals clear thread error (worker thread rejects R API calls)", {
  expect_error(drop_on_panic_with_move(), "non-main thread")
})
