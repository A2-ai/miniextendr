# Test scalar input/output conversions

test_that("i32 scalar conversions work", {
  expect_equal(test_i32_identity(42L), 42L)
  expect_equal(test_i32_add_one(41L), 42L)
  expect_equal(test_i32_sum(1L, 2L, 3L), 6L)
  expect_equal(test_i32_sum(-10L, 5L, 5L), 0L)
})

test_that("f64 scalar conversions work", {
  expect_equal(test_f64_identity(3.14), 3.14)
  expect_equal(test_f64_add_one(2.5), 3.5)
  expect_equal(test_f64_multiply(2.0, 3.0), 6.0)
  expect_equal(test_f64_multiply(-2.5, 4.0), -10.0)
})

test_that("u8 (raw) scalar conversions work", {
  expect_equal(test_u8_identity(as.raw(42)), as.raw(42))
  expect_equal(test_u8_add_one(as.raw(41)), as.raw(42))
  expect_equal(test_u8_add_one(as.raw(255)), as.raw(0))  # wrapping
})

test_that("logical scalar conversions work", {
  expect_true(test_logical_identity(TRUE))
  expect_false(test_logical_identity(FALSE))
  expect_false(test_logical_not(TRUE))
  expect_true(test_logical_not(FALSE))
  expect_true(test_logical_and(TRUE, TRUE))
  expect_false(test_logical_and(TRUE, FALSE))
  expect_false(test_logical_and(FALSE, TRUE))
  expect_false(test_logical_and(FALSE, FALSE))
})

test_that("mixed type conversions work", {
  expect_equal(test_i32_to_f64(42L), 42.0)
  expect_equal(test_i32_to_f64(-10L), -10.0)
  expect_equal(test_f64_to_i32(42.9), 42L)
  expect_equal(test_f64_to_i32(-3.7), -3L)
})

test_that("strict i64 conversion succeeds for in-range values", {
  expect_equal(strict_echo_i64(42L), 42L)
  expect_equal(strict_echo_i64(0L), 0L)
  expect_equal(strict_echo_i64(-1L), -1L)
})

test_that("strict i64 conversion errors for out-of-range values", {
  # i64 value outside i32 range should produce R error
  expect_error(strict_echo_i64(2^31), "strict conversion failed")
  expect_error(strict_echo_i64(-2^31), "strict conversion failed")
})

test_that("strict Vec<i64> conversion succeeds for in-range values", {
  expect_equal(strict_echo_vec_i64(c(1L, 2L, 3L)), c(1L, 2L, 3L))
})

test_that("strict Vec<i64> conversion errors for out-of-range values", {
  expect_error(strict_echo_vec_i64(c(1, 2^31)), "strict conversion failed")
})
