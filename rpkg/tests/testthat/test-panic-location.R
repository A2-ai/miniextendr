# Rust panic source location surfaced into R error messages (#1121 / location feature).
#
# Fixtures: rpkg/src/rust/panic_location_tests.rs.
# A generic panic!'s R error message must end with `\n(at <file>:<line>)`
# pointing at the real panic! site; typed conditions (error!/Err/None) must not.

# Grab conditionMessage for a call that is expected to error.
cond_msg <- function(expr) {
  e <- tryCatch(expr, error = function(e) e)
  if (!inherits(e, "condition")) {
    stop("expected an error condition, got a normal value")
  }
  conditionMessage(e)
}

# Regex for the surfaced location, scoped to the fixture's own source file.
loc_re <- "\\(at .*panic_location_tests\\.rs:[0-9]+\\)"

test_that("direct panic surfaces the Rust (at file:line)", {
  msg <- cond_msg(panic_location_main_direct())
  expect_match(msg, "boom-main-direct", fixed = TRUE)
  expect_match(msg, loc_re)
})

test_that("nested panic points at the plain helper's panic!", {
  msg <- cond_msg(panic_location_main_nested())
  expect_match(msg, "boom-nested", fixed = TRUE)
  expect_match(msg, loc_re)
})

# Regression guards: the typed branches (error!/warning!/message!/condition!,
# Result::Err, Option::None) are untouched by the location feature and must
# carry NO `(at …)` suffix.

test_that("error!() gets no location suffix", {
  msg <- cond_msg(panic_location_regression_error())
  expect_match(msg, "regression-error-no-location", fixed = TRUE)
  expect_false(grepl("(at ", msg, fixed = TRUE))
})

test_that("Result::Err return gets no location suffix", {
  msg <- cond_msg(panic_location_regression_result_err())
  expect_match(msg, "regression-result-err-no-location", fixed = TRUE)
  expect_false(grepl("(at ", msg, fixed = TRUE))
})

test_that("Option::None return gets no location suffix", {
  msg <- cond_msg(panic_location_regression_option_none())
  expect_false(grepl("(at ", msg, fixed = TRUE))
})
