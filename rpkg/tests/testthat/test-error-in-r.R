# Tests for #[miniextendr(error_in_r)] mode.
#
# error_in_r promotes all Rust-origin failures (panics, Result::Err, Option::None)
# as R error conditions (class "rust_error") that are raised past the Rust boundary.

# =============================================================================
# Standalone function tests
# =============================================================================

test_that("error_in_r panic raises rust_error condition", {
  err <- tryCatch(
    error_in_r_panic(),
    error = function(e) e
  )
  expect_s3_class(err, "rust_error")
  expect_equal(err$kind, "panic")
  expect_match(err$message, "test panic from error_in_r")
})

test_that("error_in_r Result::Err raises rust_error condition", {
  err <- tryCatch(
    error_in_r_result_err(),
    error = function(e) e
  )
  expect_s3_class(err, "rust_error")
  expect_equal(err$kind, "result_err")
  expect_match(err$message, "test result error")
})

test_that("error_in_r Result::Ok returns value normally", {
  expect_equal(error_in_r_result_ok(), "success")
})

test_that("error_in_r Option::None raises rust_error condition", {
  err <- tryCatch(
    error_in_r_option_none(),
    error = function(e) e
  )
  expect_s3_class(err, "rust_error")
  expect_equal(err$kind, "none_err")
  expect_match(err$message, "returned None")
})

test_that("error_in_r Option::Some returns NULL", {
  expect_null(error_in_r_option_some())
})

test_that("error_in_r normal return works", {
  expect_equal(error_in_r_normal(), "all good")
})

test_that("error_in_r i32 return works", {
  expect_equal(error_in_r_i32_ok(), 42L)
})

test_that("error_in_r i32 Result::Err raises condition", {
  err <- tryCatch(
    error_in_r_i32_err(),
    error = function(e) e
  )
  expect_s3_class(err, "rust_error")
  expect_equal(err$kind, "result_err")
  expect_match(err$message, "integer conversion failed")
})

test_that("error_in_r custom panic message", {
  err <- tryCatch(
    error_in_r_panic_custom("custom panic msg"),
    error = function(e) e
  )
  expect_s3_class(err, "rust_error")
  expect_equal(err$kind, "panic")
  expect_match(err$message, "custom panic msg")
})

# =============================================================================
# Condition class hierarchy
# =============================================================================

test_that("rust_error inherits from simpleError, error, condition", {
  err <- tryCatch(
    error_in_r_panic(),
    error = function(e) e
  )
  expect_true(inherits(err, "rust_error"))
  expect_true(inherits(err, "simpleError"))
  expect_true(inherits(err, "error"))
  expect_true(inherits(err, "condition"))
})

test_that("rust_error has call information", {
  err <- tryCatch(
    error_in_r_panic(),
    error = function(e) e
  )
  # The condition should have a call component
  expect_true(!is.null(err$call))
})

# =============================================================================
# Condition can be caught with tryCatch handlers
# =============================================================================

test_that("rust_error can be caught by class-specific handler", {
  result <- tryCatch(
    error_in_r_panic(),
    rust_error = function(e) paste("caught:", e$kind)
  )
  expect_equal(result, "caught: panic")
})

test_that("rust_error can be caught by generic error handler", {
  result <- tryCatch(
    error_in_r_result_err(),
    error = function(e) paste("caught:", e$kind)
  )
  expect_equal(result, "caught: result_err")
})

# =============================================================================
# Env class system tests
# =============================================================================

test_that("error_in_r env class methods work normally", {
  c <- ErrorInRCounter$new()
  expect_equal(c$get(), 0L)
  c$inc()
  expect_equal(c$get(), 1L)
  c$inc()
  expect_equal(c$get(), 2L)
})

test_that("error_in_r env class panic raises condition", {
  c <- ErrorInRCounter$new()
  err <- tryCatch(
    c$panic_method(),
    error = function(e) e
  )
  expect_s3_class(err, "rust_error")
  expect_equal(err$kind, "panic")
  expect_match(err$message, "method panic in error_in_r")
})

test_that("error_in_r env class Result::Err raises condition", {
  c <- ErrorInRCounter$new()
  err <- tryCatch(
    c$failing_method(),
    error = function(e) e
  )
  expect_s3_class(err, "rust_error")
  expect_equal(err$kind, "result_err")
  expect_match(err$message, "method error")
})

test_that("error_in_r env class object survives after error", {
  c <- ErrorInRCounter$new()
  c$inc()
  # Error should not corrupt the object
  tryCatch(c$panic_method(), error = function(e) NULL)
  expect_equal(c$get(), 1L)
})

# =============================================================================
# R6 class system tests
# =============================================================================

test_that("error_in_r R6 class methods work normally", {
  w <- ErrorInRR6Widget$new("test_widget")
  expect_equal(w$get_name(), "test_widget")
})

test_that("error_in_r R6 class panic raises condition", {
  w <- ErrorInRR6Widget$new("test_widget")
  err <- tryCatch(
    w$panic_method(),
    error = function(e) e
  )
  expect_s3_class(err, "rust_error")
  expect_equal(err$kind, "panic")
  expect_match(err$message, "R6 panic in error_in_r")
})

test_that("error_in_r R6 class Result::Err raises condition", {
  w <- ErrorInRR6Widget$new("test_widget")
  err <- tryCatch(
    w$failing_result(),
    error = function(e) e
  )
  expect_s3_class(err, "rust_error")
  expect_equal(err$kind, "result_err")
  expect_match(err$message, "R6 result error")
})

test_that("error_in_r R6 class object survives after error", {
  w <- ErrorInRR6Widget$new("survivor")
  # Error should not corrupt the R6 object
  tryCatch(w$panic_method(), error = function(e) NULL)
  expect_equal(w$get_name(), "survivor")
})

# =============================================================================
# Multiple errors in sequence
# =============================================================================

test_that("error_in_r handles multiple sequential errors", {
  # Each error should be independently catchable
  for (i in seq_len(5)) {
    err <- tryCatch(
      error_in_r_panic(),
      error = function(e) e
    )
    expect_s3_class(err, "rust_error")
  }
  # Normal calls should still work after errors
  expect_equal(error_in_r_normal(), "all good")
})
