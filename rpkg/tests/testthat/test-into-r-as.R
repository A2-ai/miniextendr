test_that("IntoRAs i64 to i32 roundtrip works", {
  result <- into_r_as_i64_to_i32(c(1L, 2L, 3L))
  expect_equal(result, c(1L, 2L, 3L))
})

test_that("IntoRAs preserves negative values", {
  result <- into_r_as_i64_to_i32(c(-10L, 0L, 10L))
  expect_equal(result, c(-10L, 0L, 10L))
})
