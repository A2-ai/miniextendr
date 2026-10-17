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
#     .call = sys.call()), not the producer-side vtable shim or R_NilValue.
#
# These migrated to the error_in_r tagged-SEXP transport in PR #349 (#345); the
# direct Rf_error in the trait shim is gone, so cross-package raises
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

# warning!(call = none, ...) (#1725)
test_that("trait warning!(call = none): no call across the trait-ABI boundary", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  for (counter in list(new_counter(5L), new_double_counter(1L))) {
    w <- tryCatch(
      counter_raise_warning_without_call(counter, "data warn"),
      data_warning = function(w) w
    )
    expect_equal(
      class(w),
      c("data_warning", "rust_warning", "simpleWarning", "warning", "condition")
    )
    expect_equal(conditionMessage(w), "data warn")
    expect_equal(w$kind, "warning")
    expect_null(conditionCall(w))
    expect_equal(capture.output(print(w)), "<data_warning: data warn>")
  }

  # The sibling warning from the same trait keeps the consumer's call.
  w <- tryCatch(counter_raise_warning(new_counter(5L), "warn"), warning = function(w) w)
  expect_true(grepl("counter_raise_warning", deparse(conditionCall(w))[[1]]))
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

# arg_error!() — an argument error found by the implementation's body
test_that("trait arg_error!(): kind, param and the implementing package's classes", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  base <- c("rust_error", "simpleError", "error", "condition")
  # The classes are those of the package that implements the method, as from
  # its own wrapper, not those of the consumer that raises the condition:
  # producer.pkg for SimpleCounter and StatefulCounter, consumer.pkg for
  # DoubleCounter.
  cases <- list(
    list(counter = new_counter(5L), class = c("producer_error_argument", "producer_error", base)),
    list(counter = new_stateful_counter(2L), class = c("producer_error_argument", "producer_error", base)),
    list(counter = new_double_counter(1L), class = c("consumer_error_argument", "consumer_error", base))
  )
  for (case in cases) {
    e <- tryCatch(counter_raise_arg_error(case$counter, "sideways"), error = function(e) e)
    expect_identical(class(e), case$class)
    expect_identical(e$kind, "conversion")
    expect_identical(e$param, "mode")
    expect_identical(conditionMessage(e), "'mode' is not a counter mode: \"sideways\"")
    expect_identical(sort(names(unclass(e))), c("call", "kind", "message", "param"))
    expect_true(grepl("counter_raise_arg_error", deparse(conditionCall(e))[[1]]))
  }

  # The consumer function's own check of its `mode` argument is the
  # consumer's argument error, whichever counter it was given.
  e_check <- tryCatch(counter_raise_arg_error(new_counter(1L), 1:2), error = function(e) e)
  expect_identical(class(e_check), c("consumer_error_argument", "consumer_error", base))
  expect_identical(e_check$kind, "conversion")
  expect_identical(e_check$param, "mode")
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

# region: structured data round-trip across trait-ABI boundary (issue #996 path-1)
#
# `counter_raise_error_with_data(counter)` calls
#   producer Counter::raise_error_with_data()
#     → error!(class = "data_bearing_error", data = [("field_a", value), ...], "msg")
#     → vtable shim returns tagged SEXP (slot [4] = named list)
#     → repanic_if_rust_error calls from_tagged_sexp, which now reads slot [4]
#     → RCondition::Error { data: Some([...]) } is re-panicked
#     → consumer's with_r_unwind_protect produces a new tagged SEXP with data
#     → R wrapper raises the condition with e$field_a etc. accessible.

test_that("structured data survives trait-ABI re-panic (from_tagged_sexp slot [4])", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(5L)

  e <- tryCatch(
    counter_raise_error_with_data(counter),
    data_bearing_error = function(e) e,
    error = function(e) stop(paste("wrong class caught:", conditionMessage(e)))
  )

  # Verify class layering is intact
  expect_true(inherits(e, "data_bearing_error"),
    info = paste("class was:", paste(class(e), collapse = ", ")))
  expect_true(inherits(e, "rust_error"),
    info = "rust_error must also be present")

  # Verify structured data fields survived the cross-package round-trip.
  # field_a is self.value which is 5L for new_counter(5L).
  expect_equal(e$field_a, 5L,
    info = paste("field_a was:", e$field_a))
  expect_equal(e$field_b, "hello from producer",
    info = paste("field_b was:", e$field_b))
  expect_equal(e$flag, TRUE,
    info = paste("flag was:", e$flag))
  expect_equal(e$score, 1.23,
    info = paste("score was:", e$score),
    tolerance = 1e-10)
})

test_that("structured data round-trip also works for StatefulCounter", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  sc <- new_stateful_counter(10L)

  e <- tryCatch(
    counter_raise_error_with_data(sc),
    data_bearing_error = function(e) e,
    error = function(e) stop(paste("wrong class:", conditionMessage(e)))
  )

  expect_true(inherits(e, "data_bearing_error"))
  # field_a is the counter value: 10L
  expect_equal(e$field_a, 10L)
  expect_equal(e$field_b, "hello from stateful producer")
  expect_equal(e$flag, TRUE)
  expect_equal(e$score, 1.23, tolerance = 1e-10)
})

