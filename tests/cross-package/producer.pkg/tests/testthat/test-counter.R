test_that("new_counter creates a Counter", {
  counter <- new_counter(5L)
  # counter_get_value uses trait dispatch
  expect_equal(counter_get_value(counter), 5L)
})

test_that("Counter trait methods work via trait dispatch", {
  counter <- new_counter(10L)

  # Test value() via counter_get_value (uses trait dispatch)
  expect_equal(counter_get_value(counter), 10L)
})

test_that("debug_tag_counter returns consistent tag", {
  tag1 <- debug_tag_counter()
  tag2 <- debug_tag_counter()

  # Tags should be consistent
  expect_equal(tag1, tag2)

  # Tag should be a 32-character hex string
  expect_equal(nchar(tag1), 32L)
})
