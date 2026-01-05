# Tests for vctrs C API integration
#
# These tests verify that the vctrs support correctly interfaces with vctrs'
# maturing C API (obj_is_vector, short_vec_size, short_vec_recycle).
#
# Note: Test functions are internal (not exported), so we use `:::` to access them.

# Helper to skip if vctrs feature is not enabled
skip_if_vctrs_disabled <- function() {
  skip_if_not("vctrs" %in% miniextendr::rpkg_enabled_features(), "vctrs feature not enabled")
  skip_if_not_installed("vctrs")
}

test_that("vctrs support is initialized", {
  skip_if_vctrs_disabled()

  # vctrs should be initialized when the package loads
  expect_true(miniextendr:::test_vctrs_is_initialized())
})

test_that("obj_is_vector returns TRUE for vectors", {
  skip_if_vctrs_disabled()

  # Atomic vectors
  expect_true(miniextendr:::test_vctrs_obj_is_vector(logical(0)))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(TRUE))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(integer(0)))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(1L))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(double(0)))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(1.5))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(character(0)))
  expect_true(miniextendr:::test_vctrs_obj_is_vector("hello"))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(raw(0)))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(as.raw(1)))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(complex(0)))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(1+2i))

  # Lists
  expect_true(miniextendr:::test_vctrs_obj_is_vector(list()))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(list(1, 2, 3)))

  # Data frames
  expect_true(miniextendr:::test_vctrs_obj_is_vector(data.frame()))
  expect_true(miniextendr:::test_vctrs_obj_is_vector(data.frame(x = 1:3, y = letters[1:3])))
})

test_that("obj_is_vector returns FALSE for non-vectors", {
  skip_if_vctrs_disabled()

  # NULL
  expect_false(miniextendr:::test_vctrs_obj_is_vector(NULL))

  # Language objects
  expect_false(miniextendr:::test_vctrs_obj_is_vector(quote(x + y)))
  expect_false(miniextendr:::test_vctrs_obj_is_vector(call("sum")))

  # Environments
  expect_false(miniextendr:::test_vctrs_obj_is_vector(new.env()))
  expect_false(miniextendr:::test_vctrs_obj_is_vector(globalenv()))

  # Symbols
  expect_false(miniextendr:::test_vctrs_obj_is_vector(as.symbol("x")))

  # Functions
  expect_false(miniextendr:::test_vctrs_obj_is_vector(function() {}))
  expect_false(miniextendr:::test_vctrs_obj_is_vector(mean))
})

test_that("short_vec_size returns correct sizes", {
  skip_if_vctrs_disabled()

  # Atomic vectors
  expect_equal(miniextendr:::test_vctrs_short_vec_size(integer(0)), 0L)
  expect_equal(miniextendr:::test_vctrs_short_vec_size(1:10), 10L)
  expect_equal(miniextendr:::test_vctrs_short_vec_size(c(1.5, 2.5, 3.5)), 3L)
  expect_equal(miniextendr:::test_vctrs_short_vec_size(c("a", "b")), 2L)

  # Lists
  expect_equal(miniextendr:::test_vctrs_short_vec_size(list()), 0L)
  expect_equal(miniextendr:::test_vctrs_short_vec_size(list(1, 2, 3, 4)), 4L)

  # Data frames - size is number of rows
  expect_equal(miniextendr:::test_vctrs_short_vec_size(data.frame()), 0L)
  expect_equal(miniextendr:::test_vctrs_short_vec_size(data.frame(x = 1:5)), 5L)
  expect_equal(miniextendr:::test_vctrs_short_vec_size(mtcars), 32L)
})

test_that("short_vec_recycle recycles vectors", {
  skip_if_vctrs_disabled()

  # Recycle length-1 to length-5
  result <- miniextendr:::test_vctrs_short_vec_recycle(42L, 5L)
  expect_equal(result, rep(42L, 5))

  # Recycle length-1 character
  result <- miniextendr:::test_vctrs_short_vec_recycle("x", 3L)
  expect_equal(result, c("x", "x", "x"))

  # No recycling needed when sizes match
  result <- miniextendr:::test_vctrs_short_vec_recycle(1:3, 3L)
  expect_equal(result, 1:3)

  # Empty vector stays empty
  result <- miniextendr:::test_vctrs_short_vec_recycle(integer(0), 0L)
  expect_equal(result, integer(0))
})

test_that("short_vec_recycle matches vctrs behavior", {
  skip_if_vctrs_disabled()

  # Test that our wrapper matches vctrs::vec_recycle
  x <- 1:1
  size <- 5L
  our_result <- miniextendr:::test_vctrs_short_vec_recycle(x, size)
  vctrs_result <- vctrs::vec_recycle(x, size)
  expect_equal(our_result, vctrs_result)

  x <- list(a = 1)
  size <- 3L
  our_result <- miniextendr:::test_vctrs_short_vec_recycle(x, size)
  vctrs_result <- vctrs::vec_recycle(x, size)
  expect_equal(our_result, vctrs_result)
})

test_that("vctrs error messages are informative", {
  # Test error message formatting (doesn't require vctrs)
  expect_match(
    miniextendr:::test_vctrs_error_message(0L),
    "not initialized"
  )
  expect_match(
    miniextendr:::test_vctrs_error_message(1L),
    "not found"
  )
  expect_match(
    miniextendr:::test_vctrs_error_message(2L),
    "already initialized"
  )
  expect_match(
    miniextendr:::test_vctrs_error_message(3L),
    "main thread"
  )
})
