# Test adapter traits (RDebug, RDisplay, RHash, ROrd, etc.)

test_that("Point - RDebug works", {
  p <- Point$new(3L, 4L)

  # debug_str() returns compact format
  debug <- p$debug_str()
  expect_true(is.character(debug))
  expect_true(grepl("Point", debug))
  expect_true(grepl("3", debug))
  expect_true(grepl("4", debug))

  # debug_str_pretty() returns formatted version
  pretty <- p$debug_str_pretty()
  expect_true(is.character(pretty))
})

test_that("Point - RDisplay works", {
  p <- Point$new(3L, 4L)

  # to_r_string() returns user-friendly format
  s <- p$to_r_string()
  expect_equal(s, "(3, 4)")
})

test_that("Point - RHash works", {
  p1 <- Point$new(1L, 2L)
  p2 <- Point$new(1L, 2L)
  p3 <- Point$new(3L, 4L)

  # Same values should produce same hash
  h1 <- p1$hash()
  h2 <- p2$hash()
  h3 <- p3$hash()

  expect_true(is.numeric(h1))
  expect_equal(h1, h2)
  # Different values should (very likely) produce different hash
  # Note: collision is theoretically possible but extremely unlikely
  expect_true(h1 != h3)
})

test_that("Point - ROrd works", {
  # Skip ROrd tests - requires &Point parameter which needs special handling
  # The adapter trait methods work, but passing Point references in R
  # requires additional R wrapper infrastructure.
  skip("ROrd methods require &Point parameter - not yet supported")
})

test_that("Point - RClone works", {
  p1 <- Point$new(5L, 10L)
  p2_ptr <- p1$clone_point()
  # Wrap the raw pointer as a Point
  class(p2_ptr) <- "Point"

  # Clone should produce equal values
  expect_equal(p1$x(), p2_ptr$x())
  expect_equal(p1$y(), p2_ptr$y())
})

test_that("Point - RDefault works", {
  p_ptr <- Point$default_point()
  # Wrap the raw pointer as a Point
  class(p_ptr) <- "Point"

  # Default Point should be (0, 0)
  expect_equal(p_ptr$x(), 0L)
  expect_equal(p_ptr$y(), 0L)
})

test_that("Point - RFromStr works with &str parameter", {
  # Valid string parses correctly - tests &str parameter on worker thread
  p_ptr <- Point$from_str("(10, 20)")
  expect_false(is.null(p_ptr))
  # Wrap the raw pointer as a Point
  class(p_ptr) <- "Point"
  expect_equal(p_ptr$x(), 10L)
  expect_equal(p_ptr$y(), 20L)

  # Invalid string returns error (Option<T> with None becomes R error)
  expect_error(Point$from_str("invalid"), "returned None")

  # Empty parens with valid numbers
  p2_ptr <- Point$from_str("(-5, 15)")
  expect_false(is.null(p2_ptr))
  class(p2_ptr) <- "Point"
  expect_equal(p2_ptr$x(), -5L)
  expect_equal(p2_ptr$y(), 15L)
})
