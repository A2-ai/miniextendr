test_that("SimpleCounter trait methods work via standalone and $ dispatch", {
  # Create a SimpleCounter
  counter <- SimpleCounter$new_counter(10L)
  expect_true(inherits(counter, "SimpleCounter"))

  # Test standalone calling: Type$Trait$method(x)
  expect_equal(SimpleCounter$Counter$value(counter), 10L)

  # Test $ dispatch calling: obj$Trait$method()
  expect_equal(counter$Counter$value(), 10L)

  # Test trait method: increment (both styles)
  SimpleCounter$Counter$increment(counter)
  expect_equal(counter$Counter$value(), 11L)

  counter$Counter$increment()
  expect_equal(SimpleCounter$Counter$value(counter), 12L)

  # Test trait method: checked_add (both styles)
  SimpleCounter$Counter$checked_add(counter, 5L)
  expect_equal(counter$Counter$value(), 17L)

  counter$Counter$checked_add(3L)
  expect_equal(SimpleCounter$Counter$value(counter), 20L)

  # Verify inherent method and trait method return same value
  expect_equal(counter$get_value(), counter$Counter$value())
})

test_that("PanickyCounter trait methods work via standalone and $ dispatch", {
  # Create a PanickyCounter
  counter <- PanickyCounter$new_panicky(5L)
  expect_true(inherits(counter, "PanickyCounter"))

  # Test trait method: value (both styles)
  expect_equal(PanickyCounter$Counter$value(counter), 5L)
  expect_equal(counter$Counter$value(), 5L)

  # Test trait method: increment ($ dispatch)
  counter$Counter$increment()
  expect_equal(PanickyCounter$Counter$value(counter), 6L)

  # Test trait method: checked_add with positive value (standalone)
  PanickyCounter$Counter$checked_add(counter, 10L)
  expect_equal(counter$Counter$value(), 16L)
})

test_that("PanickyCounter trait method panics are caught and converted to R errors", {
  counter <- PanickyCounter$new_panicky(5L)

  # Adding a negative value that would go below zero should panic (standalone)
  expect_error(
    PanickyCounter$Counter$checked_add(counter, -10L),
    "cannot go below zero"
  )
  expect_equal(PanickyCounter$Counter$value(counter), 5L)

  # Same via $ dispatch
  expect_error(
    counter$Counter$checked_add(-10L),
    "cannot go below zero"
  )
  expect_equal(counter$Counter$value(), 5L)
})

test_that("trait methods work with multiple instances independently", {
  counter1 <- SimpleCounter$new_counter(0L)
  counter2 <- SimpleCounter$new_counter(100L)

  # Modify counter1 via $ dispatch
  counter1$Counter$increment()
  counter1$Counter$increment()
  counter1$Counter$checked_add(10L)

  # Modify counter2 via standalone calling
  SimpleCounter$Counter$checked_add(counter2, -50L)

  # Verify independence (mix both calling styles)
  expect_equal(SimpleCounter$Counter$value(counter1), 12L)  # 0 + 1 + 1 + 10
  expect_equal(counter2$Counter$value(), 50L) # 100 - 50
})

