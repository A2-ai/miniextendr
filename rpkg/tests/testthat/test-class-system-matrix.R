# Tests for class system matrix: different trait impl styles
# All types use Env inherent impl (default), varying the trait impl style

# =============================================================================
# Env trait impl (CounterTraitEnv)
# =============================================================================

test_that("CounterTraitEnv works with Env trait impl", {
  counter <- CounterTraitEnv$new(10L)
  expect_true(inherits(counter, "CounterTraitEnv"))

  # Inherent method
  expect_equal(counter$get_value(), 10L)

  # Env trait method: standalone Type$Trait$method(x)
  expect_equal(CounterTraitEnv$MatrixCounter$custom_get(counter), 10L)
  CounterTraitEnv$MatrixCounter$custom_add(counter, 5L)
  expect_equal(CounterTraitEnv$MatrixCounter$custom_get(counter), 15L)

  # Env trait method: $ dispatch obj$Trait$method()
  expect_equal(counter$MatrixCounter$custom_get(), 15L)
  counter$MatrixCounter$custom_add(5L)
  expect_equal(counter$MatrixCounter$custom_get(), 20L)

  # Static trait method
  expect_equal(CounterTraitEnv$MatrixCounter$default_value(), 1L)
})

# =============================================================================
# S3 trait impl (CounterTraitS3)
# =============================================================================

test_that("CounterTraitS3 works with S3 trait impl", {
  counter <- CounterTraitS3$new(10L)
  expect_true(inherits(counter, "CounterTraitS3"))

  # Inherent method
  expect_equal(counter$get_value(), 10L)

  # S3 trait method: generic(x)
  expect_equal(custom_get(counter), 10L)
  custom_add(counter, 5L)
  expect_equal(custom_get(counter), 15L)

  # Static trait method (still uses Type$Trait$method pattern)
  expect_equal(CounterTraitS3$MatrixCounter$default_value(), 2L)
})

test_that("CounterTraitS3 direct trait helpers exist", {
  counter <- CounterTraitS3$new(3L)
  expect_equal(custom_get.CounterTraitS3(counter), 3L)
  custom_add.CounterTraitS3(counter, 4L)
  expect_equal(custom_get.CounterTraitS3(counter), 7L)
})

# =============================================================================
# S4 trait impl (CounterTraitS4)
# =============================================================================

test_that("CounterTraitS4 works with S4 trait impl", {
  counter <- CounterTraitS4$new(10L)
  expect_true(inherits(counter, "CounterTraitS4"))

  # Inherent method
  expect_equal(counter$get_value(), 10L)

  # S4 trait method: s4_trait_Trait_method(x)
  expect_equal(s4_trait_MatrixCounter_custom_get(counter), 10L)
  s4_trait_MatrixCounter_custom_add(counter, 5L)
  expect_equal(s4_trait_MatrixCounter_custom_get(counter), 15L)

  # Static trait method (standalone function)
  expect_equal(CounterTraitS4_MatrixCounter_default_value(), 3L)
})

# =============================================================================
# S7 trait impl (CounterTraitS7)
# =============================================================================

test_that("CounterTraitS7 works with S7 trait impl", {
  counter <- CounterTraitS7$new(10L)
  expect_true(inherits(counter, "CounterTraitS7"))

  # Inherent method
  expect_equal(counter$get_value(), 10L)

  # S7 trait method: s7_trait_Trait_method(x)
  expect_equal(s7_trait_MatrixCounter_custom_get(counter), 10L)
  s7_trait_MatrixCounter_custom_add(counter, 5L)
  expect_equal(s7_trait_MatrixCounter_custom_get(counter), 15L)

  # Static trait method
  expect_equal(CounterTraitS7$MatrixCounter$default_value(), 4L)
})

# =============================================================================
# R6 trait impl (CounterTraitR6)
# =============================================================================

test_that("CounterTraitR6 works with R6 trait impl", {
  counter <- CounterTraitR6$new(10L)
  expect_true(inherits(counter, "CounterTraitR6"))

  # Inherent method
  expect_equal(counter$get_value(), 10L)

  # R6 trait method: r6_trait_Trait_method(x)
  expect_equal(r6_trait_MatrixCounter_custom_get(counter), 10L)
  r6_trait_MatrixCounter_custom_add(counter, 5L)
  expect_equal(r6_trait_MatrixCounter_custom_get(counter), 15L)

  # Static trait method
  expect_equal(CounterTraitR6$MatrixCounter$default_value(), 5L)
})

# =============================================================================
# Cross-style compatibility tests
# =============================================================================

test_that("different trait impl styles work independently", {
  # Create counters with different trait styles
  env_counter <- CounterTraitEnv$new(100L)
  s3_counter <- CounterTraitS3$new(100L)
  s4_counter <- CounterTraitS4$new(100L)
  s7_counter <- CounterTraitS7$new(100L)
  r6_counter <- CounterTraitR6$new(100L)

  # Modify each using its trait style (env uses standalone for variety)
  CounterTraitEnv$MatrixCounter$custom_add(env_counter, 1L)
  custom_add(s3_counter, 2L)
  s4_trait_MatrixCounter_custom_add(s4_counter, 3L)
  s7_trait_MatrixCounter_custom_add(s7_counter, 4L)
  r6_trait_MatrixCounter_custom_add(r6_counter, 5L)

  # Verify each is independent (env uses $ dispatch for variety)
  expect_equal(env_counter$MatrixCounter$custom_get(), 101L)
  expect_equal(custom_get(s3_counter), 102L)
  expect_equal(s4_trait_MatrixCounter_custom_get(s4_counter), 103L)
  expect_equal(s7_trait_MatrixCounter_custom_get(s7_counter), 104L)
  expect_equal(r6_trait_MatrixCounter_custom_get(r6_counter), 105L)
})

# =============================================================================
# Regression: static method with first param named 'x'
# =============================================================================

test_that("static trait method with x param works via $ dispatch", {
  # Static method whose first param is 'x' - the old formals heuristic would
  # have misclassified this as an instance method and prepended self
  expect_equal(CounterTraitEnv$StaticXParam$from_value(10L), 20L)

  # Also works via $ dispatch on an instance (should NOT prepend self)
  counter <- CounterTraitEnv$new(5L)
  expect_equal(counter$StaticXParam$from_value(10L), 20L)
})

# =============================================================================
# Cross-style compatibility tests
# =============================================================================

test_that("static trait methods return different values per type", {
  # Each type has a different default_value
  expect_equal(CounterTraitEnv$MatrixCounter$default_value(), 1L)
  expect_equal(CounterTraitS3$MatrixCounter$default_value(), 2L)
  expect_equal(CounterTraitS4_MatrixCounter_default_value(), 3L)
  expect_equal(CounterTraitS7$MatrixCounter$default_value(), 4L)
  expect_equal(CounterTraitR6$MatrixCounter$default_value(), 5L)
})
