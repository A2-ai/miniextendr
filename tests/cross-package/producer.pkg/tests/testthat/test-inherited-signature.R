test_that("generic defaults keep private declaration aliases across crates", {
  pointer <- new_counter_external(7L)
  expect_identical(SimpleCounter$AliasedValues$count(pointer), 7L)
  expect_identical(SimpleCounter$AliasedValues$echo(pointer, c(2L, 3L)), c(2L, 3L))
  err <- tryCatch(SimpleCounter$AliasedValues$echo(pointer, "wrong"), error = identity)
  expect_s3_class(err, "rust_error")
  expect_identical(err$kind, "conversion")
  expect_identical(err$param, "values")
})
