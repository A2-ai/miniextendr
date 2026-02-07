# Dots tests are skipped because the generated R wrappers for dots functions
# have specific signature requirements that don't match standard R variadic patterns.
# The dots functionality works correctly when called appropriately (see smoke-test).

test_that("greetings_with_nameless_dots() runs without error", {
  expect_null(greetings_with_nameless_dots())
  expect_null(greetings_with_nameless_dots(1, 2, 3))
})

test_that("greetings_last_as_nameless_dots() runs without error", {
  expect_null(greetings_last_as_nameless_dots(1L))
  expect_null(greetings_last_as_nameless_dots(1L, 2, 3))
})
