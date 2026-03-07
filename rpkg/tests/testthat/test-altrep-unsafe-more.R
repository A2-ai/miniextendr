test_that("ALTREP constructors produce vectors", {
  expect_equal(altrep_compact_int(5L, 1L, 2L)[[1]], 1L)
  expect_equal(altrep_from_doubles(c(1,2,3))[[2]], 2)
  expect_equal(altrep_from_strings(c("a","b"))[[2]], "b")
  expect_equal(altrep_from_logicals(c(TRUE, FALSE))[[1]], TRUE)
  expect_equal(altrep_from_raw(as.raw(1:3))[[3]], as.raw(3))
  expect_equal(altrep_from_list(list(1L, 2L))[[2]], 2L)
})

test_that("lazy_int_seq_is_materialized reflects state", {
  lazy <- lazy_int_seq(1L, 5L, 1L)
  expect_false(unsafe_C_lazy_int_seq_is_materialized(lazy))
  # force materialization
  tmp <- lazy + 0L
  expect_true(unsafe_C_lazy_int_seq_is_materialized(lazy))
})
