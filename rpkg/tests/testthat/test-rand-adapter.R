# rand / rand_distr adapter fixtures
# (RRng via rand traits, RDistributions, RRngOps, RDistributionOps)

# =============================================================================
# RRng through the rand trait surface
# =============================================================================

test_that("rand_rrng_uniform draws from R's RNG in [0, 1)", {
  skip_if_missing_feature("rand")
  x <- rand_rrng_uniform(100L)
  expect_length(x, 100L)
  expect_true(all(x >= 0 & x < 1))
})

test_that("rand_rrng_uniform is reproducible under set.seed (uses R's RNG, not an OS source)", {
  skip_if_missing_feature("rand")
  set.seed(42)
  a <- rand_rrng_uniform(10L)
  set.seed(42)
  b <- rand_rrng_uniform(10L)
  expect_identical(a, b)
  set.seed(43)
  expect_false(identical(a, rand_rrng_uniform(10L)))
})

test_that("rand_rrng_range respects bounds and R's RNG", {
  skip_if_missing_feature("rand")
  set.seed(1)
  x <- rand_rrng_range(50L, -2, 3)
  expect_length(x, 50L)
  expect_true(all(x >= -2 & x < 3))
  set.seed(1)
  expect_identical(rand_rrng_range(50L, -2, 3), x)
})

test_that("rand_rrng_range rejects an empty range with an R error", {
  skip_if_missing_feature("rand")
  expect_error(rand_rrng_range(1L, 5, 5))
})

# =============================================================================
# RDistributions (R's native distribution functions through the trait)
# =============================================================================

test_that("rand_rdistributions_sample exercises all four trait methods", {
  skip_if_missing_feature("rand")
  set.seed(7)
  v <- rand_rdistributions_sample()
  expect_length(v, 4L)
  expect_gt(v[[2]], 0) # standard_exp
  expect_true(v[[3]] >= 0 && v[[3]] < 100 && v[[3]] == trunc(v[[3]])) # uniform_index
  expect_true(v[[4]] >= 0 && v[[4]] < 1) # uniform_f64
  set.seed(7)
  expect_identical(rand_rdistributions_sample(), v)
})

# =============================================================================
# RRngOps (pure-Rust StdRng exposed to R; independent of set.seed)
# =============================================================================

test_that("SeededRng exposes RRngOps deterministically per seed", {
  skip_if_missing_feature("rand")
  a <- SeededRng$new(42L)
  b <- SeededRng$new(42L)
  expect_identical(a$RRngOps$random_f64(), b$RRngOps$random_f64())
  expect_identical(a$RRngOps$random_i32(), b$RRngOps$random_i32())
})

test_that("RRngOps methods return the documented types and ranges", {
  skip_if_missing_feature("rand")
  r <- SeededRng$new(7L)
  x <- r$RRngOps$random_f64()
  expect_true(x >= 0 && x < 1)
  expect_type(r$RRngOps$random_i32(), "integer")
  expect_type(r$RRngOps$random_bool(), "logical")
  y <- r$RRngOps$gen_range_f64(0, 10)
  expect_true(y >= 0 && y < 10)
  expect_true(r$RRngOps$gen_range_i32(5L, 15L) %in% 5:14)
  expect_true(r$RRngOps$gen_bool(1))
  expect_false(r$RRngOps$gen_bool(0))
  v <- r$RRngOps$random_f64_vec(20L)
  expect_length(v, 20L)
  expect_true(all(v >= 0 & v < 1))
})

test_that("RRngOps invalid inputs surface as R errors, not crashes", {
  skip_if_missing_feature("rand")
  r <- SeededRng$new(1L)
  expect_error(r$RRngOps$gen_range_f64(10, 5)) # empty range
  expect_error(r$RRngOps$gen_bool(1.5)) # p outside [0, 1]
})

# =============================================================================
# rand_distr through RRng
# =============================================================================

test_that("rand_distr_normal samples via R's RNG with sane moments", {
  skip_if_missing_feature("rand_distr")
  set.seed(99)
  x <- rand_distr_normal(2000L, 5, 2)
  expect_length(x, 2000L)
  # Fixed seed → deterministic values; tolerances are deliberately loose.
  expect_equal(mean(x), 5, tolerance = 0.2)
  expect_equal(sd(x), 2, tolerance = 0.2)
  set.seed(99)
  expect_identical(rand_distr_normal(2000L, 5, 2), x)
})

test_that("rand_distr_normal rejects invalid parameters with an R error", {
  skip_if_missing_feature("rand_distr")
  # rand_distr's Normal::new only rejects non-finite sd (negative sd is
  # allowed — it mirrors the distribution).
  expect_error(rand_distr_normal(10L, 0, Inf), "invalid normal parameters")
})

# =============================================================================
# RDistributionOps (rand_distr Normal exposed to R)
# =============================================================================

test_that("SeededNormal exposes RDistributionOps sampling and statistics", {
  skip_if_missing_feature("rand_distr")
  d <- SeededNormal$new(0, 1, 7L)
  expect_identical(d$RDistributionOps$mean(), 0)
  expect_identical(d$RDistributionOps$std_dev(), 1)
  x <- d$RDistributionOps$sample_n(1000L)
  expect_length(x, 1000L)
  expect_equal(mean(x), 0, tolerance = 0.15)
  expect_equal(sd(x), 1, tolerance = 0.15)
})

test_that("SeededNormal sampling is deterministic per seed", {
  skip_if_missing_feature("rand_distr")
  a <- SeededNormal$new(3, 0.5, 11L)
  b <- SeededNormal$new(3, 0.5, 11L)
  expect_identical(a$RDistributionOps$sample(), b$RDistributionOps$sample())
  expect_identical(a$RDistributionOps$sample_n(5L), b$RDistributionOps$sample_n(5L))
})
