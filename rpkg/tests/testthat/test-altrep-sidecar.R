test_that("ALTREP callbacks can read owner sidecars and materialize independently", {
  source <- c(3L, NA_integer_, 8L)
  x <- miniextendr:::make_sidecar_altrep(source)
  expect_length(x, 3L)
  expect_identical(x[[1L]], 3L)
  expect_true(is.na(x[[2L]]))
  saved <- unserialize(serialize(x, NULL))
  expect_identical(saved, source)
  other <- x
  x[[1L]] <- 99L
  expect_identical(x, c(99L, NA_integer_, 8L))
  expect_identical(other, source)
  expect_identical(source, c(3L, NA_integer_, 8L))
  miniextendr:::gc_stress_altrep_sidecar()
})

test_that("ALTREP owner sidecars survive allocation-triggered collection", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE))
  miniextendr:::gc_stress_altrep_sidecar()
})
