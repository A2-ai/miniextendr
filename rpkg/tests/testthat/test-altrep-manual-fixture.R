# Regression tests for #[altrep(manual)] and #[altrep(no_lowlevel)] fixtures.
#
# altrep_manual_fixture: proves #[altrep(manual)] auto-emits impl_altinteger_from_data!
# altrep_no_lowlevel_fixture: proves #[altrep(no_lowlevel)] requires the manual call.

test_that("#[altrep(manual)] auto-emits impl_altinteger_from_data bridge", {
  x <- make_doubling_altrep(5L)
  expect_equal(length(x), 5L)
  # elt(i) is 0-indexed: x[1] -> elt(0) -> 0*2 = 0, x[3] -> elt(2) -> 2*2 = 4
  expect_equal(x[1L], 0L)
  expect_equal(x[3L], 4L)
  expect_equal(as.integer(x), c(0L, 2L, 4L, 6L, 8L))
})

test_that("#[altrep(no_lowlevel)] requires manual impl_altinteger_from_data", {
  x <- make_no_lowlevel_altrep(3L)
  expect_equal(length(x), 3L)
  # elt(i) is 0-indexed: x[1] -> elt(0) -> 0+100 = 100, x[2] -> elt(1) -> 1+100 = 101
  expect_equal(x[1L], 100L)
  expect_equal(x[2L], 101L)
  expect_equal(as.integer(x), c(100L, 101L, 102L))
})
