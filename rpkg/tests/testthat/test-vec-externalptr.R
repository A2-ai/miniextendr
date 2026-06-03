# Tests for Vec<ExternalPtr<T>> across the R boundary (issue #827).

test_that("Vec<ExternalPtr<T>> return side produces a list() of external pointers", {
  bags <- veptr_make_bags(4L)
  expect_type(bags, "list")
  expect_length(bags, 4L)
  for (b in bags) {
    expect_true(inherits(b, "externalptr"))
  }
})

test_that("Vec<ExternalPtr<T>> argument side accepts a list() of external pointers", {
  bags <- veptr_make_bags(5L)
  # values are 1..=5, so the sum is 15
  expect_equal(veptr_sum_bags(bags), 15L)
})

test_that("Vec<ExternalPtr<T>> round-trips through both directions", {
  bags <- veptr_make_bags(3L)        # values 1, 2, 3
  incremented <- veptr_increment_bags(bags)  # values 2, 3, 4
  expect_type(incremented, "list")
  expect_length(incremented, 3L)
  expect_equal(veptr_sum_bags(incremented), 9L)
  # original handles are untouched
  expect_equal(veptr_sum_bags(bags), 6L)
})

test_that("empty list() round-trips as an empty Vec<ExternalPtr<T>>", {
  empty <- veptr_make_bags(0L)
  expect_type(empty, "list")
  expect_length(empty, 0L)
  expect_equal(veptr_sum_bags(empty), 0L)
  expect_equal(veptr_sum_bags(list()), 0L)
})

test_that("a non-list argument is rejected", {
  expect_error(veptr_sum_bags(1L))
})

test_that("Vec<Option<ExternalPtr<T>>> maps NULL elements to None", {
  # even indices (0-based) are NULL → 4 NULLs, 4 handles for n = 8
  holey <- veptr_make_bags_with_holes(8L)
  expect_type(holey, "list")
  expect_length(holey, 8L)
  expect_null(holey[[1L]])         # index 0 → NULL
  expect_true(inherits(holey[[2L]], "externalptr"))  # index 1 → handle
  expect_equal(veptr_count_some(holey), 4L)
})

test_that("Vec<Option<ExternalPtr<T>>> argument accepts a hand-built mixed list", {
  bag <- veptr_make_bags(1L)[[1L]]
  expect_equal(veptr_count_some(list(bag, NULL, bag, NULL, NULL)), 2L)
  expect_equal(veptr_count_some(list()), 0L)
})

test_that("gc_stress fixture drives both directions without corruption", {
  # Smoke test; the real value is the gctorture sweep, but make sure the
  # no-arg fixture runs and its internal assertions hold.
  expect_null(gc_stress_vec_externalptr())
})
