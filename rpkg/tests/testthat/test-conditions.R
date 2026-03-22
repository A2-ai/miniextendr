# Tests for RCondition structured error adapter

test_that("RCondition propagates parse error", {
  expect_error(test_condition_parse_int("not_a_number"), "invalid digit")
})

test_that("RCondition passes through on success", {
  expect_equal(test_condition_ok(), 42L)
})

test_that("RCondition includes cause chain in error message", {
  err <- tryCatch(test_condition_chained("abc"), error = function(e) e)
  # Should contain the top-level message AND the cause
  expect_true(grepl("config error", err$message))
  expect_true(grepl("caused by", err$message))
  expect_true(grepl("invalid digit", err$message))
})

test_that("RCondition chained succeeds on valid input", {
  expect_equal(test_condition_chained("8"), 8L)
})
