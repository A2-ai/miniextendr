test_that("ExternalPtr works from main-thread function", {
  expect_equal(test_extptr_on_main_thread(), 99L)
})
