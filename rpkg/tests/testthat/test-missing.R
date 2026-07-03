# Tests for Missing<T> — distinguishes a *missing* argument from NULL and from
# a present value (see miniextendr-api/src/missing.rs rustdoc for semantics).

test_that("Missing<f64> distinguishes absent from present", {
  expect_equal(missing_test_f64(), "absent")
  expect_equal(missing_test_f64(2.5), "2.5")
})

test_that("Missing<String> unwrap_or_else falls back when absent", {
  expect_equal(missing_test_string(), "default_value")
  expect_equal(missing_test_string("hi"), "hi")
})

test_that("Missing<i32> is_present reflects argument presence", {
  expect_false(missing_test_present())
  expect_true(missing_test_present(1L))
})

test_that("Missing<Option<f64>> distinguishes missing, NULL, and present", {
  expect_equal(missing_test_option(), "missing")
  expect_equal(missing_test_option(NULL), "null")
  expect_equal(missing_test_option(3.5), "3.5")
})