test_that("structured data round-trip works for consumer DoubleCounter (no package boundary)", {
  double <- new_double_counter(3L)

  e <- tryCatch(
    counter_raise_error_with_data(double),
    data_bearing_error = function(e) e,
    error = function(e) stop(paste("wrong class:", conditionMessage(e)))
  )

  expect_true(inherits(e, "data_bearing_error"))
  expect_equal(e$field_a, 3L)
  expect_equal(e$field_b, "hello from consumer")
  expect_equal(e$flag, TRUE)
  expect_equal(e$score, 1.23, tolerance = 1e-10)
})

# endregion

# region: conversion failures at the trait-ABI boundary
#
# A shim converts its arguments with TryFromSexp, and a View converts the
# shim's result back. A failed argument conversion returns a structured
# conversion condition; a failed result conversion panics inside the consumer's
# unwind boundary. The View re-panics with a tagged condition, and the consumer's
# wrapper raises it. Before, both sides called Rf_error,
# which raised a plain error and skipped the Rust destructors on the way.

test_that("a shim's failed argument conversion reaches the caller as rust_error", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  base <- c("rust_error", "simpleError", "error", "condition")
  producer <- c("producer_error_argument", "producer_error", base)
  consumer <- c("consumer_error_argument", "consumer_error", base)
  # counter_add_raw() hands its R value to the shim as is, as a caller that
  # skips the View does, so the implementing package's shim converts it, with
  # that package's classes.
  counters <- list(
    list(counter = new_counter(5L), class = producer),
    list(counter = new_stateful_counter(5L), class = producer),
    list(counter = new_double_counter(5L), class = consumer)
  )
  cases <- list(
    list(n = "five", message = "'n' must be a single integer: got character"),
    list(n = NA_integer_, message = "'n' must be a single integer: NA is not allowed"),
    list(n = 1:2, message = "'n' must be a single integer: got length 2")
  )
  for (impl in counters) {
    counter <- impl$counter
    for (case in cases) {
      e <- tryCatch(counter_add_raw(counter, case$n), error = function(e) e)
      expect_identical(e$kind, "conversion")
      expect_identical(e$param, "n")
      expect_identical(e$rust_type, "i32")
      expect_identical(class(e), impl$class)
      expect_true(startsWith(conditionMessage(e), case$message),
        info = conditionMessage(e))
      expect_true(grepl("counter_add_raw", deparse(conditionCall(e))[[1]]),
        info = paste(deparse(conditionCall(e)), collapse = " "))
    }
    # The failed calls left the counter alone, and the shim still works.
    expect_equal(peek_value(counter), 5L)
    counter_add_raw(counter, 2L)
    expect_equal(peek_value(counter), 7L)
  }
})

test_that("a trait method's argument error has one class on every path", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  base <- c("rust_error", "simpleError", "error", "condition")
  arg_error <- function(expr) tryCatch(expr, error = function(e) e)

  # producer.pkg implements Counter for SimpleCounter: its own wrapper and
  # consumer.pkg's call through the vtable raise producer.pkg's classes. The
  # trait is declared in shared-traits, whose classes never appear.
  producer <- c("producer_error_argument", "producer_error", base)
  direct <- arg_error(SimpleCounter$Counter$add(new_counter_external(5L), "five"))
  through_vtable <- arg_error(counter_add_raw(new_counter(5L), "five"))
  expect_identical(class(direct), producer)
  expect_identical(class(through_vtable), producer)
  expect_identical(direct$param, through_vtable$param)

  # consumer.pkg implements it for DoubleCounter: consumer.pkg's classes.
  consumer <- c("consumer_error_argument", "consumer_error", base)
  direct <- arg_error(DoubleCounter$create(5L)$Counter$add("five"))
  through_vtable <- arg_error(counter_add_raw(new_double_counter(5L), "five"))
  expect_identical(class(direct), consumer)
  expect_identical(class(through_vtable), consumer)
  expect_identical(direct$param, through_vtable$param)
})

test_that("a View's failed result conversion reaches the caller as rust_error", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  # value() returns i32::MIN, which crosses as NA_integer_ (#1419), and the
  # View cannot convert NA back to i32.
  counter <- new_counter(-2147483647L)
  e <- tryCatch(add_and_get(counter, -1L), error = function(e) e)
  expect_identical(class(e), c("rust_error", "simpleError", "error", "condition"))
  expect_true(startsWith(conditionMessage(e), "unexpected NA value in INTSXP"),
    info = conditionMessage(e))
  expect_true(grepl("add_and_get", deparse(conditionCall(e))[[1]]))

  # The call is repeatable: nothing was left half done.
  e2 <- tryCatch(peek_value(counter), error = function(e) e)
  expect_identical(class(e2), class(e))
})

# endregion
