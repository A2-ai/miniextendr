# These tests verify that direct Rf_error() calls from Rust are caught and
# converted to R errors. The functions intentionally call Rf_error() to test
# the R_UnwindProtect boundary. On the main thread (the default), the longjmp
# is caught by with_r_unwind_protect and re-raised as an R condition.
test_that("add_r_error triggers R error via Rf_error longjmp", {
  expect_error(add_r_error(1L, 2L), "r error")
})

test_that("add_r_error_heap triggers R error via Rf_error longjmp", {
  expect_error(add_r_error_heap(1L, 2L), "r error")
})

test_that("drop_on_panic_with_move triggers R error", {
  expect_error(drop_on_panic_with_move(), "r error")
})
