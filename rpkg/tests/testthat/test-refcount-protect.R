test_that("RefCountedArena protect/is_protected/unprotect/refcount cycle", {
  expect_true(refcount_arena_roundtrip())
})
