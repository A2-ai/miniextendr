test_that("unsafe_C_unwind_protect_normal() returns value and drops resources", {
  # Drop messages are printed to stderr but can't be reliably captured in testthat.
  # Visual verification: you should see "[Rust] Dropped: stack/heap resource" in output.
  result <- unsafe_C_unwind_protect_normal()
  expect_equal(result, 42L)
})

# Note: Tests for R error scenarios (unsafe_C_unwind_protect_r_error,
# unsafe_C_unwind_protect_lowlevel_test, add_r_error, etc.) are skipped
# in testthat because they can leave the R/Rust error handling in an
# inconsistent state when run in sequence. These tests work correctly
# when run individually via the smoke-test.
