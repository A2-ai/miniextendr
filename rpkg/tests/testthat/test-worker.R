# Tests for run_on_worker and with_r_thread mechanisms
# These test panic safety, R error handling, and nested wrapper scenarios

test_that("simple worker execution works", {
  result <- miniextendr:::unsafe_C_test_worker_simple()
  expect_equal(result, 42L)
})

test_that("worker with with_r_thread works", {
  result <- miniextendr:::unsafe_C_test_worker_with_r_thread()
  expect_equal(result, 123L)
})

test_that("worker with multiple R calls works", {
  result <- miniextendr:::unsafe_C_test_worker_multiple_r_calls()
  expect_equal(result, c(10L, 30L, 60L))
})

test_that("worker with RAII resources returns correct value", {
  result <- miniextendr:::unsafe_C_worker_drop_on_success()
  expect_equal(result, 42L)
})

# Panic scenarios

test_that("panic on worker thread is caught", {
  expect_error(
    miniextendr:::unsafe_C_test_worker_panic_simple(),
    "simple panic on worker"
  )
})

test_that("panic on worker with RAII resources drops them", {
  expect_error(
    miniextendr:::unsafe_C_test_worker_panic_with_drops(),
    "panic after creating resources"
  )
})

test_that("panic inside with_r_thread callback is caught", {
  expect_error(
    miniextendr:::unsafe_C_test_worker_panic_in_r_thread(),
    "panic inside with_r_thread callback"
  )
})

test_that("panic in with_r_thread with resources drops them", {
  expect_error(
    miniextendr:::unsafe_C_test_worker_panic_in_r_thread_with_drops(),
    "panic in with_r_thread with resources"
  )
})

# R error scenarios

test_that("R error inside with_r_thread is propagated", {
  expect_error(
    miniextendr:::unsafe_C_test_worker_r_error_in_r_thread(),
    "R error"
  )
})

test_that("R error with RAII resources propagates error", {
  expect_error(
    miniextendr:::unsafe_C_test_worker_r_error_with_drops(),
    "R error"
  )
})

# Mixed scenarios

test_that("multiple R calls then error propagates error", {
  expect_error(
    miniextendr:::unsafe_C_test_worker_r_calls_then_error(),
    "Error"
  )
})

test_that("multiple R calls then panic propagates panic", {
  expect_error(
    miniextendr:::unsafe_C_test_worker_r_calls_then_panic(),
    "Rust panic after successful R call"
  )
})

# Return value propagation

test_that("i32 return from miniextendr function works", {
  result <- miniextendr:::test_worker_return_i32()
  expect_equal(result, 42L)
})

test_that("String return from miniextendr function works", {
  result <- miniextendr:::test_worker_return_string()
  expect_equal(result, "hello from worker")
})

test_that("f64 return from miniextendr function works", {
  result <- miniextendr:::test_worker_return_f64()
  expect_equal(result, 2 * pi, tolerance = 0.001)
})

# ExternalPtr from worker context

test_that("ExternalPtr can be created with computation on worker", {
  ptr <- miniextendr:::unsafe_C_test_extptr_from_worker()
  expect_true(inherits(ptr, "externalptr"))
  expect_equal(miniextendr:::unsafe_C_extptr_counter_get(ptr), 100L)
})

test_that("multiple ExternalPtrs can be created from worker context", {
  result <- miniextendr:::unsafe_C_test_multiple_extptrs_from_worker()
  expect_true(is.list(result))
  expect_length(result, 2)
  expect_equal(miniextendr:::unsafe_C_extptr_counter_get(result[[1]]), 100L)
  expect_equal(miniextendr:::unsafe_C_extptr_point_get_x(result[[2]]), 1.5)
  expect_equal(miniextendr:::unsafe_C_extptr_point_get_y(result[[2]]), 2.5)
})

# Main thread functions

test_that("main_thread attribute function can call R API", {
  result <- miniextendr:::test_main_thread_r_api()
  expect_equal(result, 42L)
})

test_that("main_thread function R error is propagated", {
  expect_error(
    miniextendr:::test_main_thread_r_error(),
    "R error from main_thread fn"
  )
})

test_that("main_thread function R error with drops still drops resources", {
  expect_error(
    miniextendr:::test_main_thread_r_error_with_drops(),
    "R error from main_thread fn with drops"
  )
})

# Checked R API calls from worker thread are routed to main thread via
# with_r_thread (all return types, including pointers). This should succeed.

# test_that("checked R API from worker thread is routed to main", {
#   expect_no_error(miniextendr:::unsafe_C_test_wrong_thread_r_api())
# })

# Nested wrapper tests

test_that("helper function from worker works", {
  result <- miniextendr:::unsafe_C_test_nested_helper_from_worker()
  expect_equal(result, 42L)
})

test_that("multiple helper calls from worker work", {
  result <- miniextendr:::unsafe_C_test_nested_multiple_helpers()
  expect_equal(result, 60L)
})

test_that("nested with_r_thread calls work", {
  result <- miniextendr:::unsafe_C_test_nested_with_r_thread()
  expect_equal(result, 42L)
})

test_that("calling worker function from main thread works", {
  result <- miniextendr:::unsafe_C_test_call_worker_fn_from_main()
  expect_equal(result, 42L)
})

test_that("nested worker calls work", {
  result <- miniextendr:::unsafe_C_test_nested_worker_calls()
  expect_equal(result, 400L)
})

test_that("nested with_r_thread with error propagates error", {
  expect_error(
    miniextendr:::unsafe_C_test_nested_with_error(),
    "Error"
  )
})

test_that("nested with_r_thread with panic propagates panic", {
  expect_error(
    miniextendr:::unsafe_C_test_nested_with_panic(),
    "Panic in nested with_r_thread"
  )
})

test_that("deep with_r_thread sequence works", {
  result <- miniextendr:::unsafe_C_test_deep_with_r_thread_sequence()
  expect_equal(result, 45L)
})
