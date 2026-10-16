test_that("inherited generic defaults dispatch across DLLs with private aliases", {
  pointer <- producer.pkg::new_counter(7L)
  expect_identical(inherited_values(pointer, c(2L, 3L)), c(2L, 3L))
  err <- tryCatch(inherited_values(pointer, "wrong"), error = identity)
  expect_s3_class(err, "rust_error")
  expect_identical(err$kind, "conversion")
  expect_identical(err$param, "values")
})
