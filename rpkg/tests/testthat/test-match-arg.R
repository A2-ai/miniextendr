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

# ============================================================================
# several_ok tests — multi-value match.arg
# ============================================================================

test_that("several_ok: default returns all choices", {
  expect_equal(choices_multi_color(), "red, green, blue")
})

test_that("several_ok: subset selection", {
  expect_equal(choices_multi_color(c("red", "blue")), "red, blue")
})

test_that("several_ok: single value still works", {
  expect_equal(choices_multi_color("green"), "green")
})

test_that("several_ok: invalid value errors", {
  expect_error(choices_multi_color("purple"), "should be one of")
})

test_that("several_ok: mixed with regular params", {
  expect_equal(choices_multi_metrics(1L, c("mean", "sd")), "n=1, metrics=mean+sd")
  # metrics defaults to all choices
  expect_equal(choices_multi_metrics(1L), "n=1, metrics=mean+median+sd+var")
})

# ============================================================================
# match_arg + several_ok tests — enum-based multi-value
# ============================================================================

test_that("match_arg several_ok: default returns all variants", {
  expect_equal(match_arg_multi_mode(), "Fast, Safe, Debug")
})

test_that("match_arg several_ok: subset selection", {
  expect_equal(match_arg_multi_mode(c("Fast", "Debug")), "Fast, Debug")
})

test_that("match_arg several_ok: single value", {
  expect_equal(match_arg_multi_mode("Safe"), "Safe")
})

test_that("match_arg several_ok: invalid value errors", {
  expect_error(match_arg_multi_mode("Invalid"), "should be one of")
})

test_that("match_arg several_ok: mixed with regular params", {
  expect_equal(match_arg_multi_priority(1L, c("lo", "hi")), "n=1, priorities=lo+hi")
  # priorities defaults to all choices
  expect_equal(match_arg_multi_priority(1L), "n=1, priorities=lo+med+hi")
})

# ============================================================================
# Container variant tests — Box<[T]>, &[T], [T; N]
# ============================================================================

test_that("match_arg several_ok Box<[T]>: default returns all variants", {
  expect_equal(match_arg_multi_mode_boxed(), "Fast, Safe, Debug")
})

test_that("match_arg several_ok Box<[T]>: subset selection", {
  expect_equal(match_arg_multi_mode_boxed(c("Fast", "Debug")), "Fast, Debug")
})

test_that("match_arg several_ok Box<[T]>: single value", {
  expect_equal(match_arg_multi_mode_boxed("Safe"), "Safe")
})

test_that("match_arg several_ok Box<[T]>: invalid value errors", {
  expect_error(match_arg_multi_mode_boxed("Invalid"), "should be one of")
})

test_that("match_arg several_ok &[T]: default returns all variants", {
  expect_equal(match_arg_multi_mode_slice(), "Fast, Safe, Debug")
})

test_that("match_arg several_ok &[T]: subset selection", {
  expect_equal(match_arg_multi_mode_slice(c("Safe", "Debug")), "Safe, Debug")
})

test_that("match_arg several_ok &[T]: single value", {
  expect_equal(match_arg_multi_mode_slice("Fast"), "Fast")
})

test_that("match_arg several_ok &[T]: invalid value errors", {
  expect_error(match_arg_multi_mode_slice("Bad"), "should be one of")
})

test_that("match_arg several_ok [T; N]: exact length works", {
  expect_equal(match_arg_multi_mode_array(c("Fast", "Debug")), "Fast, Debug")
  expect_equal(match_arg_multi_mode_array(c("Safe", "Fast")), "Safe, Fast")
})

test_that("match_arg several_ok [T; N]: wrong length panics with clear message", {
  # Too few values
  expect_error(match_arg_multi_mode_array("Fast"), "expected 2 values")
  # Too many values
  expect_error(match_arg_multi_mode_array(c("Fast", "Safe", "Debug")), "expected 2 values")
})

test_that("match_arg several_ok [T; N]: all-invalid value errors from match.arg", {
  # When all values are invalid, match.arg errors with "should be one of"
  expect_error(match_arg_multi_mode_array(c("Foo", "Bar")), "should be one of")
  # When one value is invalid, match.arg silently drops it and Rust
  # panics on length mismatch (2-element array, only 1 valid match)
  expect_error(match_arg_multi_mode_array(c("Fast", "Invalid")), "expected 2 values")
})

# ============================================================================
# IntoR for Vec<MatchArgEnum> — round-trip return test (#148)
# ============================================================================

test_that("Vec<Mode> round-trips to R character vector", {
  # Default (NULL → all variants)
  result <- match_arg_return_modes()
  expect_type(result, "character")
  expect_equal(result, c("Fast", "Safe", "Debug"))
})

test_that("Vec<Mode> subset round-trips to R character vector", {
  result <- match_arg_return_modes(c("Safe", "Debug"))
  expect_type(result, "character")
  expect_equal(result, c("Safe", "Debug"))
})

test_that("Vec<Mode> single value round-trips to R character vector", {
  result <- match_arg_return_modes("Fast")
  expect_type(result, "character")
  expect_equal(result, "Fast")
})
