# Tests for the R_ext/Applic.h declarations in miniextendr_api::sys (lbfgsb
# with optimfn / optimgr callbacks), called through .Call fixtures on the
# weighted quadratic sum(weight * (x - centre)^2).

quad <- function(centre, weight = rep(1, length(centre)), x0 = rep(0, length(centre)),
                 lower = rep(-Inf, length(centre)), upper = rep(Inf, length(centre)),
                 nbd = rep(0L, length(centre)), maxit = 100L, panic_on_fn_call = 0L) {
  lbfgsb_quadratic(centre, weight, x0, lower, upper, nbd, maxit, panic_on_fn_call)
}

expect_counts <- function(res) {
  # R reports one evaluation count for both, and each request runs both
  # callbacks once.
  expect_identical(res$fncount, res$grcount)
  expect_identical(res$fn_calls, res$fncount)
  expect_identical(res$gr_calls, res$grcount)
}

test_that("lbfgsb finds an interior minimum", {
  centre <- c(1, -2, 3)
  res <- quad(centre, weight = c(1, 2, 0.5))
  expect_identical(res$fail, 0L)
  expect_match(res$message, "^CONVERGENCE")
  expect_equal(res$par, centre, tolerance = 1e-6)
  expect_lt(abs(res$value), 1e-10)
  expect_gt(res$fncount, 0L)
  expect_counts(res)
})

test_that("lbfgsb clamps to active bounds of every kind", {
  # Coordinate 1 boxed in [2, 5] (code 2), 2 capped at -3 (code 3), 3 floored
  # at 0 (code 1, inactive): the minimum is (2, -3, 3) with value 3.
  res <- quad(c(1, -2, 3), weight = c(1, 2, 0.5), x0 = c(4, -4, 1),
              lower = c(2, -Inf, 0), upper = c(5, -3, Inf), nbd = c(2L, 3L, 1L))
  expect_identical(res$fail, 0L)
  expect_identical(res$par[1:2], c(2, -3))
  expect_equal(res$par[3], 3, tolerance = 1e-6)
  expect_equal(res$value, 3, tolerance = 1e-10)
  expect_counts(res)
})

test_that("lbfgsb keeps a parameter fixed by equal bounds", {
  res <- quad(c(1, -2, 3), weight = c(1, 2, 0.5),
              lower = c(-Inf, 0.5, -Inf), upper = c(Inf, 0.5, Inf), nbd = c(0L, 2L, 0L))
  expect_identical(res$fail, 0L)
  expect_identical(res$par[2], 0.5)
  expect_equal(res$par[c(1, 3)], c(1, 3), tolerance = 1e-6)
  expect_equal(res$value, 12.5, tolerance = 1e-10)
  expect_counts(res)
})

test_that("lbfgsb agrees with stats::optim(method = 'L-BFGS-B')", {
  centre <- c(1, -2, 3)
  weight <- c(1, 2, 0.5)
  lower <- c(2, -Inf, 0)
  upper <- c(5, -3, Inf)
  ref <- stats::optim(
    c(4, -4, 1),
    function(x) sum(weight * (x - centre)^2),
    function(x) 2 * weight * (x - centre),
    method = "L-BFGS-B", lower = lower, upper = upper
  )
  res <- quad(centre, weight, x0 = c(4, -4, 1), lower = lower, upper = upper,
              nbd = c(2L, 3L, 1L))
  # Same routine; R sums the objective in long double where the platform has
  # one, so allow for last-bit differences rather than requiring identity.
  expect_equal(res$par, ref$par, tolerance = 1e-8)
  expect_equal(res$value, ref$value, tolerance = 1e-8)
  expect_identical(res$fail, ref$convergence)
  expect_match(ref$message, "^CONVERGENCE")
  expect_match(res$message, "^CONVERGENCE")
})

test_that("lbfgsb reports the iteration limit", {
  res <- quad(c(1, -2), weight = c(1, 1000), x0 = c(10, 10), maxit = 1L)
  expect_identical(res$fail, 1L)
  expect_identical(res$message, "NEW_X")
  expect_counts(res)
})

test_that("lbfgsb reports invalid input without evaluating", {
  res <- quad(c(1, -2), lower = c(0, 0), upper = c(1, 1), nbd = c(0L, 4L))
  expect_identical(res$fail, 52L)
  expect_identical(res$message, "ERROR: INVALID NBD")
  expect_identical(res$fn_calls, 0L)

  res <- quad(1, lower = 2, upper = 1, nbd = 2L)
  expect_identical(res$fail, 52L)
  expect_identical(res$message, "ERROR: NO FEASIBLE SOLUTION")
  expect_identical(res$fn_calls, 0L)
})

test_that("lbfgsb with no parameters evaluates once", {
  res <- quad(numeric())
  expect_identical(res$fail, 0L)
  expect_identical(res$message, "NOTHING TO DO")
  expect_identical(c(res$fncount, res$grcount), c(1L, 0L))
  expect_identical(c(res$fn_calls, res$gr_calls), c(1L, 0L))
  expect_identical(res$value, 0)
})

test_that("a panic in the objective is caught and raised after lbfgsb returns", {
  expect_error(
    quad(c(1, -2, 3), panic_on_fn_call = 3L),
    "objective panicked on call 3"
  )
  # The package stays usable.
  expect_identical(quad(1)$fail, 0L)
})

test_that("lbfgsb's own R errors reach R as errors", {
  expect_error(lbfgsb_report_zero(), "REPORT must be > 0")
  expect_error(lbfgsb_infinite_objective(), "L-BFGS-B needs finite values of 'fn'")
  expect_identical(quad(1)$fail, 0L)
})
