# Tests for #[miniextendr(as = "...")] attribute and R's as.<class>() coercion methods.
#
# These tests verify that the `as = "..."` attribute generates proper S3 methods
# for R's coercion generics like as.data.frame(), as.list(), as.character().

# =============================================================================
# RCoerceTestData tests - successful coercions
# =============================================================================

test_that("as.data.frame.RCoerceTestData works", {
  obj <- RCoerceTestData$new(c("Alice", "Bob", "Charlie"), c(1.0, 2.0, 3.0))

  df <- as.data.frame(obj)

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3)
  expect_equal(ncol(df), 2)
  expect_equal(names(df), c("name", "value"))
  expect_equal(df$name, c("Alice", "Bob", "Charlie"))
  expect_equal(df$value, c(1.0, 2.0, 3.0))
})

test_that("as.list.RCoerceTestData works", {
  obj <- RCoerceTestData$new(c("x", "y"), c(10.0, 20.0))

  lst <- as.list(obj)

  expect_type(lst, "list")
  expect_equal(length(lst), 2)
  expect_equal(names(lst), c("names", "values"))
  expect_equal(lst$names, c("x", "y"))
  expect_equal(lst$values, c(10.0, 20.0))
})

test_that("as.character.RCoerceTestData works", {
  obj <- RCoerceTestData$new(c("a", "b"), c(1.0, 2.0))

  chr <- as.character(obj)

  expect_type(chr, "character")
  expect_length(chr, 1)
  expect_match(chr, "RCoerceTestData")
  expect_match(chr, "2 items")
})

test_that("as_numeric.RCoerceTestData extracts numeric values", {
  obj <- RCoerceTestData$new(c("a", "b", "c"), c(1.5, 2.5, 3.5))

  # Note: R's as.numeric() is a primitive that doesn't dispatch S3 methods for
  # externalptr objects, so we use the as_numeric() wrapper instead.
  num <- as_numeric(obj)

  expect_type(num, "double")
  expect_equal(num, c(1.5, 2.5, 3.5))
})

test_that("as.integer.RCoerceTestData works", {
  obj <- RCoerceTestData$new(c("a", "b", "c"), c(10.9, 20.1, 30.5))

  int <- as.integer(obj)

  expect_type(int, "integer")
  expect_equal(int, c(10L, 20L, 30L))  # truncated, not rounded
})

# =============================================================================
# RCoerceErrorTest tests - error handling
# =============================================================================

test_that("as.data.frame.RCoerceErrorTest returns error when empty", {
  obj <- RCoerceErrorTest$new(TRUE)  # is_empty = TRUE

  expect_error(
    as.data.frame(obj),
    "cannot create data.frame from empty data"
  )
})

test_that("as.data.frame.RCoerceErrorTest works when not empty", {
  obj <- RCoerceErrorTest$new(FALSE)  # is_empty = FALSE

  df <- as.data.frame(obj)

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 1)
  expect_equal(df$value, 42.0)
})

test_that("as.list.RCoerceErrorTest returns NotSupported error", {
  obj <- RCoerceErrorTest$new(FALSE)

  expect_error(
    as.list(obj),
    "NotSupported"
  )
})

test_that("as.character.RCoerceErrorTest returns custom error", {
  obj <- RCoerceErrorTest$new(FALSE)

  expect_error(
    as.character(obj),
    "intentional error for testing"
  )
})

# =============================================================================
# Edge cases
# =============================================================================

test_that("as.data.frame.RCoerceTestData works with empty data", {
  obj <- RCoerceTestData$new(character(0), numeric(0))

  df <- as.data.frame(obj)

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 0)
  expect_equal(ncol(df), 2)
})

test_that("as.list.RCoerceTestData works with single element", {
  obj <- RCoerceTestData$new("only", 42.0)

  lst <- as.list(obj)

  expect_equal(lst$names, "only")
  expect_equal(lst$values, 42.0)
})

test_that("len method works via S3 generic", {
  obj <- RCoerceTestData$new(c("a", "b", "c"), c(1, 2, 3))

  expect_equal(len(obj), 3L)
})

# =============================================================================
# Exported snake_case surface (constructor aliases + underscore generics)
# =============================================================================

test_that("new_rcoercetestdata constructor alias builds a working object", {
  obj <- new_rcoercetestdata(c("a", "b"), c(1.0, 2.0))
  expect_true(inherits(obj, "RCoerceTestData"))
  expect_equal(len(obj), 2L)
})

test_that("new_rcoerceerrortest constructor alias builds a working object", {
  obj <- new_rcoerceerrortest(TRUE)
  expect_true(inherits(obj, "RCoerceErrorTest"))
  expect_error(as.data.frame(obj), "cannot create data.frame from empty data")
})

test_that("as_character generic dispatches like as.character", {
  obj <- RCoerceTestData$new(c("a", "b"), c(1.0, 2.0))
  expect_equal(as_character(obj), as.character(obj))
  expect_error(
    as_character(RCoerceErrorTest$new(FALSE)),
    "intentional error for testing"
  )
})

test_that("as_integer generic truncates values like as.integer", {
  obj <- RCoerceTestData$new(c("a", "b", "c"), c(10.9, 20.1, 30.5))
  expect_identical(as_integer(obj), c(10L, 20L, 30L))
})

test_that("as_data_frame generic dispatches like as.data.frame", {
  obj <- RCoerceTestData$new(c("a", "b"), c(1.0, 2.0))
  expect_equal(as_data_frame(obj), as.data.frame(obj))
})
