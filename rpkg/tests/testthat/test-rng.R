# Tests for RNG (Random Number Generation) functionality
# These test the #[miniextendr(rng)] attribute, the `&RngGuard` parameter it
# fills (no R argument), and manual RNG management (RngGuard, with_rng)

test_that("rng_uniform generates reproducible uniform random numbers", {
  set.seed(42)
  result1 <- rng_uniform(5L)

  set.seed(42)
  result2 <- rng_uniform(5L)

  expect_equal(result1, result2)
  expect_length(result1, 5)
  expect_true(all(result1 >= 0 & result1 < 1))
})

test_that("rng_normal generates reproducible normal random numbers", {
  set.seed(123)
  result1 <- rng_normal(10L)

  set.seed(123)
  result2 <- rng_normal(10L)

  expect_equal(result1, result2)
  expect_length(result1, 10)
})

test_that("rng_exponential generates reproducible exponential random numbers", {
  set.seed(999)
  result1 <- rng_exponential(5L)

  set.seed(999)
  result2 <- rng_exponential(5L)

  expect_equal(result1, result2)
  expect_length(result1, 5)
  expect_true(all(result1 >= 0)) # exponential is non-negative
})

test_that("rng_int generates reproducible integer random numbers", {
  set.seed(42)
  result1 <- rng_int(10L, 100)

  set.seed(42)
  result2 <- rng_int(10L, 100)

  expect_equal(result1, result2)
  expect_length(result1, 10)
  expect_true(all(result1 >= 0 & result1 < 100))
})

test_that("rng_with_interrupt works (main thread + RNG)", {
  set.seed(42)
  result1 <- rng_with_interrupt(5L)

  set.seed(42)
  result2 <- rng_with_interrupt(5L)

  expect_equal(result1, result2)
  expect_length(result1, 5)
})

test_that("rng_worker_uniform works (explicit worker + RNG)", {
  # rng_worker_uniform requires the worker-thread feature. Probe the DLL's
  # registration table directly: package_init locks symbols
  # (R_forceSymbols), so the PACKAGE-string form of getNativeSymbolInfo
  # always errors and had silently skipped this test since its addition.
  # C symbols are crate-prefixed since #1273 (C_miniextendr_...).
  skip_if_not(
    !is.null(tryCatch(
      getNativeSymbolInfo("C_miniextendr_rng_worker_uniform",
                          getLoadedDLLs()[["miniextendr"]]),
      error = function(e) NULL
    )),
    "worker-thread feature not enabled"
  )
  set.seed(42)
  result1 <- rng_worker_uniform(5L)

  set.seed(42)
  result2 <- rng_worker_uniform(5L)

  expect_equal(result1, result2)
  expect_length(result1, 5)
})

test_that("rng_guard_test (manual RngGuard) works", {
  set.seed(42)
  result1 <- rng_guard_test(5L)

  set.seed(42)
  result2 <- rng_guard_test(5L)

  expect_equal(result1, result2)
  expect_length(result1, 5)
})

test_that("rng_with_rng_test (with_rng helper) works", {
  set.seed(42)
  result1 <- rng_with_rng_test(5L)

  set.seed(42)
  result2 <- rng_with_rng_test(5L)

  expect_equal(result1, result2)
  expect_length(result1, 5)
})

test_that("RngSampler impl method with rng attribute works", {
  sampler <- RngSampler$new(42L)
  expect_equal(sampler$seed_hint(), 42L)

  set.seed(123)
  result1 <- sampler$sample_uniform(5L)

  set.seed(123)
  result2 <- sampler$sample_uniform(5L)

  expect_equal(result1, result2)
  expect_length(result1, 5)
})

test_that("RngSampler sample_normal works", {
  sampler <- RngSampler$new(0L)

  set.seed(456)
  result1 <- sampler$sample_normal(10L)

  set.seed(456)
  result2 <- sampler$sample_normal(10L)

  expect_equal(result1, result2)
  expect_length(result1, 10)
})

test_that("RngSampler static_sample works", {
  set.seed(789)
  result1 <- RngSampler$static_sample(5L)

  set.seed(789)
  result2 <- RngSampler$static_sample(5L)

  expect_equal(result1, result2)
  expect_length(result1, 5)
})

test_that("RNG state is properly saved after each call", {
  set.seed(42)
  r1 <- rng_uniform(1L)
  r2 <- rng_uniform(1L)

  set.seed(42)
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
  set.seed(42)
  rust_vals <- rng_uniform(5L)

  set.seed(42)
  r_vals <- runif(5L)

  expect_equal(rust_vals, r_vals)
})

# region: guard draws (#2002)

# `c(f(), base())` after one `set.seed()` equals base R's own sequence only if
# `f` loaded the state the seed set and saved the state it advanced.

