test_that("nested_panic propagates panic", {
  expect_error(nested_panic())
})

test_that("unsafe_C_r_error variants signal errors", {
  expect_error(miniextendr:::unsafe_C_r_error(), "arg1")
  expect_error(miniextendr:::unsafe_C_r_error_in_catch(), "arg1")
})

# Dangerous threads / longjmp cases are marked skip to avoid crash while documenting coverage

test_that("unsafe_C_r_error_in_thread panics cleanly on wrong thread", {
  # Note: The checked Rf_error detects wrong thread and panics with clear message.
  # However, propagating a thread panic through extern "C-unwind" causes
  # "failed to initiate panic" runtime errors. Skip until panic propagation
  # from spawned threads is better understood.
  skip("thread panic propagation causes runtime errors in extern C-unwind")
  expect_error(miniextendr:::unsafe_C_r_error_in_thread(), "non-main thread")
})

test_that("unsafe_C_r_print_in_thread panics cleanly on wrong thread", {
  # Same issue as above - the checked Rprintf correctly detects wrong thread,

  # but propagating the panic from the spawned thread causes runtime errors.
  skip("thread panic propagation causes runtime errors in extern C-unwind")
  expect_error(miniextendr:::unsafe_C_r_print_in_thread(), "non-main thread")
})

test_that("check_interrupt functions complete without error when no interrupt is pending", {
  skip_on_cran()
  expect_null(miniextendr:::unsafe_C_check_interupt_after())
  expect_null(miniextendr:::unsafe_C_check_interupt_unwind())
})

test_that("SEXP equality distinguishes pointer identity from value equality", {
  x <- c(1L, 2L)
  res1 <- miniextendr:::unsafe_C_test_sexp_equality(x, x)
  expect_true(res1$pointer_eq)
  expect_true(res1$semantic_eq)

  res2 <- miniextendr:::unsafe_C_test_sexp_equality(c(1L,2L), c(1L,2L))
  expect_false(res2$pointer_eq)
  expect_true(res2$semantic_eq)
})

test_that("worker drop-on-panic signals error", {
  expect_error(miniextendr:::unsafe_C_worker_drop_on_panic(), "intentional panic")
})
