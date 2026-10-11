# Rc<[T]> / Arc<[T]> returned as ALTREP.
# Fixtures: src/rust/altrep_shared_slice_tests.rs.

is_altrep <- function(x) miniextendr:::unsafe_C_altrep_sexp_is_altrep(x)

test_that("a write copies a buffer Rust still shares, and leaves Rust's copy alone", {
  x <- miniextendr:::shared_slice_int()
  expect_true(is_altrep(x))
  expect_identical(x, 1:3)
  x[2] <- 20L
  expect_identical(x, c(1L, 20L, 3L))
  expect_identical(miniextendr:::shared_slice_held(), 1:3)
})

test_that("copies are independent and serialize their current values", {
  x <- miniextendr:::shared_slice_int()
  y <- x
  y[1] <- 10L
  expect_identical(x, 1:3)
  expect_identical(y, c(10L, 2L, 3L))
  restored <- unserialize(serialize(y, NULL))
  expect_true(is_altrep(restored))
  expect_identical(restored, c(10L, 2L, 3L))
})

test_that("a string vector takes writes, NA included, and copies keep their own", {
  x <- miniextendr:::shared_slice_strings()
  expect_true(is_altrep(x))
  expect_identical(x[2], "b")
  y <- x
  y[3] <- NA
  x[1] <- "z"
  expect_identical(x, c("z", "b", "c"))
  expect_identical(y, c("a", "b", NA))
  expect_identical(unserialize(serialize(y, NULL)), c("a", "b", NA))
  # Only read: still saved as the shared-slice class.
  r <- miniextendr:::shared_slice_strings()
  expect_identical(r[3], "c")
  restored <- unserialize(serialize(r, NULL))
  expect_true(is_altrep(restored))
  expect_identical(restored, c("a", "b", "c"))
})

test_that("a logical vector takes NA", {
  x <- miniextendr:::shared_slice_logicals()
  expect_true(is_altrep(x))
  x[2] <- NA
  expect_identical(x, c(TRUE, NA))
  expect_identical(unserialize(serialize(x, NULL)), c(TRUE, NA))
})

test_that("shared slices survive writes and copies under gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  s <- miniextendr:::shared_slice_strings()
  first <- s[1]
  t <- s
  t[2] <- NA
  s[3] <- "written"
  i <- miniextendr:::shared_slice_int()
  j <- i
  j[3] <- 30L
  gctorture(FALSE)
  expect_identical(first, "a")
  expect_identical(s, c("a", "b", "written"))
  expect_identical(t, c("a", NA, "c"))
  expect_identical(i, 1:3)
  expect_identical(j, c(1L, 2L, 30L))
})
