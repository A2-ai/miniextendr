test_that("objects can be returned as pointer, list, or native R type", {
  expect_identical(typeof(hybrid_as_ptr(2L)), "externalptr")
  expect_identical(typeof(hybrid_as_list(2L)), "list")
  expect_identical(typeof(hybrid_as_native(2L)), "integer")
  expect_identical(hybrid_as_native(2L), 2L)
})

test_that("return-type wrappers work with subset of representations", {
  expect_identical(typeof(ptr_list_as_ptr(5L)), "externalptr")
  expect_identical(typeof(ptr_list_as_list(5L)), "list")
  expect_identical(ptr_list_as_list(5L)[[1]], 5L)

  expect_identical(typeof(native_list_as_list(6L)), "list")
  expect_identical(native_list_as_list(6L)[[1]], 6L)
  expect_identical(typeof(native_list_as_native(6L)), "integer")
  expect_identical(native_list_as_native(6L), 6L)
})

