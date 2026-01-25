# Tests for #[miniextendr(as = "...")] attribute and R's as.<class>() coercion methods.
#
# These tests verify that the `as = "..."` attribute generates proper S3 methods
# for R's coercion generics like as.data.frame(), as.list(), as.character().

# =============================================================================
# AsCoerceTestData tests - successful coercions
# =============================================================================

test_that("as.data.frame.AsCoerceTestData works", {
  obj <- AsCoerceTestData$new(c("Alice", "Bob", "Charlie"), c(1.0, 2.0, 3.0))

  df <- as.data.frame(obj)

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 3)
  expect_equal(ncol(df), 2)
  expect_equal(names(df), c("name", "value"))
  expect_equal(df$name, c("Alice", "Bob", "Charlie"))
  expect_equal(df$value, c(1.0, 2.0, 3.0))
})

test_that("as.list.AsCoerceTestData works", {
  obj <- AsCoerceTestData$new(c("x", "y"), c(10.0, 20.0))

  lst <- as.list(obj)

  expect_type(lst, "list")
  expect_equal(length(lst), 2)
  expect_equal(names(lst), c("names", "values"))
  expect_equal(lst$names, c("x", "y"))
  expect_equal(lst$values, c(10.0, 20.0))
})

test_that("as.character.AsCoerceTestData works", {
  obj <- AsCoerceTestData$new(c("a", "b"), c(1.0, 2.0))

  chr <- as.character(obj)

  expect_type(chr, "character")
  expect_length(chr, 1)
  expect_match(chr, "AsCoerceTestData")
  expect_match(chr, "2 items")
})

test_that("as.numeric.AsCoerceTestData works", {
  obj <- AsCoerceTestData$new(c("a", "b", "c"), c(1.5, 2.5, 3.5))

  num <- as.numeric(obj)

  expect_type(num, "double")
  expect_equal(num, c(1.5, 2.5, 3.5))
})

test_that("as.integer.AsCoerceTestData works", {
  obj <- AsCoerceTestData$new(c("a", "b", "c"), c(10.9, 20.1, 30.5))

  int <- as.integer(obj)

  expect_type(int, "integer")
  expect_equal(int, c(10L, 20L, 30L))  # truncated, not rounded
})

# =============================================================================
# AsCoerceErrorTest tests - error handling
# =============================================================================

test_that("as.data.frame.AsCoerceErrorTest returns error when empty", {
  obj <- AsCoerceErrorTest$new(TRUE)  # is_empty = TRUE

  expect_error(
    as.data.frame(obj),
    "cannot create data.frame from empty data"
  )
})

test_that("as.data.frame.AsCoerceErrorTest works when not empty", {
  obj <- AsCoerceErrorTest$new(FALSE)  # is_empty = FALSE

  df <- as.data.frame(obj)

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 1)
  expect_equal(df$value, 42.0)
})

test_that("as.list.AsCoerceErrorTest returns NotSupported error", {
  obj <- AsCoerceErrorTest$new(FALSE)

  expect_error(
    as.list(obj),
    "cannot coerce AsCoerceErrorTest to list"
  )
})

test_that("as.character.AsCoerceErrorTest returns custom error", {
  obj <- AsCoerceErrorTest$new(FALSE)

  expect_error(
    as.character(obj),
    "intentional error for testing"
  )
})

# =============================================================================
# Edge cases
# =============================================================================

test_that("as.data.frame.AsCoerceTestData works with empty data", {
  obj <- AsCoerceTestData$new(character(0), numeric(0))

  df <- as.data.frame(obj)

  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 0)
  expect_equal(ncol(df), 2)
})

test_that("as.list.AsCoerceTestData works with single element", {
  obj <- AsCoerceTestData$new("only", 42.0)

  lst <- as.list(obj)

  expect_equal(lst$names, "only")
  expect_equal(lst$values, 42.0)
})

test_that("len method works via S3 generic", {
  obj <- AsCoerceTestData$new(c("a", "b", "c"), c(1, 2, 3))

  expect_equal(len(obj), 3L)
})