test_that("trait methods via inherent and via trait namespace return same results", {
  counter <- SimpleCounter$new_counter(42L)

  # Call checked_add via trait_add (inherent method that wraps trait method)
  counter$trait_add(8L)
  val_after_inherent <- counter$get_value()

  # Call checked_add via standalone trait namespace
  SimpleCounter$Counter$checked_add(counter, 8L)
  val_after_standalone <- SimpleCounter$Counter$value(counter)

  # Call checked_add via $ dispatch
  counter$Counter$checked_add(8L)
  val_after_dispatch <- counter$Counter$value()

  # Each should have added 8
  expect_equal(val_after_inherent, 50L)
  expect_equal(val_after_standalone, 58L)
  expect_equal(val_after_dispatch, 66L)
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

test_that("S3TraitCounter S3 names work directly", {
  counter <- S3TraitCounter$new_s3trait(5L)
  expect_equal(value.S3TraitCounter(counter), 5L)

  increment.S3TraitCounter(counter)
  expect_equal(value.S3TraitCounter(counter), 6L)

  checked_add.S3TraitCounter(counter, 7L)
  expect_equal(value.S3TraitCounter(counter), 13L)
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

# =============================================================================
# S4 trait impl tests
# =============================================================================

test_that("S4TraitCounter trait methods work via S4 dispatch", {
  # Create an S4TraitCounter using inherent method (Env style)
  counter <- S4TraitCounter$new_s4trait(10L)
  expect_true(inherits(counter, "S4TraitCounter"))

  # Test S4 trait method: s4_trait_Counter_value
  expect_equal(s4_trait_Counter_value(counter), 10L)

  # Test S4 trait method: s4_trait_Counter_increment
  s4_trait_Counter_increment(counter)
  expect_equal(s4_trait_Counter_value(counter), 11L)

  # Test S4 trait method: s4_trait_Counter_checked_add
  s4_trait_Counter_checked_add(counter, 5L)
  expect_equal(s4_trait_Counter_value(counter), 16L)

  # Verify inherent method and S4 trait method return same value
  expect_equal(counter$get_value(), s4_trait_Counter_value(counter))
})

test_that("S4 trait static methods and associated constants work", {
  # S4 static trait methods are standalone functions: Type_Trait_method()
  # S4TraitCounter::default_initial() returns 40
  expect_equal(S4TraitCounter_Counter_default_initial(), 40L)

  # S4TraitCounter::MAX_VALUE is 400
  expect_equal(S4TraitCounter_Counter_MAX_VALUE(), 400L)
})

# =============================================================================
# S7 trait impl tests
# =============================================================================

test_that("S7TraitCounter trait methods work via S7 dispatch", {
  # Create an S7TraitCounter using inherent method (Env style)
  counter <- S7TraitCounter_new_s7trait(10L)
  expect_true(inherits(counter, "S7TraitCounter"))

  # Test S7 trait method: s7_trait_Counter_value
  expect_equal(s7_trait_Counter_value(counter), 10L)

  # Test S7 trait method: s7_trait_Counter_increment
  s7_trait_Counter_increment(counter)
  expect_equal(s7_trait_Counter_value(counter), 11L)

  # Test S7 trait method: s7_trait_Counter_checked_add
  s7_trait_Counter_checked_add(counter, 5L)
  expect_equal(s7_trait_Counter_value(counter), 16L)

  # Verify inherent method and S7 trait method return same value
  expect_equal(counter$get_value(), s7_trait_Counter_value(counter))
})

test_that("S7 trait static methods and associated constants work", {
  # S7 static trait methods use Type$Trait$method() (Env style)
  # S7TraitCounter::default_initial() returns 30
  # S7 trait statics are attached via attr() — access via attr()
  expect_equal(attr(S7TraitCounter, "Counter")$default_initial(), 30L)

  # S7TraitCounter::MAX_VALUE is 300
  expect_equal(attr(S7TraitCounter, "Counter")$MAX_VALUE(), 300L)
})

# =============================================================================
# R6 trait impl tests
# =============================================================================

test_that("R6TraitCounter trait methods work via standalone functions", {
  # Create an R6TraitCounter using inherent method (Env style)
  counter <- R6TraitCounter$new_r6trait(10L)
  expect_true(inherits(counter, "R6TraitCounter"))

  # Test R6 trait method: r6_trait_Counter_value
  expect_equal(r6_trait_Counter_value(counter), 10L)

  # Test R6 trait method: r6_trait_Counter_increment
  r6_trait_Counter_increment(counter)
  expect_equal(r6_trait_Counter_value(counter), 11L)

  # Test R6 trait method: r6_trait_Counter_checked_add
  r6_trait_Counter_checked_add(counter, 5L)
  expect_equal(r6_trait_Counter_value(counter), 16L)

  # Verify inherent method and R6 trait method return same value
  expect_equal(counter$get_value(), r6_trait_Counter_value(counter))
})

test_that("R6 trait static methods and associated constants work", {
  # R6 static trait methods use Type$Trait$method() (Env style)
  # R6TraitCounter::default_initial() returns 20
  expect_equal(R6TraitCounter$Counter$default_initial(), 20L)

  # R6TraitCounter::MAX_VALUE is 200
  expect_equal(R6TraitCounter$Counter$MAX_VALUE(), 200L)
})
