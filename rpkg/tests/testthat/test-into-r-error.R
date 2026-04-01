test_that("IntoRError StringTooLong formats correctly", {
  msg <- into_r_error_string_too_long()
  expect_true(grepl("exceeds R", msg))
})

test_that("IntoRError LengthOverflow formats correctly", {
  msg <- into_r_error_length_overflow()
  expect_true(grepl("overflow", msg, ignore.case = TRUE))
})

test_that("IntoRError Inner formats correctly", {
  msg <- into_r_error_inner()
  expect_true(grepl("custom error message", msg))
})
