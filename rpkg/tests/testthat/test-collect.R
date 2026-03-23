# Tests for Collect / CollectStrings iterator-to-R-vector adapters

# region: Collect — native type iterators

test_that("Collect writes f64 iterator directly to R numeric", {
  result <- test_collect_sines(5L)
  expect_type(result, "double")
  expect_length(result, 5)
  expect_equal(result, sin(0:4))
})

test_that("Collect writes i32 iterator directly to R integer", {
  result <- test_collect_squares(5L)
  expect_type(result, "integer")
  expect_equal(result, c(0L, 1L, 4L, 9L, 16L))
})

test_that("Collect handles Range<i32>", {
  result <- test_collect_range()
  expect_type(result, "integer")
  expect_equal(result, 1L:10L)
})

test_that("Collect handles empty iterator", {
  result <- test_collect_empty()
  expect_type(result, "double")
  expect_length(result, 0)
})

# endregion

# region: CollectStrings — string iterators

test_that("CollectStrings converts string iterator to character vector", {
  result <- test_collect_strings_upper(c("hello", "world"))
  expect_type(result, "character")
  expect_equal(result, c("HELLO", "WORLD"))
})

test_that("CollectStrings generates numbered strings", {
  result <- test_collect_strings_numbered(3L)
  expect_equal(result, c("item_1", "item_2", "item_3"))
})

# endregion

# region: CollectNA — Option iterators with NA

test_that("CollectNA produces numeric with NA for None", {
  result <- test_collect_na_f64(6L)
  expect_type(result, "double")
  expect_length(result, 6)
  # i=0,3 are NA (i %% 3 == 0)
  expect_true(is.na(result[1]))  # i=0
  expect_equal(result[2], 1.0)   # i=1
  expect_equal(result[3], 2.0)   # i=2
  expect_true(is.na(result[4]))  # i=3
  expect_equal(result[5], 4.0)   # i=4
  expect_equal(result[6], 5.0)   # i=5
})

test_that("CollectNAInt produces integer with NA for None", {
  result <- test_collect_na_i32(5L)
  expect_type(result, "integer")
  expect_length(result, 5)
  # i=0,2,4 are NA (i %% 2 == 0)
  expect_true(is.na(result[1]))   # i=0
  expect_equal(result[2], 10L)    # i=1
  expect_true(is.na(result[3]))   # i=2
  expect_equal(result[4], 30L)    # i=3
  expect_true(is.na(result[5]))   # i=4
})

# endregion
