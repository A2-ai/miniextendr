# Tests for class system matrix: different trait impl styles
# All types use Env inherent impl (default), varying the trait impl style

# =============================================================================
# Env trait impl (Counter_TraitEnv)
# =============================================================================

test_that("Counter_TraitEnv works with Env trait impl", {
  counter <- Counter_TraitEnv$new(10L)
  expect_true(inherits(counter, "Counter_TraitEnv"))

  # Inherent method
  expect_equal(counter$get_value(), 10L)

  # Env trait method: Type$Trait$method(x)
  expect_equal(Counter_TraitEnv$MatrixCounter$custom_get(counter), 10L)
  Counter_TraitEnv$MatrixCounter$custom_add(counter, 5L)
  expect_equal(Counter_TraitEnv$MatrixCounter$custom_get(counter), 15L)

  # Static trait method
  expect_equal(Counter_TraitEnv$MatrixCounter$default_value(), 1L)
})

# =============================================================================
# S3 trait impl (Counter_TraitS3)
# =============================================================================

test_that("Counter_TraitS3 works with S3 trait impl", {
  counter <- Counter_TraitS3$new(10L)
  expect_true(inherits(counter, "Counter_TraitS3"))

  # Inherent method
  expect_equal(counter$get_value(), 10L)

  # S3 trait method: generic(x)
  expect_equal(custom_get(counter), 10L)
  custom_add(counter, 5L)
  expect_equal(custom_get(counter), 15L)

  # Static trait method (still uses Type$Trait$method pattern)
  expect_equal(Counter_TraitS3$MatrixCounter$default_value(), 2L)
})

# =============================================================================
# S4 trait impl (Counter_TraitS4)
# =============================================================================

test_that("Counter_TraitS4 works with S4 trait impl", {
  counter <- Counter_TraitS4$new(10L)
  expect_true(inherits(counter, "Counter_TraitS4"))

  # Inherent method
  expect_equal(counter$get_value(), 10L)

  # S4 trait method: s4_trait_Trait_method(x)
  expect_equal(s4_trait_MatrixCounter_custom_get(counter), 10L)
  s4_trait_MatrixCounter_custom_add(counter, 5L)
  expect_equal(s4_trait_MatrixCounter_custom_get(counter), 15L)

  # Static trait method (standalone function)
  expect_equal(Counter_TraitS4_MatrixCounter_default_value(), 3L)
})

# =============================================================================
# S7 trait impl (Counter_TraitS7)
# =============================================================================

test_that("Counter_TraitS7 works with S7 trait impl", {
  counter <- Counter_TraitS7$new(10L)
  expect_true(inherits(counter, "Counter_TraitS7"))

  # Inherent method
  expect_equal(counter$get_value(), 10L)

  # S7 trait method: s7_trait_Trait_method(x)
  expect_equal(s7_trait_MatrixCounter_custom_get(counter), 10L)
  s7_trait_MatrixCounter_custom_add(counter, 5L)
  expect_equal(s7_trait_MatrixCounter_custom_get(counter), 15L)

  # Static trait method
  expect_equal(Counter_TraitS7$MatrixCounter$default_value(), 4L)
})

# =============================================================================
# R6 trait impl (Counter_TraitR6)
# =============================================================================

test_that("Counter_TraitR6 works with R6 trait impl", {
  counter <- Counter_TraitR6$new(10L)
  expect_true(inherits(counter, "Counter_TraitR6"))

  # Inherent method
  expect_equal(counter$get_value(), 10L)

  # R6 trait method: r6_trait_Trait_method(x)
  expect_equal(r6_trait_MatrixCounter_custom_get(counter), 10L)
  r6_trait_MatrixCounter_custom_add(counter, 5L)
  expect_equal(r6_trait_MatrixCounter_custom_get(counter), 15L)

  # Static trait method
  expect_equal(Counter_TraitR6$MatrixCounter$default_value(), 5L)
})

# =============================================================================
# Cross-style compatibility tests
# =============================================================================

test_that("different trait impl styles work independently", {
  # Create counters with different trait styles
  env_counter <- Counter_TraitEnv$new(100L)
  s3_counter <- Counter_TraitS3$new(100L)
  s4_counter <- Counter_TraitS4$new(100L)
  s7_counter <- Counter_TraitS7$new(100L)
  r6_counter <- Counter_TraitR6$new(100L)

  # Modify each using its trait style
  Counter_TraitEnv$MatrixCounter$custom_add(env_counter, 1L)
  custom_add(s3_counter, 2L)
  s4_trait_MatrixCounter_custom_add(s4_counter, 3L)
  s7_trait_MatrixCounter_custom_add(s7_counter, 4L)
  r6_trait_MatrixCounter_custom_add(r6_counter, 5L)

  # Verify each is independent
  expect_equal(Counter_TraitEnv$MatrixCounter$custom_get(env_counter), 101L)
  expect_equal(custom_get(s3_counter), 102L)
  expect_equal(s4_trait_MatrixCounter_custom_get(s4_counter), 103L)
  expect_equal(s7_trait_MatrixCounter_custom_get(s7_counter), 104L)
  expect_equal(r6_trait_MatrixCounter_custom_get(r6_counter), 105L)
})

test_that("static trait methods return different values per type", {
  # Each type has a different default_value
  expect_equal(Counter_TraitEnv$MatrixCounter$default_value(), 1L)
  expect_equal(Counter_TraitS3$MatrixCounter$default_value(), 2L)
  expect_equal(Counter_TraitS4_MatrixCounter_default_value(), 3L)
  expect_equal(Counter_TraitS7$MatrixCounter$default_value(), 4L)
  expect_equal(Counter_TraitR6$MatrixCounter$default_value(), 5L)
})
