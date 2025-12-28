test_that("add_r_error signals clear thread error (pending main-thread dispatch fix)", {
  expect_error(add_r_error(1L, 2L), "non-main thread")
})

test_that("add_r_error_heap signals clear thread error (pending main-thread dispatch fix)", {
  expect_error(add_r_error_heap(1L, 2L), "non-main thread")
})

test_that("drop_on_panic_with_move signals clear thread error (pending main-thread dispatch fix)", {
  expect_error(drop_on_panic_with_move(), "non-main thread")
})
