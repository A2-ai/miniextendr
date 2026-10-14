# The shared Tagged trait takes the owner in every method, so its vtable slots
# are mx_owner_meth and its tag carries the receiver shapes. consumer.pkg calls
# producer.pkg's implementation through TaggedView, which passes the owner it
# roots; the methods read and write a sidecar in that owner.

test_that("a View passes its owner to another package's owner receivers", {
  skip_if_not_installed("producer.pkg")
  obj <- producer.pkg::new_labelled("first")
  expect_identical(tagged_add_and_read(obj, "second"), c("first", "second"))
  gc(full = TRUE)
  expect_identical(tagged_add_and_read(obj, "third"), c("first", "second", "third"))
})

test_that("an object without the trait is refused", {
  skip_if_not_installed("producer.pkg")
  expect_error(
    tagged_add_and_read(producer.pkg::new_counter(1L), "x"),
    "does not implement Tagged"
  )
})
