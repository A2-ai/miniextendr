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
  counter <- SimpleCounter$new(10L)

  # Object is an ExternalPtr that works across package boundaries
  expect_true(inherits(counter, "SimpleCounter"))
  expect_true(inherits(counter, "externalptr"))

  # Can be passed to functions expecting SEXP
  # In cross-package scenario, consumer would receive as SEXP parameter
  expect_equal(counter$get_value(), 10L)
})

test_that("Trait methods work via Type$Trait$method pattern", {
  # Create counter (producer.pkg)
  counter <- SimpleCounter$new(5L)

  # Call trait methods (these would be available in consumer.pkg too)
  expect_equal(SimpleCounter$Counter$value(counter), 5L)

  SimpleCounter$Counter$increment(counter)
  expect_equal(SimpleCounter$Counter$value(counter), 6L)

  SimpleCounter$Counter$add(counter, 10L)
  expect_equal(SimpleCounter$Counter$value(counter), 16L)

  SimpleCounter$Counter$reset(counter)
  expect_equal(SimpleCounter$Counter$value(counter), 0L)
})

test_that("Same trait methods work for different implementations", {
  # Two different Counter implementations
  simple <- SimpleCounter$new(100L)
  atomic <- AtomicCounter$new_atomic(100L)

  # Same trait interface works for both
  expect_equal(SimpleCounter$Counter$value(simple), 100L)
  expect_equal(AtomicCounter$Counter$value(atomic), 100L)

  # Increment both
  SimpleCounter$Counter$increment(simple)
  AtomicCounter$Counter$increment(atomic)

  expect_equal(SimpleCounter$Counter$value(simple), 101L)
  expect_equal(AtomicCounter$Counter$value(atomic), 101L)
})

test_that("Cross-package pattern: objects created in producer used via trait in consumer", {
  # Simulate cross-package workflow:

  # Step 1: producer.pkg creates object
  counter <- SimpleCounter$new(0L)

  # Step 2: Object passed through R to consumer.pkg
  # Consumer imports Counter trait but doesn't need SimpleCounter type definition

  # Step 3: Consumer uses trait methods (Type$Trait$method available in both packages)
  SimpleCounter$Counter$increment(counter)
  SimpleCounter$Counter$increment(counter)
  result <- SimpleCounter$Counter$value(counter)
  expect_equal(result, 2L)

  # Step 4: Can still use producer's inherent methods
  expect_equal(counter$get_value(), 2L)
})

test_that("Multiple Counter implementations can be used interchangeably", {
  # Create instances of different Counter implementations
  counters <- list(
    simple = SimpleCounter$new(1L),
    atomic = AtomicCounter$new_atomic(1L)
  )

  # Apply same operation to all via trait interface
  for (counter in counters) {
    # In consumer.pkg, this would be a single generic function
    # Here we show the R-level pattern works for both types
    if (inherits(counter, "SimpleCounter")) {
      SimpleCounter$Counter$add(counter, 5L)
      result <- SimpleCounter$Counter$value(counter)
    } else {
      AtomicCounter$Counter$add(counter, 5L)
      result <- AtomicCounter$Counter$value(counter)
    }
    expect_equal(result, 6L)
  }
})

test_that("Plain ExternalPtr passing works across package boundaries", {
  # Even without trait dispatch, ExternalPtr objects can cross packages
  counter <- SimpleCounter$new(42L)

  # In consumer.pkg, if it has SimpleCounter type definition,
  # it can receive as: fn process(counter: &mut SimpleCounter)
  # ExternalPtr handles the FFI marshaling

  expect_equal(counter$get_value(), 42L)
  expect_true(is(counter, "externalptr"))
})

test_that("Trait wrappers are type-specific but interface-compatible", {
  simple <- SimpleCounter$new(10L)
  atomic <- AtomicCounter$new_atomic(20L)

  # Each type has its own trait wrapper
  expect_equal(SimpleCounter$Counter$value(simple), 10L)
  expect_equal(AtomicCounter$Counter$value(atomic), 20L)

  # But they follow the same interface (Counter trait)
  SimpleCounter$Counter$add(simple, 5L)
  AtomicCounter$Counter$add(atomic, 5L)

  expect_equal(SimpleCounter$Counter$value(simple), 15L)
  expect_equal(AtomicCounter$Counter$value(atomic), 25L)
})
