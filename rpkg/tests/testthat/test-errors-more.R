test_that("nested_panic propagates panic", {
  expect_error(nested_panic())
})

test_that("unsafe_C_r_error variants signal errors", {
  expect_error(miniextendr:::unsafe_C_r_error(), "arg1")
  expect_error(miniextendr:::unsafe_C_r_error_in_catch(), "arg1")
})

# Dangerous threads / longjmp cases are marked skip to avoid crash while documenting coverage

test_that("unsafe_C_r_error_in_thread is currently unsafe", {
  skip("crashes when run; exported for demonstration")
  miniextendr:::unsafe_C_r_error_in_thread()
})

test_that("unsafe_C_r_print_in_thread is currently unsafe", {
  skip("interacts with stdout from worker thread; skip to avoid flakiness")
  miniextendr:::unsafe_C_r_print_in_thread()
})

test_that("unsafe_C_check_interupt_* return", {
  skip_on_cran()
  expect_null(miniextendr:::unsafe_C_check_interupt_after())
  expect_null(miniextendr:::unsafe_C_check_interupt_unwind())
})

test_that("unsafe_C_test_sexp_equality reports pointer vs semantic", {
  x <- c(1L, 2L)
  res1 <- miniextendr:::unsafe_C_test_sexp_equality(x, x)
  expect_true(res1$pointer_eq)
  expect_true(res1$semantic_eq)

  res2 <- miniextendr:::unsafe_C_test_sexp_equality(c(1L,2L), c(1L,2L))
  expect_false(res2$pointer_eq)
  expect_true(res2$semantic_eq)
})

test_that("unsafe_C_worker_drop_on_panic signals error", {
  expect_error(miniextendr:::unsafe_C_worker_drop_on_panic())
})
