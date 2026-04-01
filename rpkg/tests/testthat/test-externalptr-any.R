test_that("ExternalPtr into_inner recovers value via Any::downcast", {
  expect_equal(extptr_any_into_inner(42L), 42L)
  expect_equal(extptr_any_into_inner(0L), 0L)
  expect_equal(extptr_any_into_inner(-1L), -1L)
})

test_that("ErasedExternalPtr is<T> checks via Any::is", {
  ptr <- TypeA$new(10L)
  # ptr IS the ExternalPtr SEXP (env-class, not R6 with .ptr slot)
  expect_true(extptr_any_erased_is(ptr))
})

test_that("ErasedExternalPtr downcast_ref returns correct value", {
  ptr <- TypeA$new(99L)
  expect_equal(extptr_any_erased_downcast(ptr), 99L)
})

test_that("ErasedExternalPtr wrong type returns false/None (not crash)", {
  ptr <- TypeA$new(5L)
  # TypeA should NOT be detected as TypeB
  expect_true(extptr_any_wrong_type_is(ptr))
})
