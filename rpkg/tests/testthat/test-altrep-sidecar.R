# ALTREP owners with `#[r_data] Sidecar<T>` fields.
# Fixtures: src/rust/altrep_sidecar_tests.rs.

is_altrep <- function(x) miniextendr:::unsafe_C_altrep_sexp_is_altrep(x)

test_that("ALTREP callbacks can read owner sidecars and materialize independently", {
  source <- c(3L, NA_integer_, 8L)
  x <- miniextendr:::make_sidecar_altrep(source)
  expect_true(is_altrep(x))
  expect_length(x, 3L)
  expect_identical(x[[1L]], 3L)
  expect_identical(x[[2L]], NA_integer_)
  saved <- unserialize(serialize(x, NULL))
  expect_identical(saved, source)
  other <- x
  x[[1L]] <- 99L
  expect_identical(x, c(99L, NA_integer_, 8L))
  expect_identical(other, source)
  expect_identical(source, c(3L, NA_integer_, 8L))
})

test_that("an ALTREP sidecar keeps its source alive without another reference", {
  x <- miniextendr:::make_sidecar_altrep(c(4L, 5L, 6L))
  gc()
  expect_identical(x[[3L]], 6L)
  expect_identical(sum(x), 15L)
})

test_that("a clone-policy copy shares the sidecar values and not the writes", {
  tag <- list(origin = "fixture")
  x <- miniextendr:::altrep_cloned_sidecar_ints(c(1L, 2L, 3L), tag)
  y <- x
  y[1L] <- 0L
  expect_true(is_altrep(y))
  expect_identical(x, c(1L, 2L, 3L))
  expect_identical(y, c(0L, 2L, 3L))
  expect_identical(miniextendr:::altrep_cloned_sidecar_tag(x), tag)
  expect_identical(miniextendr:::altrep_cloned_sidecar_tag(y), tag)
})

test_that("gc_stress_altrep_sidecar runs", {
  expect_null(miniextendr:::gc_stress_altrep_sidecar())
})

test_that("ALTREP owner sidecars survive allocation-triggered collection", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  miniextendr:::gc_stress_altrep_sidecar()
  x <- miniextendr:::altrep_cloned_sidecar_ints(1:8, list("a", 1:3))
  y <- x
  y[2L] <- 0L
  tag <- miniextendr:::altrep_cloned_sidecar_tag(y)
  gctorture(FALSE)
  expect_identical(tag, list("a", 1:3))
  expect_identical(y, c(1L, 0L, 3:8))
})
