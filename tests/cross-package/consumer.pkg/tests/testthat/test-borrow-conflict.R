# A producer method that calls back into R holds its object's borrow. A
# consumer call that reaches the same object meets it, and the conflict error
# keeps its class across the trait-ABI boundary (#1965).

test_that("a borrow conflict in the producer reaches R with its class", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  counter <- new_counter(1L)
  expect_error(
    add_around(counter, function() peek_value(counter)),
    class = "miniextendr_borrow_conflict"
  )
  expect_error(
    add_around(counter, function() add_and_get(counter, 5L)),
    class = "miniextendr_borrow_conflict"
  )
  # The refused calls released every borrow and left the value alone.
  expect_identical(add_around(counter, function() NULL), 2L)
  expect_identical(peek_value(counter), 2L)
})

test_that("a borrow of one producer object doesn't hold another", {
  skip_if_not_installed("producer.pkg")
  library(producer.pkg)

  a <- new_counter(1L)
  b <- new_counter(10L)
  expect_identical(add_around(a, function() add_and_get(b, 5L)), 2L)
  expect_identical(peek_value(b), 15L)
})
