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

  # as_r_string() returns user-friendly format
  s <- p$as_r_string()
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
  p1 <- Point$new(1L, 2L)
  p2 <- Point$new(3L, 4L)
  p3 <- Point$new(1L, 2L)

  # cmp_to returns -1, 0, or 1
  expect_equal(p1$cmp_to(p2), -1L)  # p1 < p2
  expect_equal(p2$cmp_to(p1), 1L)   # p2 > p1
  expect_equal(p1$cmp_to(p3), 0L)   # p1 == p3

  # Comparison helpers
  expect_true(p1$is_less_than(p2))
  expect_false(p1$is_greater_than(p2))
  expect_true(p1$is_equal_to(p3))
  expect_true(p2$is_greater_than(p1))
})

test_that("Point - clone produces equal values", {
  p1 <- Point$new(5L, 10L)
  p2_ptr <- p1$clone_point()
  # Wrap the raw pointer as a Point
  class(p2_ptr) <- "Point"

  # Clone should produce equal values
  expect_equal(p1$x(), p2_ptr$x())
  expect_equal(p1$y(), p2_ptr$y())
})

test_that("Point - default creates (0, 0)", {
  p_ptr <- Point$default_point()
  # Wrap the raw pointer as a Point
  class(p_ptr) <- "Point"

  # Default Point should be (0, 0)
  expect_equal(p_ptr$x(), 0L)
  expect_equal(p_ptr$y(), 0L)
})

