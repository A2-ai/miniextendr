# Tests for the Rmath.h declarations in miniextendr_api::sys (Rf_dnorm4,
# Rf_pnorm5, Rf_qnorm5, Rf_qchisq), called through .Call fixtures.
#
# stats:: calls the same libR routines, so finite and infinite results must be
# identical. For NA/NaN inputs only the class is compared: stats:: normalises
# the result to NA_real_ / NaN, the C routine returns whatever NaN the
# arithmetic propagates.

x_in <- c(-Inf, -1e10, -40, -38.5, -10, -2.5, -1, -0.3, 0, 0.3, 1, 1.5, 2.5,
          8.25, 10, 38.5, 40, 1e10, Inf, NA, NaN)
p_in <- c(0, 1e-300, 1e-100, 1e-10, 0.001, 0.025, 0.3, 0.5, 0.7, 0.975, 0.999,
          1 - 1e-10, 1 - 2^-53, 1, -0.1, 1.1, NA, NaN)
lp_in <- c(-Inf, -1e5, -800, -50, -5, log(0.5), -0.01, -1e-10, -1e-300, 0, 0.5,
           NA, NaN)
normal_params <- list(c(0, 1), c(1.5, 2.5), c(-3, 0.25), c(1, 0), c(0, Inf))

expect_same_as_stats <- function(got, want) {
  na <- is.na(want)
  expect_identical(is.na(got), na)
  expect_identical(got[!na], want[!na])
}

test_that("rmath_dnorm matches stats::dnorm", {
  for (par in normal_params) {
    for (lg in c(FALSE, TRUE)) {
      want <- suppressWarnings(stats::dnorm(x_in, par[1], par[2], log = lg))
      expect_same_as_stats(rmath_dnorm(x_in, par[1], par[2], lg), want)
    }
  }
})

test_that("rmath_pnorm matches stats::pnorm in both tails and scales", {
  for (par in normal_params) {
    for (lower in c(TRUE, FALSE)) {
      for (lg in c(FALSE, TRUE)) {
        want <- suppressWarnings(
          stats::pnorm(x_in, par[1], par[2], lower.tail = lower, log.p = lg)
        )
        expect_same_as_stats(rmath_pnorm(x_in, par[1], par[2], lower, lg), want)
      }
    }
  }
})

test_that("rmath_qnorm matches stats::qnorm in both tails and scales", {
  for (par in normal_params) {
    for (lower in c(TRUE, FALSE)) {
      for (lg in c(FALSE, TRUE)) {
        p <- if (lg) lp_in else p_in
        want <- suppressWarnings(
          stats::qnorm(p, par[1], par[2], lower.tail = lower, log.p = lg)
        )
        expect_same_as_stats(rmath_qnorm(p, par[1], par[2], lower, lg), want)
      }
    }
  }
})

test_that("rmath_qchisq matches stats::qchisq in both tails and scales", {
  for (df in c(0, 0.5, 1, 2, 7.5, 100)) {
    for (lower in c(TRUE, FALSE)) {
      for (lg in c(FALSE, TRUE)) {
        p <- if (lg) lp_in else p_in
        want <- suppressWarnings(
          stats::qchisq(p, df, lower.tail = lower, log.p = lg)
        )
        expect_same_as_stats(rmath_qchisq(p, df, lower, lg), want)
      }
    }
  }
})

test_that("invalid domains give NaN; only the stats:: wrappers warn", {
  # nmath does not report domain errors; "NaNs produced" comes from the
  # vectorised stats:: wrapper, not from the C routine.
  expect_warning(stats::dnorm(1, 0, -1), "NaNs produced")
  expect_no_warning(res <- rmath_dnorm(1, 0, -1, FALSE))
  expect_true(is.nan(res))

  expect_warning(stats::pnorm(1, 0, -1), "NaNs produced")
  expect_no_warning(res <- rmath_pnorm(1, 0, -1, TRUE, FALSE))
  expect_true(is.nan(res))

  expect_no_warning(res <- rmath_qnorm(c(-0.1, 1.1, 0.5), 0, 1, TRUE, FALSE))
  expect_true(all(is.nan(res[1:2])))
  expect_identical(res[3], 0)

  expect_warning(stats::qchisq(0.5, -1), "NaNs produced")
  expect_no_warning(res <- rmath_qchisq(c(0.5, 1.5), c(-1), TRUE, FALSE))
  expect_true(all(is.nan(res)))
})

test_that("endpoint probabilities map to the ends of the support", {
  expect_identical(rmath_qnorm(c(0, 1), 0, 1, TRUE, FALSE), c(-Inf, Inf))
  expect_identical(rmath_qnorm(c(0, 1), 0, 1, FALSE, FALSE), c(Inf, -Inf))
  expect_identical(rmath_qnorm(c(-Inf, 0), 0, 1, TRUE, TRUE), c(-Inf, Inf))
  expect_identical(rmath_qchisq(c(0, 1), 3, TRUE, FALSE), c(0, Inf))
  expect_identical(rmath_pnorm(c(-Inf, Inf), 0, 1, TRUE, FALSE), c(0, 1))
  expect_identical(rmath_pnorm(c(-Inf, Inf), 0, 1, TRUE, TRUE), c(-Inf, 0))
})

test_that("identities hold without consulting stats::", {
  # Phi(0) = 1/2 in either tail.
  expect_identical(rmath_pnorm(0, 0, 1, TRUE, FALSE), 0.5)
  expect_identical(rmath_pnorm(0, 0, 1, FALSE, FALSE), 0.5)

  # Reflection pnorm(-x) = 1 - pnorm(x), through the upper-tail flag.
  x <- c(0, 0.1, 0.5, 0.67, 0.7, 1, 2.5, 5, 10, 20, 37, 38.5, 40)
  for (lg in c(FALSE, TRUE)) {
    expect_identical(
      rmath_pnorm(-x, 0, 1, TRUE, lg),
      rmath_pnorm(x, 0, 1, FALSE, lg)
    )
  }

  # df = 2: the upper-tail quantile of log probability lp is -2 * lp.
  lp <- c(-1e-8, -0.01, -0.5, -1, -5, -50, -700, -1e4)
  expect_equal(rmath_qchisq(lp, 2, FALSE, TRUE), -2 * lp, tolerance = 1e-14)
})

test_that("extreme tails stay finite on the log scale", {
  expect_identical(rmath_pnorm(-40, 0, 1, TRUE, FALSE), 0)
  lp <- rmath_pnorm(-40, 0, 1, TRUE, TRUE)
  expect_true(is.finite(lp) && lp < -800)
  q <- rmath_qnorm(-1e5, 0, 1, TRUE, TRUE)
  expect_true(is.finite(q) && q < -400)
  expect_equal(rmath_qnorm(lp, 0, 1, TRUE, TRUE), -40, tolerance = 1e-12)
})
