# ALTREP ownership policies: duplication, native buffers, iterator caches.
# Fixtures: src/rust/altrep_ownership_tests.rs.

is_altrep <- function(x) miniextendr:::unsafe_C_altrep_sexp_is_altrep(x)

# region: duplicate = "clone" -------------------------------------------------

test_that("a clone-policy copy takes the write and stays ALTREP", {
  x <- miniextendr:::altrep_cloned_ints(c(10L, 20L, 30L))
  y <- x
  y[1L] <- 0L
  expect_identical(x, c(10L, 20L, 30L))
  expect_identical(y, c(0L, 20L, 30L))
  expect_true(is_altrep(x))
  expect_true(is_altrep(y))
})

test_that("a clone-policy copy keeps its own attributes", {
  x <- miniextendr:::altrep_cloned_ints(c(10L, 20L, 30L))
  names(x) <- c("a", "b", "c")
  y <- x
  names(y)[1L] <- "z"
  class(y) <- "tagged"
  expect_identical(names(x), c("a", "b", "c"))
  expect_identical(class(x), "integer")
  v <- as.vector(x)
  expect_null(names(v))
  expect_identical(names(x), c("a", "b", "c"))
  expect_true(is_altrep(y))
})

test_that("modifying a clone-policy argument leaves the caller's vector alone", {
  x <- miniextendr:::altrep_cloned_ints(c(10L, 20L, 30L))
  f <- function(v) {
    v[2L] <- -5L
    v
  }
  out <- f(x)
  expect_identical(x, c(10L, 20L, 30L))
  expect_identical(out, c(10L, -5L, 30L))
  expect_true(is_altrep(out))
})

test_that("a clone-policy string copy keeps the partial cache and its writes", {
  x <- miniextendr:::altrep_cloned_strings(c("a", "b", NA))
  expect_identical(x[[1L]], "a")
  y <- x
  y[2L] <- "q"
  expect_identical(x, c("a", "b", NA))
  expect_identical(y, c("a", "q", NA))
  expect_true(is_altrep(y))
  z <- y
  z[3L] <- "r"
  expect_identical(y, c("a", "q", NA))
  expect_identical(z, c("a", "q", "r"))
  expect_true(is_altrep(z))
})

test_that("gc_stress_altrep_clone_cache returns the copied written values", {
  x <- miniextendr:::gc_stress_altrep_clone_cache()
  expected <- sprintf("cache-%d", 0:23)
  expected[0:23 %% 5 == 0] <- NA
  expected[1:2] <- c("written", NA)
  expect_identical(x, expected)
  expect_true(is_altrep(x))
})

test_that("gc_stress_altrep_clone_cache keeps cached strings under gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  x <- miniextendr:::gc_stress_altrep_clone_cache()
  y <- x
  y[3] <- "assigned"
  z <- y[c(1:4, 11L)]
  gctorture(FALSE)
  expect_identical(z, c("written", NA, "assigned", "cache-3", NA))
  expect_identical(x[3], "cache-2")
})

# endregion
