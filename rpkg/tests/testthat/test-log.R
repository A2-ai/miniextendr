# Tests for log crate → R console routing.
#
# install_r_logger() defaults to LevelFilter::Off — every test that expects
# log output must opt in explicitly via test_log_set_level(). Each test
# resets to "off" on exit so it doesn't leak into the next.

test_that("log::info! outputs to R console", {
  test_log_set_level("info")
  on.exit(test_log_set_level("off"), add = TRUE)
  out <- capture.output(test_log_info("hello from rust"))
  expect_true(any(grepl("hello from rust", out)))
})

test_that("log::warn! outputs to R stderr", {
  test_log_set_level("warn")
  on.exit(test_log_set_level("off"), add = TRUE)
  # REprintf goes to stderr — capture via capture.output(type = "message")
  msg <- capture.output(test_log_warn("watch out"), type = "message")
  expect_true(any(grepl("watch out", msg)))
})

test_that("log::error! outputs to R stderr", {
  test_log_set_level("error")
  on.exit(test_log_set_level("off"), add = TRUE)
  msg <- capture.output(test_log_error("something broke"), type = "message")
  expect_true(any(grepl("something broke", msg)))
})

test_that("debug messages hidden at info level", {
  test_log_set_level("info")
  on.exit(test_log_set_level("off"), add = TRUE)
  out <- capture.output(test_log_debug("hidden debug"))
  expect_false(any(grepl("hidden debug", out)))
})

test_that("debug messages visible at debug level", {
  test_log_set_level("debug")
  on.exit(test_log_set_level("off"), add = TRUE)
  out <- capture.output(test_log_debug("visible debug"))
  expect_true(any(grepl("visible debug", out)))
})
