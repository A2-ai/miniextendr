test_that("fixed arrays match their Vec return conversions", {
  expect_identical(miniextendr:::fixed_array_native(), miniextendr:::fixed_array_native_vec())
  expect_identical(miniextendr:::fixed_array_i64(FALSE), miniextendr:::fixed_array_i64_vec(FALSE))
  expect_identical(miniextendr:::fixed_array_i64(TRUE), miniextendr:::fixed_array_i64_vec(TRUE))
  expect_identical(miniextendr:::fixed_array_i64(FALSE), 1:2)
  expect_identical(miniextendr:::fixed_array_i64(TRUE), c(1, 3e9))
  expect_identical(miniextendr:::fixed_array_strings(), miniextendr:::fixed_array_strings_vec())
  expect_identical(miniextendr:::fixed_array_bools(), miniextendr:::fixed_array_bools_vec())
  expect_identical(miniextendr:::fixed_array_empty_strings(), character())
  expect_identical(miniextendr:::fixed_array_optional_i64(TRUE), c(1, 3e9))
  expect_null(miniextendr:::fixed_array_optional_i64(FALSE))
})

test_that("lax and strict arrays agree in range and differ on overflow", {
  expect_identical(miniextendr:::lax_array_usize(FALSE), miniextendr:::strict_array_usize(FALSE))
  expect_identical(miniextendr:::lax_array_usize(TRUE), c(1, 3e9))
  expect_error(miniextendr:::strict_array_usize(TRUE), "strict conversion failed", fixed = TRUE)
})
