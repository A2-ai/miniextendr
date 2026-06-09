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

# region: cross-package condition matrix (issue #654)
#
# Row-by-row parity with rpkg's test-conditions-comprehensive.R, but raised
# through the trait-ABI vtable shim path (consumer wrapper -> CounterView shim
# -> producer error!/warning!/message!/condition! -> tagged SEXP -> re-panic at
# the View boundary -> consumer's outer with_r_unwind_protect -> consumer R
# wrapper). Asserts the same three dimensions the in-process matrix pins:
#   - class layering (rust_*, with optional user class layered first),
#   - e$kind round-trip,
#   - conditionCall(e) reflecting the CONSUMER-side wrapper (the .Call carries
#     .call = match.call()), not the producer-side vtable shim or R_NilValue.
#
# These migrated to the error_in_r tagged-SEXP transport in PR #349 (#345); the
# direct Rf_error at miniextendr_trait.rs:808 is gone, so cross-package raises
# now layer rust_* classes identically to a same-package call.

# error!() — bare, no user class
test_that("trait error!(): rust_error layering + e$kind + conditionCall", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(5L)
  e <- tryCatch(counter_raise_error(counter, "trait boom"), error = function(e) e)

  expect_true(inherits(e, "rust_error"),
    info = paste("class was:", paste(class(e), collapse = ", ")))
  expect_equal(class(e)[1], "rust_error")
  expect_equal(conditionMessage(e), "trait boom")
  expect_equal(e$kind, "error")

  # conditionCall must reflect the consumer-side wrapper, NOT the vtable shim.
  cl <- conditionCall(e)
  expect_false(is.null(cl))
  expect_true(grepl("counter_raise_error", deparse(cl)[[1]]),
    info = paste("conditionCall was:", paste(deparse(cl), collapse = " ")))
})

# warning!()
test_that("trait warning!(): rust_warning + e$kind + withCallingHandlers", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(5L)

  w <- tryCatch(counter_raise_warning(counter, "trait warn"), warning = function(w) w)
  expect_true(inherits(w, "rust_warning"),
    info = paste("class was:", paste(class(w), collapse = ", ")))
  expect_equal(conditionMessage(w), "trait warn")
  expect_equal(w$kind, "warning")

  # conditionCall reflects the consumer-side wrapper.
  cl <- conditionCall(w)
  expect_false(is.null(cl))
  expect_true(grepl("counter_raise_warning", deparse(cl)[[1]]),
    info = paste("conditionCall was:", paste(deparse(cl), collapse = " ")))

  # withCallingHandlers + muffleWarning resumes execution (non-fatal).
  saw <- FALSE
  withCallingHandlers(
    counter_raise_warning(counter, "transient"),
    warning = function(w) {
      saw <<- TRUE
      invokeRestart("muffleWarning")
    }
  )
  expect_true(saw)
})

# message!()
test_that("trait message!(): rust_message + e$kind", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(5L)

  m <- tryCatch(counter_raise_message(counter, "trait info"), message = function(m) m)
  expect_true(inherits(m, "rust_message"),
    info = paste("class was:", paste(class(m), collapse = ", ")))
  expect_equal(m$kind, "message")

  # withCallingHandlers + muffleMessage resumes execution (non-fatal).
  saw <- FALSE
  withCallingHandlers(
    counter_raise_message(counter, "intercepted"),
    message = function(m) {
      saw <<- TRUE
      invokeRestart("muffleMessage")
    }
  )
  expect_true(saw)
})

# condition!() with custom class
test_that("trait condition!(): classed + catchable + e$kind", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(5L)

  result <- tryCatch(
    counter_raise_condition_classed(counter, "trait_signal", "step done"),
    trait_signal = function(c) conditionMessage(c)
  )
  expect_equal(result, "step done")

  c_val <- tryCatch(
    counter_raise_condition_classed(counter, "trait_signal2", "kindcheck"),
    trait_signal2 = function(c) c
  )
  expect_true(inherits(c_val, "trait_signal2"),
    info = paste("class was:", paste(class(c_val), collapse = ", ")))
  expect_equal(c_val$kind, "condition")
})

# class-vector ordering parity with the in-process matrix
test_that("trait error!(class): user class precedes rust_error", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(3L)
  e <- tryCatch(
    counter_error_with_class(counter, "trait_specific_error"),
    error = function(e) e
  )
  cls <- class(e)
  user_pos <- match("trait_specific_error", cls)
  rust_pos <- match("rust_error", cls)
  expect_false(is.na(user_pos))
  expect_false(is.na(rust_pos))
  expect_true(user_pos < rust_pos,
    info = paste("class:", paste(cls, collapse = ", ")))
})

# StatefulCounter (producer-side, different impl, same shim path)
test_that("trait conditions round-trip for producer StatefulCounter", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  sc <- new_stateful_counter(2L)

  e <- tryCatch(counter_raise_error(sc, "stateful boom"), error = function(e) e)
  expect_true(inherits(e, "rust_error"))
  expect_equal(e$kind, "error")

  w <- tryCatch(counter_raise_warning(sc, "stateful warn"), warning = function(w) w)
  expect_true(inherits(w, "rust_warning"))
  expect_equal(w$kind, "warning")
})

# DoubleCounter (consumer-side impl — no package boundary, same shim path)
test_that("trait conditions round-trip for consumer DoubleCounter", {
  double <- new_double_counter(10L)

  e <- tryCatch(counter_raise_error(double, "double boom"), error = function(e) e)
  expect_true(inherits(e, "rust_error"))
  expect_equal(e$kind, "error")
  cl <- conditionCall(e)
  expect_false(is.null(cl))
  expect_true(grepl("counter_raise_error", deparse(cl)[[1]]))

  w <- tryCatch(counter_raise_warning(double, "double warn"), warning = function(w) w)
  expect_true(inherits(w, "rust_warning"))
  expect_equal(w$kind, "warning")

  m <- tryCatch(counter_raise_message(double, "double info"), message = function(m) m)
  expect_true(inherits(m, "rust_message"))
  expect_equal(m$kind, "message")

  result <- tryCatch(
    counter_raise_condition_classed(double, "double_signal", "double done"),
    double_signal = function(c) conditionMessage(c)
  )
  expect_equal(result, "double done")
})

# endregion
