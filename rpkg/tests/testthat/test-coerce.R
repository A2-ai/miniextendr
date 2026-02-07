# Test coercion system (Coerce/TryCoerce traits and #[miniextendr(coerce)] attribute)

test_that("identity coercion preserves value", {
  expect_equal(test_coerce_identity(42L), 42L)
  expect_equal(test_coerce_identity(-100L), -100L)
  expect_equal(test_coerce_identity(0L), 0L)
})

test_that("integer-to-double widening coercion works", {
  # i32 -> f64 widening
  expect_equal(test_coerce_widen(42L), 42.0)
  expect_equal(test_coerce_widen(-100L), -100.0)
})

test_that("Coerce Rboolean to i32 works", {
  expect_equal(test_coerce_bool_to_int(TRUE), 1L)
  expect_equal(test_coerce_bool_to_int(FALSE), 0L)
})

test_that("coercion via internal helper preserves value", {
  expect_equal(test_coerce_via_helper(42L), 42L)
})

test_that("TryCoerce f64 to i32 works", {
  # Exact values convert fine
  expect_equal(test_try_coerce_f64_to_i32(42.0), 42L)
  expect_equal(test_try_coerce_f64_to_i32(-100.0), -100L)

  # Values with fractional parts return NA (PrecisionLoss error)
  expect_true(is.na(test_try_coerce_f64_to_i32(42.9)))
  expect_true(is.na(test_try_coerce_f64_to_i32(-3.7)))

  # Out of range values return NA (i32::MIN = NA_INTEGER in R)
  expect_true(is.na(test_try_coerce_f64_to_i32(1e15)))
  expect_true(is.na(test_try_coerce_f64_to_i32(-1e15)))

  # NaN returns NA
  expect_true(is.na(test_try_coerce_f64_to_i32(NaN)))
})

# #[miniextendr(coerce)] attribute tests

test_that("coerce attribute works for u16", {
  expect_equal(test_coerce_attr_u16(100L), 100L)
  expect_equal(test_coerce_attr_u16(0L), 0L)
  expect_equal(test_coerce_attr_u16(65535L), 65535L)  # max u16

  # Negative values should error (overflow)
  expect_error(test_coerce_attr_u16(-1L))

  # Values > u16::MAX should error
  expect_error(test_coerce_attr_u16(65536L))
})

test_that("coerce attribute works for i16", {
  expect_equal(test_coerce_attr_i16(100L), 100L)
  expect_equal(test_coerce_attr_i16(-100L), -100L)
  expect_equal(test_coerce_attr_i16(0L), 0L)
  expect_equal(test_coerce_attr_i16(32767L), 32767L)   # max i16
  expect_equal(test_coerce_attr_i16(-32768L), -32768L) # min i16

  # Values > i16::MAX should error
  expect_error(test_coerce_attr_i16(32768L))
  expect_error(test_coerce_attr_i16(-32769L))
})

test_that("coerce attribute works for Vec<u16>", {
  expect_equal(test_coerce_attr_vec_u16(c(1L, 2L, 3L)), 6L)
  expect_equal(test_coerce_attr_vec_u16(c(100L, 200L, 300L)), 600L)
  expect_equal(test_coerce_attr_vec_u16(integer(0)), 0L)

  # Negative values should error
  expect_error(test_coerce_attr_vec_u16(c(1L, -1L, 3L)))
})

test_that("coerce attribute works for f32", {
  expect_equal(test_coerce_attr_f32(3.14), 3.14, tolerance = 1e-6)
  expect_equal(test_coerce_attr_f32(-100.5), -100.5, tolerance = 1e-6)
  expect_equal(test_coerce_attr_f32(0.0), 0.0)
})

test_that("coerced functions can return invisibly", {
  # Should work the same but return invisibly
  result <- withVisible(test_coerce_attr_with_invisible(42L))
  expect_equal(result$value, 42L)
  expect_false(result$visible)
})

# Per-argument coerce tests

test_that("per-argument coerce on first arg works", {
  # First arg (u16) is coerced, second (i32) is not
  expect_equal(test_per_arg_coerce_first(100L, 50L), 150L)
  expect_equal(test_per_arg_coerce_first(0L, 100L), 100L)

  # First arg overflow should error
  expect_error(test_per_arg_coerce_first(-1L, 50L))
})

test_that("per-argument coerce on second arg works", {
  # First arg (i32) is not coerced, second (u16) is
  expect_equal(test_per_arg_coerce_second(100L, 50L), 150L)
  expect_equal(test_per_arg_coerce_second(-100L, 50L), -50L)

  # Second arg overflow should error
  expect_error(test_per_arg_coerce_second(100L, -1L))
})

test_that("per-argument coerce on both args works", {
  # Both args coerced (u16, i16)
  expect_equal(test_per_arg_coerce_both(100L, 50L), 150L)
  expect_equal(test_per_arg_coerce_both(100L, -50L), 50L)

  # First arg (u16) overflow
  expect_error(test_per_arg_coerce_both(-1L, 50L))

  # Second arg (i16) overflow
  expect_error(test_per_arg_coerce_both(100L, 40000L))
})

test_that("per-argument coerce with Vec works", {
  # First arg is Vec<u16>, second is i32
  expect_equal(test_per_arg_coerce_vec(c(1L, 2L, 3L), 10L), 16L)
  expect_equal(test_per_arg_coerce_vec(c(100L, 200L), 0L), 300L)
  expect_equal(test_per_arg_coerce_vec(integer(0), 42L), 42L)

  # Vec with negative value should error
  expect_error(test_per_arg_coerce_vec(c(1L, -1L), 10L))
})
