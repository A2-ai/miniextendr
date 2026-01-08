test_that("constant_int returns constant ALTREP", {
  x <- constant_int()
  expect_equal(length(x), 10L)
  expect_equal(x[1], 42L)
})

test_that("constant_real returns constant ALTREP", {
  x <- constant_real()
  expect_equal(length(x), 10L)
  expect_equal(x[1], pi)
})

test_that("altrep_from_integers mirrors input", {
  vec <- 1:4
  out <- altrep_from_integers(vec)
  expect_identical(out, vec)
})

test_that("altrep_from_doubles mirrors numeric input", {
  vec <- c(1.5, 2.5, 3.5)
  out <- altrep_from_doubles(vec)
  expect_equal(out, vec)
})
