test_that("add_r_error signals R error", {
  expect_error(add_r_error(1L, 2L), "r error in `add_r_error`")
})

test_that("add_r_error_heap signals R error", {
  expect_error(add_r_error_heap(1L, 2L), "r error in `add_r_error`")
})

test_that("drop_on_panic_with_move signals R error", {
  expect_error(drop_on_panic_with_move(), "an r error occurred")
})