test_that("Point - from_str parses string to Point", {
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

test_that("Point - RCopy works", {
  p1 <- Point$new(7L, 8L)

  # copy_point() creates a bitwise copy
  p2_ptr <- p1$copy_point()
  class(p2_ptr) <- "Point"

  expect_equal(p1$x(), p2_ptr$x())
  expect_equal(p1$y(), p2_ptr$y())

  # is_copy() returns TRUE for Copy types
  expect_true(p1$is_copy())
})

# =============================================================================
# MyFloat - RPartialOrd tests
# =============================================================================

test_that("MyFloat - basic operations work", {
  f1 <- MyFloat$new(1.5)
  f2 <- MyFloat$new(2.5)

  expect_equal(f1$value(), 1.5)
  expect_equal(f2$value(), 2.5)
})

test_that("MyFloat - RPartialOrd works for comparable values", {
  f1 <- MyFloat$new(1.0)
  f2 <- MyFloat$new(2.0)
  f3 <- MyFloat$new(1.0)

  # Comparison results: -1 (less), 0 (equal), 1 (greater)
  expect_equal(f1$partial_cmp_to(f2), -1L)  # 1.0 < 2.0
  expect_equal(f2$partial_cmp_to(f1), 1L)   # 2.0 > 1.0
  expect_equal(f1$partial_cmp_to(f3), 0L)   # 1.0 == 1.0

  # Helper methods
  expect_true(f1$is_less_than(f2))
  expect_false(f1$is_greater_than(f2))
  expect_true(f1$is_equal_to(f3))
  expect_true(f1$is_comparable(f2))
})

test_that("MyFloat - RPartialOrd handles NaN correctly", {
  f1 <- MyFloat$new(1.0)
  nan <- MyFloat$nan()

  # NaN is not comparable to anything (including itself)
  # Option<T> returning None throws error in miniextendr
  expect_error(nan$partial_cmp_to(f1), "returned None")
  expect_error(f1$partial_cmp_to(nan), "returned None")
  expect_error(nan$partial_cmp_to(nan), "returned None")

  expect_false(nan$is_comparable(f1))
  expect_false(f1$is_comparable(nan))
})

# =============================================================================
# ChainedError - RError tests
# =============================================================================

test_that("ChainedError - basic creation works", {
  err <- ChainedError$new("outer error", "inner cause")
  expect_s3_class(err, "ChainedError")
})

test_that("ChainedError - RError error_message works", {
  err <- ChainedError$new("file not found", "permission denied")
  msg <- err$error_message()

  expect_true(is.character(msg))
  expect_equal(msg, "file not found")
})
test_that("ChainedError - RError error_chain works", {
  err <- ChainedError$new("outer", "inner")
  chain <- err$error_chain()

  expect_true(is.character(chain))
  expect_equal(length(chain), 2L)
  expect_equal(chain[1], "outer")
  expect_equal(chain[2], "inner")
})

test_that("ChainedError - RError error_chain_length works", {
  err_with_source <- ChainedError$new("outer", "inner")
  err_no_source <- ChainedError$without_source("standalone error")

  expect_equal(err_with_source$error_chain_length(), 2L)
  expect_equal(err_no_source$error_chain_length(), 1L)
})

# =============================================================================
# IntVecIter - RIterator tests
# =============================================================================

test_that("IntVecIter - basic iteration works", {
  it <- IntVecIter$new(c(1L, 2L, 3L))

  expect_equal(it$next_item(), 1L)
  expect_equal(it$next_item(), 2L)
  expect_equal(it$next_item(), 3L)
  # Option<T> returning None throws error in miniextendr
  expect_error(it$next_item(), "returned None")
})

test_that("IntVecIter - size_hint works", {
  it <- IntVecIter$new(c(10L, 20L, 30L, 40L))
  hint <- it$size_hint()

  # size_hint returns [lower_bound, upper_bound] where -1 means unknown
  expect_equal(length(hint), 2L)
  expect_equal(hint[1], 4L)  # exact size known
  expect_equal(hint[2], 4L)  # upper bound also known
})

test_that("IntVecIter - count works", {
  it <- IntVecIter$new(c(1L, 2L, 3L, 4L, 5L))
  expect_equal(it$count(), 5L)

  # After count, iterator is exhausted - Option<T> returning None throws
  expect_error(it$next_item(), "returned None")
})

test_that("IntVecIter - collect_n works", {
  it <- IntVecIter$new(c(1L, 2L, 3L, 4L, 5L))

  # Collect first 3
  first3 <- it$collect_n(3L)
  expect_equal(first3, c(1L, 2L, 3L))

  # Remaining elements
  expect_equal(it$next_item(), 4L)
  expect_equal(it$next_item(), 5L)
})

test_that("IntVecIter - skip works", {
  it <- IntVecIter$new(c(1L, 2L, 3L, 4L, 5L))

  # Skip 2 elements, returns count skipped
  skipped <- it$skip(2L)
  expect_equal(skipped, 2L)

  # Next element is 3
  expect_equal(it$next_item(), 3L)
})

test_that("IntVecIter - nth works", {
  it <- IntVecIter$new(c(10L, 20L, 30L, 40L))

  # nth(0) is the first element
  expect_equal(it$nth(0L), 10L)

  # nth(1) skips one more and returns the next
  expect_equal(it$nth(1L), 30L)

  # nth past end throws error - Option<T> returning None throws
  expect_error(it$nth(10L), "returned None")
})

# =============================================================================
# GrowableVec - RExtend tests
# =============================================================================

test_that("GrowableVec - basic operations work", {
  v <- GrowableVec$new()

  expect_equal(v$len(), 0L)
  expect_true(v$is_empty())
  expect_equal(v$to_vec(), integer(0))
})

test_that("GrowableVec - RExtend extend_from_vec works", {
  v <- GrowableVec$new()

  v$extend(c(1L, 2L, 3L))
  expect_equal(v$len(), 3L)
  expect_false(v$is_empty())
  expect_equal(v$to_vec(), c(1L, 2L, 3L))

  # Extend again
  v$extend(c(4L, 5L))
  expect_equal(v$len(), 5L)
  expect_equal(v$to_vec(), c(1L, 2L, 3L, 4L, 5L))
})

test_that("GrowableVec - from_vec works", {
  v <- GrowableVec$from_vec(c(10L, 20L, 30L))

  expect_equal(v$len(), 3L)
  expect_equal(v$to_vec(), c(10L, 20L, 30L))
})

test_that("GrowableVec - clear works", {
  v <- GrowableVec$from_vec(c(1L, 2L, 3L))
  expect_equal(v$len(), 3L)

  v$clear()
  expect_equal(v$len(), 0L)
  expect_true(v$is_empty())
})

# =============================================================================
# IntSet - RFromIter and RToVec tests
# =============================================================================

test_that("IntSet - RFromIter from_vec works with deduplication", {
  # HashSet automatically deduplicates
  s <- IntSet$from_vec(c(1L, 2L, 2L, 3L, 3L, 3L))

  expect_equal(s$len(), 3L)
  expect_false(s$is_empty())
})

test_that("IntSet - RToVec to_vec works", {
  s <- IntSet$from_vec(c(3L, 1L, 2L))

  # to_vec returns sorted vector
  v <- s$to_vec()
  expect_equal(v, c(1L, 2L, 3L))
})

test_that("IntSet - contains works", {
  s <- IntSet$from_vec(c(10L, 20L, 30L))

  expect_true(s$contains(10L))
  expect_true(s$contains(20L))
  expect_true(s$contains(30L))
  expect_false(s$contains(15L))
  expect_false(s$contains(0L))
})

test_that("IntSet - empty set works", {
  s <- IntSet$from_vec(integer(0))

  expect_equal(s$len(), 0L)
  expect_true(s$is_empty())
  expect_equal(s$to_vec(), integer(0))
})

# =============================================================================
# IterableVec and IterableVecIter - RMakeIter tests
# =============================================================================

test_that("IterableVec - basic operations work", {
  v <- IterableVec$new(c(1L, 2L, 3L))

  expect_equal(v$len(), 3L)
  expect_equal(v$to_vec(), c(1L, 2L, 3L))
})

test_that("IterableVec - RMakeIter creates independent iterators", {
  v <- IterableVec$new(c(1L, 2L, 3L))

  # Create two independent iterators
  it1_ptr <- v$make_iter()
  it2_ptr <- v$make_iter()

  # Wrap raw pointers
  class(it1_ptr) <- "IterableVecIter"
  class(it2_ptr) <- "IterableVecIter"

  # Advance it1 once
  expect_equal(it1_ptr$next_item(), 1L)

  # it2 should still be at the start
  expect_equal(it2_ptr$next_item(), 1L)
  expect_equal(it2_ptr$next_item(), 2L)

  # Continue with it1
  expect_equal(it1_ptr$next_item(), 2L)
  expect_equal(it1_ptr$next_item(), 3L)
})

test_that("IterableVecIter - size_hint works", {
  v <- IterableVec$new(c(10L, 20L, 30L))
  it_ptr <- v$make_iter()
  class(it_ptr) <- "IterableVecIter"

  hint <- it_ptr$size_hint()
  expect_equal(length(hint), 2L)
  expect_equal(hint[1], 3L)  # exact size known
  expect_equal(hint[2], 3L)  # upper bound also known
})

test_that("IterableVecIter - collect_all works", {
  v <- IterableVec$new(c(100L, 200L, 300L))
  it_ptr <- v$make_iter()
  class(it_ptr) <- "IterableVecIter"

  result <- it_ptr$collect_all()
  expect_equal(result, c(100L, 200L, 300L))
})
