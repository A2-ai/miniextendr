test_that("protect pool roundtrip: insert, get, release, stale", {
  expect_true(protect_pool_roundtrip())
})

test_that("protect pool handles multiple inserts and releases", {
  # Returns 3 if: k1 found, k2 stale (released), k3 found
  expect_equal(protect_pool_multi(), 3L)
})
