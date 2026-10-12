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

# region: AltrepBuffer --------------------------------------------------------

test_that("R writes into a native buffer reach reads, copies and serialization", {
  x <- miniextendr:::altrep_buffer_ints(c(1L, 2L, 3L))
  x[2L] <- 99L
  expect_true(is_altrep(x))
  expect_identical(x, c(1L, 99L, 3L))
  expect_identical(sum(x), 103L)
  y <- x
  y[1L] <- 0L
  expect_identical(x, c(1L, 99L, 3L))
  expect_identical(y, c(0L, 99L, 3L))
  expect_true(is_altrep(y))
  restored <- unserialize(serialize(x, NULL))
  expect_true(is_altrep(restored))
  expect_identical(restored, c(1L, 99L, 3L))
})

test_that("a buffer edit after R took its pointer keeps R's writes", {
  x <- miniextendr:::altrep_buffer_ints(c(1L, 2L, 3L))
  invisible(x + 0L)
  x[3L] <- 30L
  miniextendr:::altrep_buffer_extend(x, c(4L, 5L))
  expect_true(is_altrep(x))
  expect_identical(x, c(1L, 2L, 30L, 4L, 5L))
  expect_identical(x + 0L, c(1L, 2L, 30L, 4L, 5L))
  y <- x
  y[5L] <- 50L
  miniextendr:::altrep_buffer_extend(x, 6L)
  expect_identical(x, c(1L, 2L, 30L, 4L, 5L, 6L))
  expect_identical(y, c(1L, 2L, 30L, 4L, 50L))
})

test_that("real and logical buffers read, copy and serialize their values", {
  r <- miniextendr:::altrep_buffer_reals(c(1.5, NA, 3))
  r[1L] <- 2.5
  expect_identical(r, c(2.5, NA, 3))
  expect_true(is_altrep(unserialize(serialize(r, NULL))))
  l <- miniextendr:::altrep_buffer_logicals(c(TRUE, NA, FALSE))
  expect_identical(l, c(TRUE, NA, FALSE))
  expect_identical(l[[2L]], NA)
  m <- l
  m[2L] <- TRUE
  expect_identical(l, c(TRUE, NA, FALSE))
  expect_identical(m, c(TRUE, TRUE, FALSE))
  expect_true(is_altrep(m))
  expect_identical(unserialize(serialize(m, NULL)), c(TRUE, TRUE, FALSE))
})

# endregion
