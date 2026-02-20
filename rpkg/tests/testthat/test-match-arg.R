test_that("match_arg default (NULL) picks first choice", {
  expect_equal(match_arg_set_mode(), "Fast")
  expect_equal(match_arg_set_status(), "InProgress")
  expect_equal(match_arg_set_priority(), "Low")
})

test_that("match_arg exact match", {
  expect_equal(match_arg_set_mode("Fast"), "Fast")
  expect_equal(match_arg_set_mode("Safe"), "Safe")
  expect_equal(match_arg_set_mode("Debug"), "Debug")
})

test_that("match_arg partial match", {
  expect_equal(match_arg_set_mode("F"), "Fast")
  expect_equal(match_arg_set_mode("Sa"), "Safe")
  expect_equal(match_arg_set_mode("D"), "Debug")
})

test_that("match_arg rename_all snake_case", {
  expect_equal(match_arg_set_status("in_progress"), "InProgress")
  expect_equal(match_arg_set_status("completed"), "Completed")
  expect_equal(match_arg_set_status("not_started"), "NotStarted")
})

test_that("match_arg variant rename", {
  expect_equal(match_arg_set_priority("lo"), "Low")
  expect_equal(match_arg_set_priority("med"), "Medium")
  expect_equal(match_arg_set_priority("hi"), "High")
})

test_that("match_arg factor input", {
  expect_equal(match_arg_set_mode(factor("Fast")), "Fast")
  expect_equal(match_arg_set_mode(factor("Debug")), "Debug")
})

test_that("match_arg ambiguous partial errors", {
  # "S" matches both "Safe" and... no, only "Safe" starts with "S"

  # For Mode: "Fast", "Safe", "Debug" — no ambiguity possible with single-char prefixes
  # For Status: "in_progress", "completed", "not_started" — "n" matches "not_started" only
  # This is a valid partial
  expect_equal(match_arg_set_status("n"), "NotStarted")
})

test_that("match_arg invalid string errors", {
  expect_error(match_arg_set_mode("Invalid"), "should be one of")
  expect_error(match_arg_set_mode("X"), "should be one of")
})

test_that("match_arg non-character input errors", {
  expect_error(match_arg_set_mode(123), "must be NULL or a character vector")
  expect_error(match_arg_set_mode(TRUE), "must be NULL or a character vector")
})

test_that("match_arg mixed params", {
  expect_equal(match_arg_mixed(42L, "Safe"), "x=42, mode=Safe")
  # mode defaults to NULL (→ first choice)
  expect_equal(match_arg_mixed(42L), "x=42, mode=Fast")
})

test_that("match_arg with explicit default", {
  # Default is "Safe" (not NULL → first choice)
  expect_equal(match_arg_with_default(), "Safe")
  expect_equal(match_arg_with_default("Fast"), "Fast")
  expect_equal(match_arg_with_default("Debug"), "Debug")
})

test_that("match_arg choices accessors", {
  expect_equal(match_arg_mode_choices(), c("Fast", "Safe", "Debug"))
  expect_equal(match_arg_status_choices(), c("in_progress", "completed", "not_started"))
  expect_equal(match_arg_priority_choices(), c("lo", "med", "hi"))
})

test_that("match_arg return value is character", {
  result <- match_arg_return_mode("Debug")
  expect_equal(result, "Debug")
  expect_type(result, "character")
})

# ============================================================================
# choices() tests — idiomatic match.arg for string parameters
# ============================================================================

test_that("choices: default picks first choice", {
  # First choice is "pearson"
  expect_equal(choices_correlation(1.0, 2.0), "method=pearson, x=1, y=2")
  # First choice is "red"
  expect_equal(choices_color(), "color=red")
})

test_that("choices: exact match", {
  expect_equal(choices_correlation(1.0, 2.0, "pearson"), "method=pearson, x=1, y=2")
  expect_equal(choices_correlation(1.0, 2.0, "kendall"), "method=kendall, x=1, y=2")
  expect_equal(choices_correlation(1.0, 2.0, "spearman"), "method=spearman, x=1, y=2")
})

test_that("choices: partial match", {
  expect_equal(choices_correlation(1.0, 2.0, "p"), "method=pearson, x=1, y=2")
  expect_equal(choices_correlation(1.0, 2.0, "k"), "method=kendall, x=1, y=2")
  expect_equal(choices_correlation(1.0, 2.0, "sp"), "method=spearman, x=1, y=2")
})

test_that("choices: invalid value errors", {
  expect_error(choices_correlation(1.0, 2.0, "invalid"), "should be one of")
  expect_error(choices_color("purple"), "should be one of")
})

test_that("choices: mixed with regular params", {
  expect_equal(choices_mixed(42L, "fast", TRUE), "n=42, mode=fast, verbose=true")
  # mode defaults to first choice
  expect_equal(choices_mixed(42L, verbose = TRUE), "n=42, mode=fast, verbose=true")
})
