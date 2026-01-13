# Tests for GC protection: ListBuilder, StrVecBuilder, and ReprotectSlot

# =============================================================================
# ListBuilder tests
# =============================================================================

test_that("test_list_builder_length creates list with correct length", {
  expect_equal(test_list_builder_length(0L), 0L)
  expect_equal(test_list_builder_length(1L), 1L)
  expect_equal(test_list_builder_length(5L), 5L)
  expect_equal(test_list_builder_length(100L), 100L)
})

test_that("test_list_builder_set creates list with typed elements", {
  result <- test_list_builder_set()

  expect_true(is.list(result))
  expect_equal(length(result), 3L)

  # First element should be integer vector of length 1
  expect_true(is.integer(result[[1]]))
  expect_equal(length(result[[1]]), 1L)

  # Second element should be numeric vector of length 2
  expect_true(is.double(result[[2]]))
  expect_equal(length(result[[2]]), 2L)

  # Third element should be character vector of length 3
  expect_true(is.character(result[[3]]))
  expect_equal(length(result[[3]]), 3L)
})

test_that("test_list_set_elt creates list with unprotected children", {
  result <- test_list_set_elt()

  expect_true(is.list(result))
  expect_equal(length(result), 2L)

  # First element should be integer vector of length 5
  expect_true(is.integer(result[[1]]))
  expect_equal(length(result[[1]]), 5L)

  # Second element should be numeric vector of length 10
  expect_true(is.double(result[[2]]))
  expect_equal(length(result[[2]]), 10L)
})

test_that("test_list_set_elt_with creates list with closure-allocated children", {
  result <- test_list_set_elt_with()

  expect_true(is.list(result))
  expect_equal(length(result), 2L)

  # First element should be integer vector of length 3
  expect_true(is.integer(result[[1]]))
  expect_equal(length(result[[1]]), 3L)

  # Second element should be numeric vector of length 4
  expect_true(is.double(result[[2]]))
  expect_equal(length(result[[2]]), 4L)
})

# =============================================================================
# StrVecBuilder tests
# =============================================================================

test_that("test_strvec_builder_length creates string vector with correct length", {
  expect_equal(test_strvec_builder_length(0L), 0L)
  expect_equal(test_strvec_builder_length(1L), 1L)
  expect_equal(test_strvec_builder_length(5L), 5L)
  expect_equal(test_strvec_builder_length(100L), 100L)
})

test_that("test_strvec_builder_set creates character vector with expected values", {
  result <- test_strvec_builder_set()

  # Vec<Option<String>> converts to character vector with NA for None
  expect_true(is.character(result))
  expect_equal(length(result), 4L)

  # First two elements are strings
  expect_equal(result[1], "hello")
  expect_equal(result[2], "world")

  # Third element is NA
  expect_true(is.na(result[3]))

  # Fourth element is a string
  expect_equal(result[4], "test")
})

test_that("test_strvec_set_str creates character vector with expected values", {
  result <- test_strvec_set_str()

  # Vec<Option<String>> converts to character vector with NA for None
  expect_true(is.character(result))
  expect_equal(length(result), 3L)

  expect_equal(result[1], "first")
  expect_equal(result[2], "second")
  expect_true(is.na(result[3]))  # NA
})

# =============================================================================
# ReprotectSlot tests
# =============================================================================

test_that("test_reprotect_slot_accumulate replaces vectors without leaking", {
  # After replacing the vector n times, the final length should be n
  expect_equal(test_reprotect_slot_accumulate(1L), 1L)
  expect_equal(test_reprotect_slot_accumulate(5L), 5L)
  expect_equal(test_reprotect_slot_accumulate(10L), 10L)
})

test_that("test_reprotect_slot_count tracks protection count correctly", {
  # Returns 1 on success (correct protection counts), 0 on failure
  expect_equal(test_reprotect_slot_count(), 1L)
})

test_that("test_reprotect_slot_no_growth does not increase protect stack", {
  # Returns 1 on success (no stack growth), 0 on failure
  expect_equal(test_reprotect_slot_no_growth(1L), 1L)
  expect_equal(test_reprotect_slot_no_growth(10L), 1L)
  expect_equal(test_reprotect_slot_no_growth(100L), 1L)
})
