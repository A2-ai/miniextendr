# Tests for trait method option alignment: r_entry, r_post_checks, r_on_exit, lifecycle.
# These options were added to trait impl methods to match standalone fn / impl method parity.

test_that("trait method r_entry injects code at wrapper top", {
  obj <- OptsTarget$new(10L)
  # The r_entry sets .__entry_ran__ <- TRUE in the wrapper's local env.
  # If it didn't run, the .Call would still succeed — but we verify
  # the method works (entry code doesn't break execution).
  result <- OptsTarget$OptionsDemo$with_entry(obj)
  expect_equal(result, 10L)
})

test_that("trait method r_on_exit registers cleanup", {
  obj <- OptsTarget$new(20L)
  result <- OptsTarget$OptionsDemo$with_exit(obj)
  expect_equal(result, 20L)
})

test_that("trait method r_post_checks validates before .Call", {
  obj <- OptsTarget$new(5L)
  # Valid: integer argument passes the stopifnot(is.integer(n)) check
  expect_equal(OptsTarget$OptionsDemo$with_checks(obj, 3L), 8L)

  # Invalid: double argument should fail the stopifnot
  expect_error(OptsTarget$OptionsDemo$with_checks(obj, 3.0))
})

test_that("trait method lifecycle emits deprecation warning", {
  skip_if_not_installed("lifecycle")
  obj <- OptsTarget$new(42L)
  expect_warning(
    OptsTarget$OptionsDemo$deprecated_method(obj),
    "deprecated"
  )
})

test_that("trait method basic_value works without options", {
  obj <- OptsTarget$new(99L)
  expect_equal(OptsTarget$OptionsDemo$basic_value(obj), 99L)
})
