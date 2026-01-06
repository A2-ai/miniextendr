test_that("greetings variants handle dots correctly", {
  # All named/unnamed dots variants now work correctly after macro fix
  expect_null(greetings_with_named_dots(a = 1, b = 2))
  expect_null(greetings_with_named_and_unused_dots(a = 1))
  expect_null(greetings_with_nameless_dots(1, 2, 3))
  expect_null(greetings_last_as_named_dots(1L, x = 2))
  expect_null(greetings_last_as_named_and_unused_dots(1L, x = 2))
  expect_null(greetings_last_as_nameless_dots(1L, 2, 3))
})
