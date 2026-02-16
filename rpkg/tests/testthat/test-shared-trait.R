# Cross-Package Trait Dispatch Tests
#
# These tests demonstrate miniextendr's cross-package interoperability pattern.
#
# Pattern: Objects created with one implementation can be manipulated via
# trait methods exposed through R wrappers. In a real multi-package scenario:
#
# - producer.pkg: Defines SharedCounter trait, implements for SimpleCounter
# - consumer.pkg: Imports SharedCounter trait, has functions using it
# - R user: Creates objects in producer, passes to consumer functions
#
# The trait methods (obj$Trait$method) provide the cross-package interface.

test_that("ExternalPtr objects can be passed and retain type identity", {
  # Create SimpleCounter (simulates producer.pkg)
  counter <- SharedSimpleCounter$new(10L)

  # Object is an ExternalPtr that works across package boundaries
  expect_true(inherits(counter, "SharedSimpleCounter"))

  # Can be passed to functions expecting SEXP
  # In cross-package scenario, consumer would receive as SEXP parameter
  expect_equal(counter$get_value(), 10L)
})

test_that("Trait methods work via both standalone and $ dispatch", {
  # Create counter (producer.pkg)
  counter <- SharedSimpleCounter$new(5L)

  # Standalone calling: Type$Trait$method(x)
  expect_equal(SharedSimpleCounter$SharedCounter$value(counter), 5L)

  # $ dispatch calling: obj$Trait$method()
  expect_equal(counter$SharedCounter$value(), 5L)

  # Mix both calling styles
  SharedSimpleCounter$SharedCounter$increment(counter)
  expect_equal(counter$SharedCounter$value(), 6L)

  counter$SharedCounter$add(10L)
  expect_equal(SharedSimpleCounter$SharedCounter$value(counter), 16L)

  SharedSimpleCounter$SharedCounter$reset(counter)
  expect_equal(counter$SharedCounter$value(), 0L)
})

test_that("Same trait methods work for different implementations", {
  # Two different Counter implementations
  simple <- SharedSimpleCounter$new(100L)
  atomic <- AtomicCounter$new_atomic(100L)

  # Standalone calling
  expect_equal(SharedSimpleCounter$SharedCounter$value(simple), 100L)
  expect_equal(AtomicCounter$SharedCounter$value(atomic), 100L)

  # $ dispatch calling
  expect_equal(simple$SharedCounter$value(), 100L)
  expect_equal(atomic$SharedCounter$value(), 100L)

  # Increment both (mix styles)
  simple$SharedCounter$increment()
  AtomicCounter$SharedCounter$increment(atomic)

  expect_equal(SharedSimpleCounter$SharedCounter$value(simple), 101L)
  expect_equal(atomic$SharedCounter$value(), 101L)
})

test_that("Trait methods and inherent methods can be mixed on the same object", {
  counter <- SharedSimpleCounter$new(0L)

  # Use trait methods via both calling conventions
  SharedSimpleCounter$SharedCounter$increment(counter)
  counter$SharedCounter$increment()
  expect_equal(SharedSimpleCounter$SharedCounter$value(counter), 2L)
  expect_equal(counter$SharedCounter$value(), 2L)

  # Inherent methods also see the updated state
  expect_equal(counter$get_value(), 2L)
})

test_that("Counter trait interface works for each implementation", {
  # Both implementations follow the same SharedCounter trait interface, supporting
  # both standalone Type$Trait$method(x) and obj$Trait$method() styles.
  simple <- SharedSimpleCounter$new(1L)
  atomic <- AtomicCounter$new_atomic(1L)

  # Standalone
  SharedSimpleCounter$SharedCounter$add(simple, 5L)
  expect_equal(SharedSimpleCounter$SharedCounter$value(simple), 6L)

  # $ dispatch
  atomic$SharedCounter$add(5L)
  expect_equal(atomic$SharedCounter$value(), 6L)
})

test_that("ExternalPtr objects retain values after creation", {
  counter <- SharedSimpleCounter$new(42L)
  expect_equal(counter$get_value(), 42L)
})

test_that("Trait wrappers are type-specific but interface-compatible", {
  simple <- SharedSimpleCounter$new(10L)
  atomic <- AtomicCounter$new_atomic(20L)

  # Each type has its own trait wrapper (mix both calling styles)
  expect_equal(SharedSimpleCounter$SharedCounter$value(simple), 10L)
  expect_equal(atomic$SharedCounter$value(), 20L)

  # But they follow the same interface (SharedCounter trait)
  simple$SharedCounter$add(5L)
  AtomicCounter$SharedCounter$add(atomic, 5L)

  expect_equal(simple$SharedCounter$value(), 15L)
  expect_equal(AtomicCounter$SharedCounter$value(atomic), 25L)
})
