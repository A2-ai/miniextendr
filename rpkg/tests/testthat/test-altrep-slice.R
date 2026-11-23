# ALTREP vectors read from a struct's slice field (#2363).
# Fixtures: src/rust/altrep_slice_tests.rs. Each fixture returns the ALTREP
# and the eager conversion of the same elements, so the two must be identical.

is_altrep <- function(x) miniextendr:::unsafe_C_altrep_sexp_is_altrep(x)

expect_altrep_pairs <- function(pairs) {
  for (i in seq(1L, length(pairs), by = 2L)) {
    expect_true(is_altrep(pairs[[i]]), info = paste("element", i))
    expect_identical(pairs[[i]], pairs[[i + 1L]], info = paste("element", i))
  }
}

test_that("a bare derive over an RFactor array is a factor, for two array lengths", {
  for (pair in list(miniextendr:::altrep_slice_dice3(), miniextendr:::altrep_slice_dice5())) {
    dice <- pair[[1]]
    expect_true(is_altrep(dice))
    expect_true(is.factor(dice))
    expect_identical(levels(dice), c("One", "Two", "Three", "Four", "Five", "Six"))
    expect_identical(dice, pair[[2]])
  }
  dice <- miniextendr:::altrep_slice_dice5()
  expect_identical(dice[[1]][2:3], dice[[2]][2:3])
  expect_identical(as.integer(dice[[1]]), c(5L, 5L, 3L, 4L, 1L))
})

test_that("a tuple struct over an RFactor array is a factor", {
  expect_altrep_pairs(miniextendr:::altrep_slice_outcomes3())
  expect_altrep_pairs(miniextendr:::altrep_slice_outcomes5())
  expect_identical(levels(miniextendr:::altrep_slice_outcomes3()[[1]]), c("Loss", "Win"))
})

test_that("family derives convert each element with Coerce", {
  expect_altrep_pairs(miniextendr:::altrep_slice_dice_values3())
  expect_altrep_pairs(miniextendr:::altrep_slice_dice_values5())
  expect_identical(miniextendr:::altrep_slice_dice_values3()[[1]], c(2L, 6L, 1L))
  expect_altrep_pairs(miniextendr:::altrep_slice_wins3())
  expect_altrep_pairs(miniextendr:::altrep_slice_wins5())
  expect_identical(miniextendr:::altrep_slice_wins3()[[1]], c(TRUE, FALSE, TRUE))
})

test_that("each instantiation of a generic struct has its own class", {
  dice3 <- miniextendr:::altrep_slice_dice3()[[1]]
  dice5 <- miniextendr:::altrep_slice_dice5()[[1]]
  expect_identical(
    miniextendr:::altrep_slice_dice_classes(dice3, dice5),
    c(TRUE, TRUE, FALSE, FALSE)
  )
})

test_that("arrays of native, logical and string elements match their eager values", {
  expect_altrep_pairs(miniextendr:::altrep_slice_arrays())
})

test_that("Vec, Box<[T]>, choice-name and native-newtype elements match their eager values", {
  owned <- miniextendr:::altrep_slice_owned()
  expect_altrep_pairs(owned)
  expect_identical(owned[[7]], c("Fast", "Slow", "Fast"))
})

test_that("R reads a native Vec in place", {
  expect_identical(miniextendr:::altrep_slice_reads_in_place(), c(TRUE, TRUE))
})

test_that("a write to an owned native slice goes through", {
  x <- miniextendr:::altrep_slice_owned()[[1]]
  x[1] <- 40L
  expect_identical(x, c(40L, 5L))
})

test_that("a 'static slice is read in place and copied for a write", {
  primes <- miniextendr:::altrep_slice_primes()
  expect_true(is_altrep(primes))
  expect_identical(primes, c(2L, 3L, 5L, 7L, 11L))
  primes[2] <- 30L
  expect_identical(primes, c(2L, 30L, 5L, 7L, 11L))
  expect_identical(miniextendr:::altrep_slice_primes_source(), c(2L, 3L, 5L, 7L, 11L))
  expect_identical(miniextendr:::altrep_slice_primes(), c(2L, 3L, 5L, 7L, 11L))
})

test_that("field picks one of several slice fields; family derives read borrowed strings", {
  selected <- miniextendr:::altrep_slice_selected()
  for (x in selected) expect_true(is_altrep(x))
  expect_identical(selected[[1]], c("a", "b"))
  expect_identical(selected[[2]], c(0.25, 0.75))
  expect_identical(selected[[3]], c("red", "green"))
})

test_that("a generic slice ALTREP saves as its plain vector", {
  pair <- miniextendr:::altrep_slice_dice3()
  restored <- unserialize(serialize(pair[[1]], NULL))
  expect_false(is_altrep(restored))
  expect_identical(restored, pair[[2]])
})

test_that("a concrete slice owner with serialize reads back as ALTREP", {
  # Save before anything takes a data pointer: once the faces are copied out
  # (identical() does), R saves the plain vector (#1960).
  saved <- miniextendr:::altrep_slice_saved_dice()
  restored <- unserialize(serialize(saved, NULL))
  expect_true(is_altrep(saved))
  expect_identical(saved, c(1L, 6L, 2L, 2L))
  expect_true(is_altrep(restored))
  expect_identical(restored, c(1L, 6L, 2L, 2L))
})

test_that("slice ALTREP factors keep their levels and class under gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  miniextendr:::gc_stress_altrep_slice_factor()
  dice <- miniextendr:::altrep_slice_dice5()
  outcomes <- miniextendr:::altrep_slice_outcomes3()
  primes <- miniextendr:::altrep_slice_primes()
  primes[1] <- 1L
  gctorture(FALSE)
  expect_identical(dice[[1]], dice[[2]])
  expect_identical(outcomes[[1]], outcomes[[2]])
  expect_identical(primes, c(1L, 3L, 5L, 7L, 11L))
})
