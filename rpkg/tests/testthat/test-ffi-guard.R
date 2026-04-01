test_that("ffi_guard CatchUnwind on non-panicking closure", {
  expect_equal(ffi_guard_catch_unwind_ok(), 42L)
})

test_that("ffi_guard fallback on non-panicking closure", {
  expect_equal(ffi_guard_fallback_ok(), 99L)
})

test_that("ffi_guard fallback on panicking closure returns fallback", {
  expect_equal(ffi_guard_fallback_panic(), -1L)
})
