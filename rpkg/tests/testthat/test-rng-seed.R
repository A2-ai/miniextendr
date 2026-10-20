# `rng(seed = <param>)` (#2198): the seed contract of `stats::simulate()`,
# written in the generated wrapper. Each test reads and writes `.Random.seed`
# in the global environment only, as the wrapper does.

seed_fit <- structure(list(), class = "mx_seed_fit")
lm_fit <- lm(dist ~ speed, cars)

session_seed <- function() get(".Random.seed", envir = globalenv())
set_session_seed <- function(state) assign(".Random.seed", state, envir = globalenv())

test_that("a seed draws from set.seed(seed) and puts the session's stream back", {
  set.seed(1)
  before <- session_seed()
  sim <- simulate(seed_fit, nsim = 5, seed = 42)
  expect_identical(session_seed(), before)

  set.seed(42)
  expect_equal(as.vector(sim), rnorm(5))
})

test_that("the seed attribute is the one stats::simulate() gives", {
  expect_identical(
    attr(simulate(seed_fit, seed = 42), "seed"),
    attr(simulate(lm_fit, seed = 42), "seed")
  )
  expect_identical(
    attr(simulate(seed_fit, seed = 7L), "seed"),
    attr(simulate(lm_fit, seed = 7L), "seed")
  )
})

test_that("seed = NULL continues the session's stream and records where it was", {
  set.seed(7)
  before <- session_seed()
  sim <- simulate(seed_fit, nsim = 4)
  after <- session_seed()
  expect_identical(attr(sim, "seed"), before)
  expect_identical(attr(sim, "seed"), attr(withr::with_seed(7, simulate(lm_fit)), "seed"))

  set_session_seed(before)
  expect_equal(as.vector(sim), rnorm(4))
  expect_identical(session_seed(), after)
})

test_that("a missing .Random.seed is created; a refused argument creates none", {
  set.seed(11)
  saved <- session_seed()
  withr::defer(set_session_seed(saved))

  rm(".Random.seed", envir = globalenv())
  expect_error(simulate(seed_fit, nsim = -1, seed = 42), "'nsim' must be non-negative")
  expect_false(exists(".Random.seed", envir = globalenv(), inherits = FALSE))

  sim <- simulate(seed_fit, nsim = 2)
  expect_true(exists(".Random.seed", envir = globalenv(), inherits = FALSE))
  expect_type(attr(sim, "seed"), "integer")

  rm(".Random.seed", envir = globalenv())
  sim <- simulate(seed_fit, nsim = 2, seed = 42)
  expect_true(exists(".Random.seed", envir = globalenv(), inherits = FALSE))
  set.seed(42)
  expect_equal(as.vector(sim), rnorm(2))
})

test_that("the session's stream is put back after an error or a panic", {
  set.seed(3)
  before <- session_seed()
  expect_error(rng_seed_draw_then_err(3L, seed = 42), "failed after 3 draws")
  expect_identical(session_seed(), before)
  expect_error(rng_seed_draw_then_panic(3L, seed = 42), "panicked after 3 draws")
  expect_identical(session_seed(), before)

  # Without a seed the draws stay drawn, as in stats.
  expect_error(rng_seed_draw_then_err(3L), "failed after 3 draws")
  after <- session_seed()
  set_session_seed(before)
  runif(3)
  expect_identical(after, session_seed())
})

test_that("a bad seed is set.seed()'s own error", {
  set.seed(5)
  before <- session_seed()
  expect_error(
    suppressWarnings(simulate(seed_fit, seed = "a")),
    "supplied seed is not a valid integer"
  )
  expect_error(
    suppressWarnings(simulate(lm_fit, seed = "a")),
    "supplied seed is not a valid integer"
  )
  expect_identical(session_seed(), before)
})

test_that("a NULL result is left without the attribute", {
  expect_null(rng_seed_draw_or_null(FALSE, seed = 42))
  expect_identical(
    attr(rng_seed_draw_or_null(TRUE, seed = 42), "seed"),
    structure(42, kind = as.list(RNGkind()))
  )
})

test_that("simulate.mx_seed_fit is a consistent S3 method", {
  expect_length(tools::checkS3methods(package = "miniextendr"), 0L)
})
