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

test_that("i32 slice boundary conditions work", {
  # Single element
  expect_equal(test_i32_slice_len(42L), 1L)
  expect_equal(test_i32_slice_sum(42L), 42L)
  expect_equal(test_i32_slice_first(42L), 42L)
  expect_equal(test_i32_slice_last(42L), 42L)

  # Two elements
  expect_equal(test_i32_slice_len(c(1L, 2L)), 2L)
  expect_equal(test_i32_slice_first(c(1L, 2L)), 1L)
  expect_equal(test_i32_slice_last(c(1L, 2L)), 2L)

  # Large values (near i32 limits)
  expect_equal(test_i32_slice_len(c(.Machine$integer.max, 1L)), 2L)
  expect_equal(test_i32_slice_first(c(.Machine$integer.max, 1L)), .Machine$integer.max)

  # Negative values
  expect_equal(test_i32_slice_sum(c(-10L, 5L, -3L)), -8L)
  expect_equal(test_i32_slice_first(c(-100L, 200L)), -100L)
})

test_that("f64 slice conversions work", {
  expect_equal(test_f64_slice_len(c(1.0, 2.0, 3.0)), 3L)
  expect_equal(test_f64_slice_len(numeric(0)), 0L)
  expect_equal(test_f64_slice_sum(c(1.0, 2.0, 3.0)), 6.0)
  expect_equal(test_f64_slice_sum(numeric(0)), 0.0)
  expect_equal(test_f64_slice_mean(c(2.0, 4.0, 6.0)), 4.0)
  expect_equal(test_f64_slice_mean(numeric(0)), 0.0)
})

test_that("f64 slice boundary conditions work", {
  # Single element
  expect_equal(test_f64_slice_len(3.14), 1L)
  expect_equal(test_f64_slice_sum(3.14), 3.14)
  expect_equal(test_f64_slice_mean(3.14), 3.14)

  # Two elements
  expect_equal(test_f64_slice_len(c(1.0, 2.0)), 2L)
  expect_equal(test_f64_slice_mean(c(1.0, 3.0)), 2.0)

  # Very small values
  expect_equal(test_f64_slice_sum(c(1e-10, 2e-10)), 3e-10)

  # Very large values
  expect_equal(test_f64_slice_len(c(1e100, 2e100)), 2L)

  # Negative values
  expect_equal(test_f64_slice_sum(c(-1.5, 2.5, -0.5)), 0.5)

  # Mixed positive/negative with mean
  expect_equal(test_f64_slice_mean(c(-10.0, 10.0)), 0.0)
})

test_that("u8 (raw) slice conversions work", {
  expect_equal(test_u8_slice_len(as.raw(1:5)), 5L)
  expect_equal(test_u8_slice_len(raw(0)), 0L)
  expect_equal(test_u8_slice_sum(as.raw(1:5)), 15L)
  expect_equal(test_u8_slice_sum(raw(0)), 0L)
})

test_that("u8 (raw) slice boundary conditions work", {
  # Single element
  expect_equal(test_u8_slice_len(as.raw(42)), 1L)
  expect_equal(test_u8_slice_sum(as.raw(42)), 42L)

  # Max value (255)
  expect_equal(test_u8_slice_sum(as.raw(255)), 255L)
  expect_equal(test_u8_slice_len(as.raw(c(0, 255))), 2L)

  # All zeros
  expect_equal(test_u8_slice_sum(as.raw(c(0, 0, 0))), 0L)
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

test_that("logical slice boundary conditions work", {
  # Single TRUE
  expect_equal(test_logical_slice_len(TRUE), 1L)
  expect_true(test_logical_slice_any_true(TRUE))
  expect_true(test_logical_slice_all_true(TRUE))

  # Single FALSE
  expect_equal(test_logical_slice_len(FALSE), 1L)
  expect_false(test_logical_slice_any_true(FALSE))
  expect_false(test_logical_slice_all_true(FALSE))

  # Two elements
  expect_equal(test_logical_slice_len(c(TRUE, FALSE)), 2L)
  expect_true(test_logical_slice_any_true(c(TRUE, FALSE)))
  expect_false(test_logical_slice_all_true(c(TRUE, FALSE)))

  # All TRUE
  expect_true(test_logical_slice_any_true(c(TRUE, TRUE, TRUE, TRUE)))
  expect_true(test_logical_slice_all_true(c(TRUE, TRUE, TRUE, TRUE)))

  # All FALSE
  expect_false(test_logical_slice_any_true(c(FALSE, FALSE, FALSE, FALSE)))
  expect_false(test_logical_slice_all_true(c(FALSE, FALSE, FALSE, FALSE)))
})
