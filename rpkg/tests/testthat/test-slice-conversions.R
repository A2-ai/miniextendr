# Test slice input conversions

test_that("i32 slice conversions work", {
  expect_equal(test_i32_slice_len(1:5), 5L)
  expect_equal(test_i32_slice_len(integer(0)), 0L)
  expect_equal(test_i32_slice_sum(1:5), 15L)
  expect_equal(test_i32_slice_sum(integer(0)), 0L)
  expect_equal(test_i32_slice_first(c(10L, 20L, 30L)), 10L)
  expect_equal(test_i32_slice_first(integer(0)), 0L)
  expect_equal(test_i32_slice_last(c(10L, 20L, 30L)), 30L)
  expect_equal(test_i32_slice_last(integer(0)), 0L)
})

test_that("f64 slice conversions work", {
  expect_equal(test_f64_slice_len(c(1.0, 2.0, 3.0)), 3L)
  expect_equal(test_f64_slice_len(numeric(0)), 0L)
  expect_equal(test_f64_slice_sum(c(1.0, 2.0, 3.0)), 6.0)
  expect_equal(test_f64_slice_sum(numeric(0)), 0.0)
  expect_equal(test_f64_slice_mean(c(2.0, 4.0, 6.0)), 4.0)
  expect_equal(test_f64_slice_mean(numeric(0)), 0.0)
})

test_that("u8 (raw) slice conversions work", {
  expect_equal(test_u8_slice_len(as.raw(1:5)), 5L)
  expect_equal(test_u8_slice_len(raw(0)), 0L)
  expect_equal(test_u8_slice_sum(as.raw(1:5)), 15L)
  expect_equal(test_u8_slice_sum(raw(0)), 0L)
})

test_that("logical slice conversions work", {
  expect_equal(test_logical_slice_len(c(TRUE, FALSE, TRUE)), 3L)
  expect_equal(test_logical_slice_len(logical(0)), 0L)
  expect_true(test_logical_slice_any_true(c(FALSE, FALSE, TRUE)))
  expect_false(test_logical_slice_any_true(c(FALSE, FALSE, FALSE)))
  expect_false(test_logical_slice_any_true(logical(0)))
  expect_true(test_logical_slice_all_true(c(TRUE, TRUE, TRUE)))
  expect_false(test_logical_slice_all_true(c(TRUE, FALSE, TRUE)))
  expect_true(test_logical_slice_all_true(logical(0)))  # vacuous truth
})
