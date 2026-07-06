# Regression tests for the shared TraitMethodContext trait-method emitter
# (audit/2026-07-03-dogfooding-macros-codegen.md finding #1).
#
# - BUG1: S4/S7/R6 built the receiver-ptr .Call() argument by first building
#   the call assuming self="x", then running `call.replace(", x", ", .ptr")`.
#   str::replace corrupted any parameter whose R name starts with `x` (e.g.
#   `x_factor` -> `.ptr_factor`), producing a runtime "object '.ptr_factor'
#   not found" error. `scale()` exercises this across all three legs.
# - BUG2: trait methods skipped the stopifnot() precondition-check prelude
#   step (`bump()`) and had no match_arg/choices support at all (`set_mode()`)
#   that inherent methods get.

# =============================================================================
# S4
# =============================================================================

test_that("ScalerS4 scale() does not corrupt the x_factor param (BUG1)", {
  skip_if_not(isClass("ScalerS4"), "S4 class ScalerS4 not registered")
  obj <- ScalerS4(2)
  expect_equal(s4_trait_Scaler_scale(obj, x_factor = 3), 6)
})

test_that("ScalerS4 bump() enforces the stopifnot() precondition (BUG2)", {
  skip_if_not(isClass("ScalerS4"), "S4 class ScalerS4 not registered")
  obj <- ScalerS4(2)
  expect_equal(s4_trait_Scaler_bump(obj, amount = 3L), 5)
  expect_error(s4_trait_Scaler_bump(obj, amount = "not a number"))
})

test_that("ScalerS4 set_mode() validates via match.arg() (BUG2)", {
  skip_if_not(isClass("ScalerS4"), "S4 class ScalerS4 not registered")
  obj <- ScalerS4(2)
  expect_equal(s4_trait_Scaler_set_mode(obj, mode = "slow"), "slow")
  expect_error(s4_trait_Scaler_set_mode(obj, mode = "bogus"))
})

# =============================================================================
# S7
# =============================================================================

test_that("ScalerS7 scale() does not corrupt the x_factor param (BUG1)", {
  obj <- ScalerS7(2)
  expect_equal(s7_trait_Scaler_scale(obj, x_factor = 3), 6)
  # Fast-path shortcut goes through the same self@.ptr receiver extraction.
  expect_equal(ScalerS7_scale(obj, x_factor = 2), 12)
})

test_that("ScalerS7 bump() enforces the stopifnot() precondition (BUG2)", {
  obj <- ScalerS7(2)
  expect_equal(s7_trait_Scaler_bump(obj, amount = 3L), 5)
  expect_error(s7_trait_Scaler_bump(obj, amount = "not a number"))
})

test_that("ScalerS7 set_mode() validates via match.arg() (BUG2)", {
  obj <- ScalerS7(2)
  expect_equal(s7_trait_Scaler_set_mode(obj, mode = "slow"), "slow")
  expect_error(s7_trait_Scaler_set_mode(obj, mode = "bogus"))
})

# =============================================================================
# R6
# =============================================================================

test_that("ScalerR6 scale() does not corrupt the x_factor param (BUG1)", {
  obj <- ScalerR6$new(2)
  expect_equal(r6_trait_Scaler_scale(obj, x_factor = 3), 6)
})

test_that("ScalerR6 bump() enforces the stopifnot() precondition (BUG2)", {
  obj <- ScalerR6$new(2)
  expect_equal(r6_trait_Scaler_bump(obj, amount = 3L), 5)
  expect_error(r6_trait_Scaler_bump(obj, amount = "not a number"))
})

test_that("ScalerR6 set_mode() validates via match.arg() (BUG2)", {
  obj <- ScalerR6$new(2)
  expect_equal(r6_trait_Scaler_set_mode(obj, mode = "slow"), "slow")
  expect_error(r6_trait_Scaler_set_mode(obj, mode = "bogus"))
})
