test_that("add_panic() converts panic to R error", {
  expect_error(add_panic(1L, 2L), "we cannot add right now")
})

test_that("add_panic_heap() converts panic to R error (heap allocated)", {
  expect_error(add_panic_heap(1L, 2L), "we cannot add right now")
})

test_that("drop_message_on_success() returns value", {
  # Drop messages are printed to stdout but can't be reliably captured in testthat.
  # Visual verification: you should see "[Rust] Dropped `MsgOnDrop`!" in test output.
  result <- drop_message_on_success()
  expect_equal(result, 42L)
})

test_that("drop_on_panic() converts panic to R error", {
  # Drop messages are printed to stdout but can't be reliably captured in testthat.
  # Visual verification: you should see "[Rust] Dropped `MsgOnDrop`!" in test output.
  expect_error(drop_on_panic())
})

# Note: unsafe_C_just_panic() and unsafe_C_panic_and_catch() are not tested here
# because they demonstrate edge cases with extern "C" panic handling that can
# abort R when run in testthat's multi-run context. They work correctly when
# called directly (see smoke-test).
