# Tests for the condition macro system and RErrorAdapter

# region: RErrorAdapter structured error adapter (legacy path)

test_that("RErrorAdapter propagates parse error", {
  expect_error(test_condition_parse_int("not_a_number"), "invalid digit")
})

test_that("RErrorAdapter passes through on success", {
  expect_equal(test_condition_ok(), 42L)
})

test_that("RErrorAdapter includes cause chain in error message", {
  err <- tryCatch(test_condition_chained("abc"), error = function(e) e)
  expect_true(grepl("config error", err$message))
  expect_true(grepl("caused by", err$message))
  expect_true(grepl("invalid digit", err$message))
})

test_that("RErrorAdapter chained succeeds on valid input", {
  expect_equal(test_condition_chained("8"), 8L)
})

# endregion

# region: error!() macro

test_that("error!() raises rust_error class", {
  e <- tryCatch(demo_error("boom"), error = function(e) e)
  expect_true(inherits(e, "rust_error"))
  expect_true(inherits(e, "simpleError"))
  expect_true(inherits(e, "error"))
  expect_true(inherits(e, "condition"))
})

test_that("error!() message is preserved", {
  e <- tryCatch(demo_error("hello world"), error = function(e) e)
  expect_equal(conditionMessage(e), "hello world")
})

test_that("error!() class vector is c(rust_error, simpleError, error, condition)", {
  e <- tryCatch(demo_error("x"), error = function(e) e)
  expect_equal(class(e), c("rust_error", "simpleError", "error", "condition"))
})

test_that("error!() with custom class prepends to layered vector", {
  e <- tryCatch(demo_error_custom_class("my_error", "typed"), error = function(e) e)
  expect_equal(class(e)[1], "my_error")
  expect_true(inherits(e, "rust_error"))
  expect_true(inherits(e, "error"))
})

test_that("tryCatch(rust_error = h) catches error!()", {
  result <- tryCatch(
    demo_error("test"),
    rust_error = function(e) "caught_rust",
    error = function(e) "caught_generic"
  )
  expect_equal(result, "caught_rust")
})

test_that("tryCatch(custom_class = h) catches error!(class = 'custom_class', ...)", {
  result <- tryCatch(
    demo_error_custom_class("my_error", "msg"),
    my_error = function(e) "caught_custom",
    error = function(e) "caught_generic"
  )
  expect_equal(result, "caught_custom")
})

# endregion

# region: warning!() macro

test_that("warning!() raises rust_warning class", {
  w <- tryCatch(demo_warning("watch out"), warning = function(w) w)
  expect_true(inherits(w, "rust_warning"))
  expect_true(inherits(w, "simpleWarning"))
  expect_true(inherits(w, "warning"))
  expect_true(inherits(w, "condition"))
})

test_that("warning!() class vector is c(rust_warning, simpleWarning, warning, condition)", {
  w <- tryCatch(demo_warning("x"), warning = function(w) w)
  expect_equal(class(w), c("rust_warning", "simpleWarning", "warning", "condition"))
})

test_that("tryCatch(rust_warning = h) catches warning!()", {
  result <- tryCatch(
    demo_warning("oops"),
    rust_warning = function(w) "caught_rust_warning",
    warning = function(w) "caught_generic"
  )
  expect_equal(result, "caught_rust_warning")
})

test_that("withCallingHandlers continues execution after warning!()", {
  saw_warning <- FALSE
  result <- withCallingHandlers(
    {
      demo_warning("warn")
      42L
    },
    warning = function(w) {
      saw_warning <<- TRUE
      invokeRestart("muffleWarning")
    }
  )
  expect_true(saw_warning)
  expect_equal(result, 42L)
})

test_that("warning!() with custom class prepends to layered vector", {
  w <- tryCatch(
    demo_warning_custom_class("my_warning", "typed"),
    warning = function(w) w
  )
  expect_equal(class(w)[1], "my_warning")
  expect_true(inherits(w, "rust_warning"))
})

# endregion

# region: message!() macro

test_that("message!() raises rust_message class", {
  m <- tryCatch(demo_message("info"), message = function(m) m)
  expect_true(inherits(m, "rust_message"))
  expect_true(inherits(m, "simpleMessage"))
  expect_true(inherits(m, "message"))
  expect_true(inherits(m, "condition"))
})

test_that("message!() class vector is c(rust_message, simpleMessage, message, condition)", {
  m <- tryCatch(demo_message("x"), message = function(m) m)
  expect_equal(class(m), c("rust_message", "simpleMessage", "message", "condition"))
})

test_that("suppressMessages() muffles message!()", {
  # If message!() works correctly, suppressMessages should swallow it silently.
  expect_no_condition(suppressMessages(demo_message("silenced")))
})

# endregion

# region: condition!() macro

test_that("condition!() raises rust_condition class", {
  # withCallingHandlers does not unwind; use tryCatch to capture the signalled condition.
  c_val <- tryCatch(
    demo_condition("signal"),
    rust_condition = function(c) c
  )
  expect_true(inherits(c_val, "rust_condition"))
  expect_true(inherits(c_val, "simpleCondition"))
  expect_true(inherits(c_val, "condition"))
})

test_that("condition!() class vector is c(rust_condition, simpleCondition, condition)", {
  c_val <- tryCatch(
    demo_condition("x"),
    rust_condition = function(c) c
  )
  expect_equal(class(c_val), c("rust_condition", "simpleCondition", "condition"))
})

test_that("tryCatch(condition = h) catches condition!()", {
  result <- tryCatch(
    demo_condition("a signal"),
    condition = function(c) conditionMessage(c)
  )
  expect_equal(result, "a signal")
})

test_that("condition!() without handler is a no-op (returns NULL)", {
  # signalCondition with no handler → returns NULL invisibly
  result <- demo_condition("silent")
  expect_null(result)
})

test_that("condition!() with custom class catches via that class", {
  result <- tryCatch(
    demo_condition_custom_class("my_progress", "step 1"),
    my_progress = function(c) conditionMessage(c)
  )
  expect_equal(result, "step 1")
})

# endregion
