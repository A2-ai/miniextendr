test_that("backtrace hook can be installed without error", {
  # Just verify the hook installs without crashing
  backtrace_install_hook()
  expect_true(TRUE)
})
