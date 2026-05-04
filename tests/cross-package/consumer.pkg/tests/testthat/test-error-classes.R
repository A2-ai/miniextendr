# Spike tests for issue #345: rust_error class layering across the trait-ABI boundary.
#
# Approach 1: shim returns tagged SEXP → View re-panics → consumer's outer
# error_in_r guard produces rust_* class layering.
#
# These tests run against producer.pkg objects (cross-package) to verify the
# behaviour at the trait-ABI boundary.

test_that("panic from cross-package trait method matches rust_error", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(5L)

  err <- tryCatch(
    counter_panic_plain(counter),
    rust_error = function(e) e,
    error = function(e) stop(paste("unexpected non-rust_error:", conditionMessage(e)))
  )

  # Should have matched rust_error, not fallen through to generic error
  expect_true(inherits(err, "rust_error"),
    info = paste("class was:", paste(class(err), collapse = ", ")))
  expect_match(conditionMessage(err), "panic_plain")
})

test_that("error!() with custom class from cross-package trait matches user class first", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(7L)

  # Should match user_class_456 first
  err_user <- tryCatch(
    counter_error_with_class(counter, "user_class_456"),
    user_class_456 = function(e) e,
    error = function(e) stop(paste("unexpected catch:", conditionMessage(e)))
  )
  expect_true(inherits(err_user, "user_class_456"),
    info = paste("class was:", paste(class(err_user), collapse = ", ")))
  expect_true(inherits(err_user, "rust_error"),
    info = "rust_error should also be in class vector")
  expect_match(conditionMessage(err_user), "error_with_class")

  # Should also match rust_error
  err_rust <- tryCatch(
    counter_error_with_class(counter, "another_class"),
    rust_error = function(e) e,
    error = function(e) stop(paste("unexpected catch:", conditionMessage(e)))
  )
  expect_true(inherits(err_rust, "rust_error"),
    info = paste("class was:", paste(class(err_rust), collapse = ", ")))
  expect_true(inherits(err_rust, "another_class"),
    info = "user class should also be in class vector")
})

test_that("class vector ordering: user class comes before rust_error", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(3L)

  err <- tryCatch(
    counter_error_with_class(counter, "my_specific_error"),
    error = function(e) e
  )

  cls <- class(err)
  user_pos <- match("my_specific_error", cls)
  rust_pos <- match("rust_error", cls)

  expect_false(is.na(user_pos),
    info = paste("my_specific_error not found in class:", paste(cls, collapse = ", ")))
  expect_false(is.na(rust_pos),
    info = paste("rust_error not found in class:", paste(cls, collapse = ", ")))
  expect_true(user_pos < rust_pos,
    info = paste("user class should precede rust_error in:", paste(cls, collapse = ", ")))
})

test_that("consumer's own DoubleCounter trait dispatch also gets rust_error class", {
  # DoubleCounter is in consumer.pkg (no package boundary for the trait impl),
  # but still goes through the same vtable shim path.
  double <- new_double_counter(10L)

  err <- tryCatch(
    counter_panic_plain(double),
    rust_error = function(e) e,
    error = function(e) stop(paste("unexpected non-rust_error:", conditionMessage(e)))
  )
  expect_true(inherits(err, "rust_error"),
    info = paste("class was:", paste(class(err), collapse = ", ")))
})

test_that("normal counter_get_value still works (no regression)", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(42L)
  expect_equal(peek_value(counter), 42L)
  increment_twice(counter)
  expect_equal(peek_value(counter), 44L)
})
