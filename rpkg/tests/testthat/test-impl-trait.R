test_that("impl IntoR return position with String", {
  expect_equal(impl_return_string(), "hello from impl IntoR")
})

test_that("impl IntoR return position with Vec<i32>", {
  expect_equal(impl_return_vec(), c(10L, 20L, 30L))
})
