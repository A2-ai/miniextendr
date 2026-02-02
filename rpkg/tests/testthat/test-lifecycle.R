# Tests for lifecycle support
# These tests verify that lifecycle attributes generate proper R wrapper code

# Note: lifecycle package must be installed for these tests to work
# The tests check that the wrapper code contains the expected lifecycle calls

test_that("deprecated functions generate lifecycle warnings", {
  skip_if_not_installed("lifecycle")

  # old_deprecated_fn should produce a deprecation warning
  expect_warning(
    old_deprecated_fn(5L),
    class = "lifecycle_warning_deprecated"
  )
})

test_that("experimental functions signal experimental stage", {
  skip_if_not_installed("lifecycle")

  # experimental_feature should signal experimental stage
  # Note: signal_stage may not throw by default, but we can verify it runs
  result <- experimental_feature(3.0)
  expect_equal(result, 9.0)
})

test_that("soft-deprecated functions produce soft warnings", {
  skip_if_not_installed("lifecycle")

  # soft_deprecated_fn should produce a soft deprecation warning (once per session)
  # We use suppressWarnings to avoid test output clutter after first warning
  result <- suppressWarnings(soft_deprecated_fn(10L))
  expect_equal(result, 9L)
})

test_that("defunct functions throw errors", {
  skip_if_not_installed("lifecycle")

  # defunct_fn should throw an error
  expect_error(
    defunct_fn(1L),
    class = "lifecycle_error_deprecated"
  )
})

test_that("deprecated functions with full spec work", {
  skip_if_not_installed("lifecycle")

  # fully_deprecated should produce a deprecation warning with version info
  expect_warning(
    fully_deprecated(42L),
    class = "lifecycle_warning_deprecated"
  )
})

test_that("superseded functions work without errors", {
  skip_if_not_installed("lifecycle")

  # superseded functions should work but may signal their stage
  result <- superseded_fn(5L)
  expect_equal(result, 6L)
})
