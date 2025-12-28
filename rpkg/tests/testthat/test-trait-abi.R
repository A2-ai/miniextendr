test_that("SimpleCounter trait methods work via .Call wrappers", {
  # Create a SimpleCounter
  counter <- SimpleCounter$new_counter(10L)
  expect_true(inherits(counter, "SimpleCounter"))

  # Test trait method: value
  expect_equal(counter$Counter$value(), 10L)

  # Test trait method: increment
  counter$Counter$increment()
  expect_equal(counter$Counter$value(), 11L)

  # Test trait method: checked_add
  counter$Counter$checked_add(5L)
  expect_equal(counter$Counter$value(), 16L)

  # Verify inherent method and trait method return same value
  expect_equal(counter$get_value(), counter$Counter$value())
})

test_that("PanickyCounter trait methods work via .Call wrappers", {
  # Create a PanickyCounter
  counter <- PanickyCounter$new_panicky(5L)
  expect_true(inherits(counter, "PanickyCounter"))

  # Test trait method: value
  expect_equal(counter$Counter$value(), 5L)

  # Test trait method: increment
  counter$Counter$increment()
  expect_equal(counter$Counter$value(), 6L)

  # Test trait method: checked_add with positive value (should work)
  counter$Counter$checked_add(10L)
  expect_equal(counter$Counter$value(), 16L)
})

test_that("PanickyCounter trait method panics are caught and converted to R errors", {
  counter <- PanickyCounter$new_panicky(5L)

  # Adding a negative value that would go below zero should panic
  # The panic should be caught and converted to an R error
  expect_error(
    counter$Counter$checked_add(-10L),
    "cannot go below zero"
  )

  # Counter value should be unchanged after the caught panic
  expect_equal(counter$Counter$value(), 5L)
})

test_that("trait methods work with multiple instances independently", {
  counter1 <- SimpleCounter$new_counter(0L)
  counter2 <- SimpleCounter$new_counter(100L)

  # Modify counter1 via trait methods
  counter1$Counter$increment()
  counter1$Counter$increment()
  counter1$Counter$checked_add(10L)

  # Modify counter2 via trait methods
  counter2$Counter$checked_add(-50L)

  # Verify independence

  expect_equal(counter1$Counter$value(), 12L)  # 0 + 1 + 1 + 10
  expect_equal(counter2$Counter$value(), 50L) # 100 - 50
})

test_that("trait methods via inherent and via trait namespace return same results", {
  counter <- SimpleCounter$new_counter(42L)

  # Call checked_add via trait_add (inherent method that wraps trait method)
  counter$trait_add(8L)
  val_after_inherent <- counter$get_value()

  # Call checked_add via trait namespace
  counter$Counter$checked_add(8L)
  val_after_trait <- counter$Counter$value()

  # Both should have added 8
  expect_equal(val_after_inherent, 50L)
  expect_equal(val_after_trait, 58L)
})

test_that("static trait methods work", {
  # Static trait methods can be called without an instance
  # They are accessed via Type$Trait$static_method()

  # SimpleCounter::default_initial() returns 0
  expect_equal(SimpleCounter$Counter$default_initial(), 0L)

  # PanickyCounter::default_initial() returns 100
  expect_equal(PanickyCounter$Counter$default_initial(), 100L)
})

test_that("trait associated constants work", {
  # Trait associated constants can be accessed without an instance
  # They are accessed via Type$Trait$CONST_NAME()

  # SimpleCounter::MAX_VALUE is i32::MAX
  expect_equal(SimpleCounter$Counter$MAX_VALUE(), .Machine$integer.max)

  # PanickyCounter::MAX_VALUE is 1000
  expect_equal(PanickyCounter$Counter$MAX_VALUE(), 1000L)
})

# =============================================================================
# S3 trait dispatch tests (for #[miniextendr(s3)] impl Trait for Type)
# =============================================================================

test_that("S3TraitCounter trait methods work via S3 dispatch", {
  # Create an S3TraitCounter
  counter <- S3TraitCounter$new_s3trait(10L)
  expect_true(inherits(counter, "S3TraitCounter"))

  # Test S3 generic dispatch: value(x) instead of x$Counter$value()
  expect_equal(value(counter), 10L)

  # Test S3 generic dispatch: increment(x)
  increment(counter)
  expect_equal(value(counter), 11L)

  # Test S3 generic dispatch: checked_add(x, n)
  checked_add(counter, 5L)
  expect_equal(value(counter), 16L)

  # Verify inherent method and S3 trait method return same value
  expect_equal(counter$get_value(), value(counter))
})

test_that("S3 trait methods work with multiple instances independently", {
  counter1 <- S3TraitCounter$new_s3trait(0L)
  counter2 <- S3TraitCounter$new_s3trait(100L)

  # Modify counter1 via S3 trait methods
  increment(counter1)
  increment(counter1)
  checked_add(counter1, 10L)

  # Modify counter2 via S3 trait methods
  checked_add(counter2, -50L)

  # Verify independence
  expect_equal(value(counter1), 12L)  # 0 + 1 + 1 + 10
  expect_equal(value(counter2), 50L)  # 100 - 50
})

test_that("S3 trait static methods and associated constants work", {
  # Static trait methods for S3 types still use Type$Trait$method() syntax
  # S3TraitCounter::default_initial() returns 50
  expect_equal(S3TraitCounter$Counter$default_initial(), 50L)

  # S3TraitCounter::MAX_VALUE is 500
  expect_equal(S3TraitCounter$Counter$MAX_VALUE(), 500L)
})
