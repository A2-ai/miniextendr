# Fixed-size arrays and boxed unsized markers returned as ALTREP.
# Fixtures: src/rust/lib.rs (array_int_altrep, array_string_altrep,
# boxed_slice_marker_altrep).

is_altrep <- function(x) miniextendr:::unsafe_C_altrep_sexp_is_altrep(x)

test_that("a fixed-size integer array is an ALTREP R can read, write and save", {
  x <- miniextendr:::array_int_altrep()
  expect_true(is_altrep(x))
  expect_identical(x, 1:3)
  expect_identical(sum(x), 6L)
  x[2] <- 20L
  expect_identical(x, c(1L, 20L, 3L))
  expect_identical(unserialize(serialize(x, NULL)), c(1L, 20L, 3L))
})

test_that("a fixed-size string array takes writes, NA included", {
  x <- miniextendr:::array_string_altrep()
  expect_true(is_altrep(x))
  expect_identical(x, c("one", "two"))
  x[2] <- NA
  expect_identical(x, c("one", NA))
  expect_identical(unserialize(serialize(x, NULL)), c("one", NA))

  # A write to a copy leaves the original alone.
  y <- miniextendr:::array_string_altrep()
  z <- y
  z[1] <- "changed"
  expect_identical(y, c("one", "two"))
  expect_identical(z, c("changed", "two"))
})

test_that("a boxed unsized marker returns its slice as ALTREP", {
  x <- miniextendr:::boxed_slice_marker_altrep()
  expect_true(is_altrep(x))
  expect_identical(x, c(0.5, 1.5, 2.5))
  expect_identical(x[2:3], c(1.5, 2.5))
})

test_that("array ALTREPs survive reads, writes and copies under gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  s <- miniextendr:::array_string_altrep()
  s[2] <- "written"
  t <- s
  t[1] <- NA
  i <- miniextendr:::array_int_altrep()
  i[3] <- 30L
  b <- miniextendr:::boxed_slice_marker_altrep()[c(3L, 1L)]
  gctorture(FALSE)
  expect_identical(s, c("one", "written"))
  expect_identical(t, c(NA, "written"))
  expect_identical(i, c(1L, 2L, 30L))
  expect_identical(b, c(2.5, 0.5))
})
