test_that("IntoRAs i64 to i32 roundtrip works", {
  result <- into_r_as_i64_to_i32(c(1L, 2L, 3L))
  expect_equal(result, c(1L, 2L, 3L))
})

test_that("IntoRAs preserves negative values", {
  result <- into_r_as_i64_to_i32(c(-10L, 0L, 10L))
  expect_equal(result, c(-10L, 0L, 10L))
})

test_that("IntoRAs refuses -2^31, R's NA_integer_, as out of integer range (#1675)", {
  expect_identical(into_r_as_i64_scalar_to_i32(-2^31 + 1), -2147483647L)
  expect_identical(into_r_as_i64_scalar_to_i32(2^31 - 1), 2147483647L)
  # Not NA_integer_: the narrowing refuses it (the fixture's Err is a plain
  # error, reported through its Debug form).
  expect_error(
    into_r_as_i64_scalar_to_i32(-2^31),
    'OutOfRange { from: "i64", to: "i32"',
    fixed = TRUE
  )
})