test_that("guard draws are base R's draws, and the state is saved", {
  set.seed(1)
  x <- c(rng_uniform(5L), runif(5L))
  set.seed(1)
  expect_identical(x, runif(10L))

  set.seed(1)
  x <- c(rng_normal(5L), rnorm(5L))
  set.seed(1)
  expect_identical(x, rnorm(10L))

  set.seed(1)
  x <- c(rng_exponential(5L), rexp(5L))
  set.seed(1)
  expect_identical(x, rexp(10L))

  # `unif_index(n)` is R_unif_index(n), the draw behind sample.int().
  set.seed(1)
  x <- c(rng_int(5L, 10) + 1L, sample.int(10L, 5L, replace = TRUE))
  set.seed(1)
  expect_identical(x, sample.int(10L, 10L, replace = TRUE))
})

test_that("a guard draw advances .Random.seed", {
  set.seed(42)
  before <- get(".Random.seed", envir = globalenv())
  rng_uniform(1L)
  expect_false(identical(before, get(".Random.seed", envir = globalenv())))
})

test_that("the guard parameter is no R formal", {
  expect_identical(names(formals(rng_uniform)), "n")
  expect_identical(names(formals(rng_normal)), "n")
  expect_identical(names(formals(rng_int)), c("n", "max"))
  expect_length(formals(rng_bool), 0L)
})

test_that("a guard parameter keeps a worker function on R's main thread", {
  expect_true(rng_guard_worker_on_main())
})

test_that("with_rng loads the state when called and saves it", {
  set.seed(42)
  x <- c(rng_with_rng_test(5L), runif(5L))
  set.seed(42)
  expect_identical(x, runif(10L))
})

test_that("with_rng draws each kind as base R does", {
  set.seed(7)
  x <- rng_with_rng_each()
  set.seed(7)
  expect_identical(x, c(runif(1L), rnorm(1L), rexp(1L), sample.int(10L, 1L) - 1))
})

test_that("a guard from RngGuard::new draws and saves", {
  set.seed(42)
  x <- c(rng_guard_test(5L), runif(5L))
  set.seed(42)
  expect_identical(x, runif(10L))
})

test_that("env methods draw from the guard, instance and static", {
  s <- RngSampler$new(1L)
  set.seed(3)
  x <- c(s$sample_uniform(4L), runif(2L))
  set.seed(3)
  expect_identical(x, runif(6L))
  set.seed(3)
  x <- s$sample_normal(4L)
  set.seed(3)
  expect_identical(x, rnorm(4L))
  set.seed(3)
  x <- RngSampler$static_sample(4L)
  set.seed(3)
  expect_identical(x, runif(4L))
  expect_identical(names(formals(s$sample_uniform)), "n")
  expect_identical(names(formals(RngSampler$static_sample)), "n")
})

test_that("an R6 method draws from the guard", {
  obj <- miniextendr:::RngSamplerR6$new(2)
  set.seed(3)
  x <- obj$unif(4L)
  set.seed(3)
  expect_identical(x, 2 * runif(4L))
  expect_identical(names(formals(obj$unif)), "n")
})

test_that("an S3 method draws from the guard", {
  obj <- new_rngsamplers3(2)
  set.seed(3)
  x <- rng_s3_unif(obj, 4L)
  set.seed(3)
  expect_identical(x, 2 * runif(4L))
})

test_that("an S4 method draws from the guard", {
  obj <- miniextendr:::RngSamplerS4(2)
  set.seed(3)
  x <- miniextendr:::s4_rng_unif(obj, 4L)
  set.seed(3)
  expect_identical(x, 2 * runif(4L))
})

test_that("an S7 method draws from the guard, through dispatch and the shortcut", {
  obj <- miniextendr:::RngSamplerS7(2)
  set.seed(3)
  x <- miniextendr:::rng_s7_unif(obj, 4L)
  set.seed(3)
  expect_identical(x, 2 * runif(4L))
  set.seed(3)
  x <- miniextendr:::RngSamplerS7_rng_s7_unif(obj, 4L)
  set.seed(3)
  expect_identical(x, 2 * runif(4L))
})

test_that("a vctrs constructor draws from the guard", {
  set.seed(3)
  v <- miniextendr:::new_rngnoise(4L)
  set.seed(3)
  expect_identical(vctrs::vec_data(v), rnorm(4L))
})

# endregion

# region: Upstream example-derived fixtures

test_that("rng_bool returns logical", {
  set.seed(123)
  result <- rng_bool()
  expect_type(result, "logical")
})

test_that("rng_range returns value in range", {
  set.seed(456)
  result <- rng_range(10.0, 20.0)
  expect_true(result >= 10.0 && result < 20.0)
})

test_that("rng_chi_sq_approx returns positive values", {
  set.seed(789)
  result <- rng_chi_sq_approx(5L, 3L)
  expect_equal(length(result), 5L)
  # Chi-squared values are always non-negative
  expect_true(all(result >= 0))
})

# endregion
