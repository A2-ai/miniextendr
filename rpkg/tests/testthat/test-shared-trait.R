# Cross-Package Trait Dispatch Tests
#
# These tests demonstrate miniextendr's cross-package interoperability pattern.
#
# Pattern: Objects created with one implementation can be manipulated via
# trait methods exposed through R wrappers. In a real multi-package scenario:
#
# - producer.pkg: Defines Counter trait, implements for SimpleCounter
# - consumer.pkg: Imports Counter trait, has functions using it
# - R user: Creates objects in producer, passes to consumer functions
#
# The trait methods (Type$Trait$method) provide the cross-package interface.

test_that("ExternalPtr objects can be passed and retain type identity", {
  # Create SimpleCounter (simulates producer.pkg)
  counter <- SharedSimpleCounter$new(10L)

  # Object is an ExternalPtr that works across package boundaries
  expect_true(inherits(counter, "SharedSimpleCounter"))

  # Can be passed to functions expecting SEXP
  # In cross-package scenario, consumer would receive as SEXP parameter
  expect_equal(counter$get_value(), 10L)
})

test_that("Trait methods work via Type$Trait$method pattern", {
  # Create counter (producer.pkg)
  counter <- SharedSimpleCounter$new(5L)

  # Call trait methods (these would be available in consumer.pkg too)
  expect_equal(SharedSimpleCounter$Counter$value(counter), 5L)

  SharedSimpleCounter$Counter$increment(counter)
  expect_equal(SharedSimpleCounter$Counter$value(counter), 6L)

  SharedSimpleCounter$Counter$add(counter, 10L)
  expect_equal(SharedSimpleCounter$Counter$value(counter), 16L)

  SharedSimpleCounter$Counter$reset(counter)
  expect_equal(SharedSimpleCounter$Counter$value(counter), 0L)
})

test_that("Same trait methods work for different implementations", {
  # Two different Counter implementations
  simple <- SharedSimpleCounter$new(100L)
  atomic <- AtomicCounter$new_atomic(100L)

  # Same trait interface works for both
  expect_equal(SharedSimpleCounter$Counter$value(simple), 100L)
  expect_equal(AtomicCounter$Counter$value(atomic), 100L)

  # Increment both
  SharedSimpleCounter$Counter$increment(simple)
  AtomicCounter$Counter$increment(atomic)

  expect_equal(SharedSimpleCounter$Counter$value(simple), 101L)
  expect_equal(AtomicCounter$Counter$value(atomic), 101L)
})

test_that("Trait methods and inherent methods can be mixed on the same object", {
  counter <- SharedSimpleCounter$new(0L)

  # Use trait methods (Type$Trait$method pattern)
  SharedSimpleCounter$Counter$increment(counter)
  SharedSimpleCounter$Counter$increment(counter)
  expect_equal(SharedSimpleCounter$Counter$value(counter), 2L)

  # Inherent methods also see the updated state
  expect_equal(counter$get_value(), 2L)
})

test_that("Counter trait interface works for each implementation", {
  # Both implementations follow the same trait interface (Type$Trait$method),
  # though R requires type-specific dispatch at the call site.
  simple <- SharedSimpleCounter$new(1L)
  atomic <- AtomicCounter$new_atomic(1L)

  SharedSimpleCounter$Counter$add(simple, 5L)
  expect_equal(SharedSimpleCounter$Counter$value(simple), 6L)

  AtomicCounter$Counter$add(atomic, 5L)
  expect_equal(AtomicCounter$Counter$value(atomic), 6L)
})

test_that("ExternalPtr objects retain values after creation", {
  counter <- SharedSimpleCounter$new(42L)
  expect_equal(counter$get_value(), 42L)
})

test_that("Trait wrappers are type-specific but interface-compatible", {
  simple <- SharedSimpleCounter$new(10L)
  atomic <- AtomicCounter$new_atomic(20L)

  # Each type has its own trait wrapper
  expect_equal(SharedSimpleCounter$Counter$value(simple), 10L)
  expect_equal(AtomicCounter$Counter$value(atomic), 20L)

  # But they follow the same interface (Counter trait)
  SharedSimpleCounter$Counter$add(simple, 5L)
  AtomicCounter$Counter$add(atomic, 5L)

  expect_equal(SharedSimpleCounter$Counter$value(simple), 15L)
  expect_equal(AtomicCounter$Counter$value(atomic), 25L)
})
