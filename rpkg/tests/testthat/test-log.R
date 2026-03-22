# Tests for log crate → R console routing

test_that("log::info! outputs to R console", {
  out <- capture.output(test_log_info("hello from rust"))
  expect_true(any(grepl("hello from rust", out)))
})

test_that("log::warn! outputs to R stderr", {
  # REprintf goes to stderr — capture via capture.output(type = "message")
  msg <- capture.output(test_log_warn("watch out"), type = "message")
  expect_true(any(grepl("watch out", msg)))
})

test_that("log::error! outputs to R stderr", {
  msg <- capture.output(test_log_error("something broke"), type = "message")
  expect_true(any(grepl("something broke", msg)))
})

test_that("debug messages hidden at info level", {
  test_log_set_level("info")
  out <- capture.output(test_log_debug("hidden debug"))
  expect_false(any(grepl("hidden debug", out)))
})

test_that("debug messages visible at debug level", {
  test_log_set_level("debug")
  out <- capture.output(test_log_debug("visible debug"))
  expect_true(any(grepl("visible debug", out)))
  # Reset to info
  test_log_set_level("info")
})
