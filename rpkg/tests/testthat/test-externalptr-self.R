test_that("ExternalPtr self receiver works", {
  obj <- PtrSelfTest$new(42L)
  expect_equal(obj$value(), 42L)
  expect_false(obj$is_null_ptr())
  expect_equal(obj$value_via_ptr(), 42L)
})

test_that("Mutable ExternalPtr self receiver works", {
  obj <- PtrSelfTest$new(10L)
  obj$set_value_via_ptr(20L)
  expect_equal(obj$value(), 20L)
})

test_that("By-value ExternalPtr self receiver works", {
  obj <- PtrSelfTest$new(99L)
  expect_equal(obj$value_owned_ptr(), 99L)
})
