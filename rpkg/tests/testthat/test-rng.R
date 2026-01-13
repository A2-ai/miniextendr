# Tests for RNG (Random Number Generation) functionality
# These test the #[miniextendr(rng)] attribute and manual RNG management
#
# Note: All tests use withr::local_seed() to avoid leaking RNG state
# between tests. This ensures .Random.seed is restored after each test.

test_that("rng_uniform generates reproducible uniform random numbers", {
  withr::local_seed(42)
  result1 <- rng_uniform(5L)

  withr::local_seed(42)
  result2 <- rng_uniform(5L)

  expect_equal(result1, result2)
  expect_length(result1, 5)
  expect_true(all(result1 >= 0 & result1 < 1))
})

test_that("rng_normal generates reproducible normal random numbers", {
  withr::local_seed(123)
  result1 <- rng_normal(10L)

  withr::local_seed(123)
  result2 <- rng_normal(10L)

  expect_equal(result1, result2)
  expect_length(result1, 10)
})

test_that("rng_exponential generates reproducible exponential random numbers", {
  withr::local_seed(999)
  result1 <- rng_exponential(5L)

  withr::local_seed(999)
  result2 <- rng_exponential(5L)

  expect_equal(result1, result2)
  expect_length(result1, 5)
  expect_true(all(result1 >= 0)) # exponential is non-negative
})

test_that("rng_int generates reproducible integer random numbers", {
  withr::local_seed(42)
  result1 <- rng_int(10L, 100)

  withr::local_seed(42)
  result2 <- rng_int(10L, 100)

  expect_equal(result1, result2)
  expect_length(result1, 10)
  expect_true(all(result1 >= 0 & result1 < 100))
})

test_that("rng_with_interrupt works (main thread + RNG)", {
  withr::local_seed(42)
  result1 <- rng_with_interrupt(5L)

  withr::local_seed(42)
  result2 <- rng_with_interrupt(5L)

  expect_equal(result1, result2)
  expect_length(result1, 5)
})

test_that("rng_worker_uniform works (explicit worker + RNG)", {
  withr::local_seed(42)
  result1 <- rng_worker_uniform(5L)

  withr::local_seed(42)
  result2 <- rng_worker_uniform(5L)

  expect_equal(result1, result2)
  expect_length(result1, 5)
})

test_that("rng_guard_test (manual RngGuard) works", {
  withr::local_seed(42)
  result1 <- rng_guard_test(5L)

  withr::local_seed(42)
  result2 <- rng_guard_test(5L)

  expect_equal(result1, result2)
  expect_length(result1, 5)
})

test_that("rng_with_rng_test (with_rng helper) works", {
  withr::local_seed(42)
  result1 <- rng_with_rng_test(5L)

  withr::local_seed(42)
  result2 <- rng_with_rng_test(5L)

  expect_equal(result1, result2)
  expect_length(result1, 5)
})

test_that("RngSampler impl method with rng attribute works", {
  sampler <- RngSampler$new(42L)
  expect_equal(sampler$seed_hint(), 42L)

  withr::local_seed(123)
  result1 <- sampler$sample_uniform(5L)

  withr::local_seed(123)
  result2 <- sampler$sample_uniform(5L)

  expect_equal(result1, result2)
  expect_length(result1, 5)
})

test_that("RngSampler sample_normal works", {
  sampler <- RngSampler$new(0L)

  withr::local_seed(456)
  result1 <- sampler$sample_normal(10L)

  withr::local_seed(456)
  result2 <- sampler$sample_normal(10L)

  expect_equal(result1, result2)
  expect_length(result1, 10)
})

test_that("RngSampler static_sample works", {
  withr::local_seed(789)
  result1 <- RngSampler$static_sample(5L)

  withr::local_seed(789)
  result2 <- RngSampler$static_sample(5L)

  expect_equal(result1, result2)
  expect_length(result1, 5)
})

test_that("RNG state is properly saved after each call", {
  # Test that calling RNG functions properly updates .Random.seed
  withr::local_seed(42)
  r1 <- rng_uniform(1L)
  r2 <- rng_uniform(1L)

  withr::local_seed(42)
  r3 <- rng_uniform(1L)
  r4 <- rng_uniform(1L)

  # First calls should match
  expect_equal(r1, r3)
  # Second calls should match (state was properly saved)
  expect_equal(r2, r4)
  # But first and second should differ
  expect_false(isTRUE(all.equal(r1, r2)))
})

test_that("Rust RNG matches R's RNG with same seed", {
  # Test that Rust's unif_rand produces the same values as R's runif
  withr::local_seed(42)
  rust_vals <- rng_uniform(5L)

  withr::local_seed(42)
  r_vals <- runif(5L)

  expect_equal(rust_vals, r_vals)
})
