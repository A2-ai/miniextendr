test_that("ExternalPtr round-trip preserves identity", {
  a <- PtrIdentityTest$new(10L)
  b <- ptr_identity(a)
  # Both should be the exact same R object (same SEXP)
  expect_identical(a, b)
})

test_that("ptr_pick_larger returns the correct ExternalPtr with identity", {
  a <- PtrIdentityTest$new(10L)
  b <- PtrIdentityTest$new(20L)

  # b has larger value, so result should be identical to b

  result <- ptr_pick_larger(a, b)
  expect_identical(result, b)

  # Reversed args: b still has larger value
  result2 <- ptr_pick_larger(b, a)
  expect_identical(result2, b)
})

test_that("ptr_pick_larger with equal values returns first arg", {
  a <- PtrIdentityTest$new(5L)
  b <- PtrIdentityTest$new(5L)

  # When equal, a (first arg) is returned (>= check)
  result <- ptr_pick_larger(a, b)
  expect_identical(result, a)
})

test_that("ExternalPtr identity preserved through multiple round-trips", {
  a <- PtrIdentityTest$new(42L)
  b <- ptr_identity(ptr_identity(ptr_identity(a)))
  expect_identical(a, b)
})
