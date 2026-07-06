# Tests for the condition macro system and AsRError

# region: AsRError structured error adapter (legacy path)

test_that("AsRError propagates parse error", {
  expect_error(miniextendr:::test_condition_parse_int("not_a_number"), "invalid digit")
})

test_that("AsRError passes through on success", {
  expect_equal(miniextendr:::test_condition_ok(), 42L)
})

test_that("AsRError includes cause chain in error message", {
  err <- tryCatch(miniextendr:::test_condition_chained("abc"), error = function(e) e)
  expect_true(grepl("config error", err$message))
  expect_true(grepl("caused by", err$message))
  expect_true(grepl("invalid digit", err$message))
})

test_that("AsRError chained succeeds on valid input", {
  expect_equal(miniextendr:::test_condition_chained("8"), 8L)
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

# region: data = ... structured payloads (#346)

test_that("error!(data = (..)) exposes the field as e$<name>", {
  e <- tryCatch(demo_error_data_scalar(150L), range_error = function(e) e)
  expect_equal(e$value, 150L)
  expect_equal(class(e)[1], "range_error")
  expect_true(inherits(e, "rust_error"))
  expect_equal(conditionMessage(e), "value 150 out of range")
})

test_that("error!(data = [..]) carries multiple heterogeneous fields", {
  e <- tryCatch(
    demo_error_data_multi(2.5, 7L, "lhs"),
    validation_error = function(e) e
  )
  expect_identical(e$value, 2.5)
  expect_identical(e$code, 7L)
  expect_identical(e$label, "lhs")
  expect_identical(e$fatal, TRUE)
})

test_that("error! data fields support vectors", {
  e <- tryCatch(
    demo_error_data_vector(c(101L, 205L)),
    batch_error = function(e) e
  )
  expect_identical(e$offending, c(101L, 205L))
})

test_that("data fields do not clobber message/call/kind", {
  e <- tryCatch(demo_error_data_scalar(150L), error = function(e) e)
  expect_equal(conditionMessage(e), "value 150 out of range")
  expect_equal(e$kind, "error")
  expect_false(is.null(conditionCall(e)))
})

test_that("handlers can route on data values for programmatic recovery", {
  recover <- function(value) {
    tryCatch(
      demo_error_data_scalar(value),
      range_error = function(e) min(max(e$value, 0L), 100L)
    )
  }
  expect_equal(recover(150L), 100L)
  expect_equal(recover(-3L), 0L)
})

test_that("warning!(data = ..) exposes fields on the warning condition", {
  w <- tryCatch(demo_warning_data(3L), truncation_warning = function(w) w)
  expect_identical(w$dropped, 3L)
  expect_true(inherits(w, "rust_warning"))
})

test_that("message!(data = ..) exposes fields on the message condition", {
  m <- tryCatch(demo_message_data(2L), message = function(m) m)
  expect_identical(m$step, 2L)
  expect_true(inherits(m, "rust_message"))
})

test_that("condition!(data = ..) exposes fields on the signalled condition", {
  c_val <- tryCatch(demo_condition_data(10L), progress = function(c) c)
  expect_identical(c_val$processed, 10L)
  expect_true(inherits(c_val, "rust_condition"))
})

test_that("conditions without data still have NULL extra fields", {
  e <- tryCatch(demo_error("plain"), error = function(e) e)
  expect_null(e$value)
  expect_null(e$data)
})

# endregion

# region: data = ... richer value types + keyed builder (#995)

test_that("Option fields round-trip: present is the value, missing is NA", {
  e_present <- tryCatch(
    demo_error_data_option(7L, TRUE),
    option_error = function(e) e
  )
  expect_identical(e_present$present, 7L)
  # has_value = TRUE so `missing` carries the value, not NA
  expect_identical(e_present$missing, 7L)

  e_missing <- tryCatch(
    demo_error_data_option(7L, FALSE),
    option_error = function(e) e
  )
  expect_identical(e_missing$present, 7L)
  # The field is *present* on the condition but its value is NA_integer_.
  expect_true("missing" %in% names(e_missing))
  expect_identical(e_missing$missing, NA_integer_)
})

test_that("Vec<Option<i32>> field carries embedded NA elements", {
  e <- tryCatch(demo_error_data_na_vector(), na_vector_error = function(e) e)
  expect_identical(e$codes, c(1L, NA_integer_, 3L))
})

test_that("i64 field within i32 range materialises as integer", {
  e <- tryCatch(demo_error_data_long(42), long_error = function(e) e)
  expect_identical(e$big, 42L)
})

test_that("i64 field beyond i32 range materialises as double", {
  e <- tryCatch(demo_error_data_long(5e9), long_error = function(e) e)
  expect_identical(e$big, 5e9)
  expect_type(e$big, "double")
})

test_that("nested named-list field is an R list readable as e$details$min", {
  e <- tryCatch(demo_error_data_nested(0L, 100L), nested_error = function(e) e)
  expect_true(is.list(e$details))
  expect_identical(e$details$min, 0L)
  expect_identical(e$details$max, 100L)
})

test_that("Debug-stringify fallback rides along as a character scalar", {
  e <- tryCatch(demo_error_data_debug(0L, 100L), debug_error = function(e) e)
  expect_identical(e$range, "0..=100")
})

test_that("keyed builder sugar produces the same fields as a pair list", {
  e <- tryCatch(demo_error_data_keyed(42L, 7L), keyed_error = function(e) e)
  expect_identical(e$value, 42L)
  expect_identical(e$code, 7L)
  expect_equal(class(e)[1], "keyed_error")
})

test_that("gc_stress_condition_data drives the richer payload path without error", {
  expect_null(miniextendr:::gc_stress_condition_data())
})

# endregion
