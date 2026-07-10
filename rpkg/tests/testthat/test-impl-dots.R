test_that("R6 impl-block constructors and methods accept dots", {
  obj <- ImplDotsR6$new(10L, alpha = 1, beta = 2)

  expect_equal(obj$ctor_dots(), 2L)
  expect_equal(obj$add_with_dots(5L, gamma = 3, delta = 4), 19L)
  expect_equal(obj$explicit_dots(foo = 1, bar = 2, baz = 3), 3L)

  expect_identical(names(formals(ImplDotsR6$public_methods$initialize)), c("seed", "...", ".ptr"))
  expect_identical(names(formals(ImplDotsR6$public_methods$add_with_dots)), c("value", "..."))
  expect_identical(names(formals(ImplDotsR6$public_methods$explicit_dots)), "...")
})

test_that("S3 impl-block constructors and methods accept dots without duplicate dispatch dots", {
  obj <- new_impldotss3(20L, alpha = 1, beta = 2, gamma = 3)

  expect_s3_class(obj, "ImplDotsS3")
  expect_equal(impl_dots_s3_ctor_dots(obj), 3L)
  expect_equal(impl_dots_s3_add_with_dots(obj, 4L, extra = 5), 28L)

  generic <- impl_dots_s3_add_with_dots
  method <- getS3method("impl_dots_s3_add_with_dots", "ImplDotsS3")

  expect_identical(names(formals(new_impldotss3)), c("seed", "..."))
  expect_identical(names(formals(generic)), c("x", "..."))
  expect_identical(names(formals(method)), c("x", "value", "..."))
  expect_equal(anyDuplicated(names(formals(method))), 0L)
})

test_that("impl-block dots gc stress fixture is self-contained", {
  expect_equal(gc_stress_impl_dots_methods(), 42L)
})
