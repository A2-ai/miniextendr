test_that("unsafe_rpkg_constant_int returns constant ALTREP", {
  x <- unsafe_rpkg_constant_int()
  expect_equal(length(x), 10L)
  expect_equal(x[1], 42L)
})

test_that("unsafe_rpkg_constant_real returns constant ALTREP", {
  x <- unsafe_rpkg_constant_real()
  expect_equal(length(x), 10L)
  expect_equal(x[1], pi)
})

test_that("unsafe_rpkg_simple_vec_int mirrors input", {
  vec <- 1:4
  out <- unsafe_rpkg_simple_vec_int(vec)
  expect_identical(out, vec)
})

test_that("unsafe_rpkg_inferred_vec_real mirrors numeric input", {
  vec <- c(1.5, 2.5, 3.5)
  out <- unsafe_rpkg_inferred_vec_real(vec)
  expect_equal(out, vec)
})
