test_that("test_extptr_on_main_thread returns value", {
  expect_equal(test_extptr_on_main_thread(), 99L)
})
