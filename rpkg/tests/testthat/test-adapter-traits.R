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

  # ROrd$cmp returns -1, 0, or 1
  expect_equal(p1$ROrd$cmp(p2), -1L)  # p1 < p2
  expect_equal(p2$ROrd$cmp(p1), 1L)   # p2 > p1
  expect_equal(p1$ROrd$cmp(p3), 0L)   # p1 == p3

  # Standalone calling convention
  expect_equal(Point$ROrd$cmp(p1, p2), -1L)
})

test_that("Point - RClone clone produces equal values", {
  p1 <- Point$new(5L, 10L)
  p2_ptr <- p1$RClone$clone()
  # Wrap the raw pointer as a Point
  class(p2_ptr) <- "Point"

  # Clone should produce equal values
  expect_equal(p1$x(), p2_ptr$x())
  expect_equal(p1$y(), p2_ptr$y())
})

test_that("Point - RDefault default creates (0, 0)", {
  p_ptr <- Point$RDefault$default()
  # Wrap the raw pointer as a Point
  class(p_ptr) <- "Point"

  # Default Point should be (0, 0)
  expect_equal(p_ptr$x(), 0L)
  expect_equal(p_ptr$y(), 0L)
})

test_that("Point - RFromStr from_str parses string to Point", {
  # Valid string parses correctly - tests &str parameter on worker thread
  p_ptr <- Point$RFromStr$from_str("(10, 20)")
  expect_false(is.null(p_ptr))
  # Wrap the raw pointer as a Point
  class(p_ptr) <- "Point"
  expect_equal(p_ptr$x(), 10L)
  expect_equal(p_ptr$y(), 20L)

  # Invalid string returns error (Option<T> with None becomes R error)
  expect_error(Point$RFromStr$from_str("invalid"), "returned None")

  # Empty parens with valid numbers
  p2_ptr <- Point$RFromStr$from_str("(-5, 15)")
  expect_false(is.null(p2_ptr))
  class(p2_ptr) <- "Point"
  expect_equal(p2_ptr$x(), -5L)
  expect_equal(p2_ptr$y(), 15L)
})

test_that("Point - RCopy works", {
  p1 <- Point$new(7L, 8L)

  # copy() creates a bitwise copy
  p2_ptr <- p1$RCopy$copy()
  class(p2_ptr) <- "Point"

  expect_equal(p1$x(), p2_ptr$x())
  expect_equal(p1$y(), p2_ptr$y())

  # is_copy() returns TRUE for Copy types
  expect_true(p1$RCopy$is_copy())
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
  expect_equal(f1$RPartialOrd$partial_cmp(f2), -1L)  # 1.0 < 2.0
  expect_equal(f2$RPartialOrd$partial_cmp(f1), 1L)   # 2.0 > 1.0
  expect_equal(f1$RPartialOrd$partial_cmp(f3), 0L)   # 1.0 == 1.0

  # Standalone calling convention
  expect_equal(MyFloat$RPartialOrd$partial_cmp(f1, f2), -1L)
})

test_that("MyFloat - RPartialOrd handles NaN correctly", {
  f1 <- MyFloat$new(1.0)
  nan <- MyFloat$nan()

  # NaN is not comparable to anything (including itself)
  # Option<T> returning None throws error in miniextendr
  expect_error(nan$RPartialOrd$partial_cmp(f1), "returned None")
  expect_error(f1$RPartialOrd$partial_cmp(nan), "returned None")
  expect_error(nan$RPartialOrd$partial_cmp(nan), "returned None")
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

  expect_equal(it$RIterator$next_item(), 1L)
  expect_equal(it$RIterator$next_item(), 2L)
  expect_equal(it$RIterator$next_item(), 3L)
  # Option<T> returning None throws error in miniextendr
  expect_error(it$RIterator$next_item(), "returned None")
})

test_that("IntVecIter - RIterator next_item and count work via trait namespace", {
  it <- IntVecIter$new(c(10L, 20L, 30L, 40L))

  # Trait methods are accessed via RIterator namespace
  expect_equal(it$RIterator$next_item(), 10L)
  expect_equal(it$RIterator$count(), 3L)  # 3 remaining after consuming 1
})

test_that("IntVecIter - count works", {
  it <- IntVecIter$new(c(1L, 2L, 3L, 4L, 5L))
  expect_equal(it$RIterator$count(), 5L)

  # After count, iterator is exhausted - Option<T> returning None throws
  expect_error(it$RIterator$next_item(), "returned None")
})

test_that("IntVecIter - collect_n works", {
  it <- IntVecIter$new(c(1L, 2L, 3L, 4L, 5L))

  # Collect first 3
  first3 <- it$RIterator$collect_n(3L)
  expect_equal(first3, c(1L, 2L, 3L))

  # Remaining elements
  expect_equal(it$RIterator$next_item(), 4L)
  expect_equal(it$RIterator$next_item(), 5L)
})

test_that("IntVecIter - skip works", {
  it <- IntVecIter$new(c(1L, 2L, 3L, 4L, 5L))

  # Skip 2 elements, returns count skipped
  skipped <- it$RIterator$skip(2L)
  expect_equal(skipped, 2L)

  # Next element is 3
  expect_equal(it$RIterator$next_item(), 3L)
})

test_that("IntVecIter - nth works", {
  it <- IntVecIter$new(c(10L, 20L, 30L, 40L))

  # nth(0) is the first element
  expect_equal(it$RIterator$nth(0L), 10L)

  # nth(1) skips one more and returns the next
  expect_equal(it$RIterator$nth(1L), 30L)

  # nth past end throws error - Option<T> returning None throws
  expect_error(it$RIterator$nth(10L), "returned None")
})

# =============================================================================
# GrowableVec - RExtend tests
# =============================================================================

test_that("GrowableVec - basic operations work", {
  v <- GrowableVec$new()

  expect_equal(v$RExtend$len(), 0L)
  expect_true(v$RExtend$is_empty())
  expect_equal(v$to_vec(), integer(0))
})

test_that("GrowableVec - RExtend extend_from_vec works", {
  v <- GrowableVec$new()

  v$RExtend$extend_from_vec(c(1L, 2L, 3L))
  expect_equal(v$RExtend$len(), 3L)
  expect_false(v$RExtend$is_empty())
  expect_equal(v$to_vec(), c(1L, 2L, 3L))

  # Extend again
  v$RExtend$extend_from_vec(c(4L, 5L))
  expect_equal(v$RExtend$len(), 5L)
  expect_equal(v$to_vec(), c(1L, 2L, 3L, 4L, 5L))
})

test_that("GrowableVec - from_vec works", {
  v <- GrowableVec$from_vec(c(10L, 20L, 30L))

  expect_equal(v$RExtend$len(), 3L)
  expect_equal(v$to_vec(), c(10L, 20L, 30L))
})

test_that("GrowableVec - clear works", {
  v <- GrowableVec$from_vec(c(1L, 2L, 3L))
  expect_equal(v$RExtend$len(), 3L)

  v$clear()
  expect_equal(v$RExtend$len(), 0L)
  expect_true(v$RExtend$is_empty())
})

# =============================================================================
# IntSet - RFromIter and RToVec tests
# =============================================================================

test_that("IntSet - RFromIter from_vec works with deduplication", {
  # HashSet automatically deduplicates
  s <- IntSet$RFromIter$from_vec(c(1L, 2L, 2L, 3L, 3L, 3L))
  class(s) <- "IntSet"

  expect_equal(s$RToVec$len(), 3L)
  expect_false(s$RToVec$is_empty())
})

test_that("IntSet - RToVec to_vec works", {
  s <- IntSet$RFromIter$from_vec(c(3L, 1L, 2L))
  class(s) <- "IntSet"

  # to_vec returns sorted vector
  v <- s$RToVec$to_vec()
  expect_equal(v, c(1L, 2L, 3L))
})

test_that("IntSet - contains works", {
  s <- IntSet$RFromIter$from_vec(c(10L, 20L, 30L))
  class(s) <- "IntSet"

  expect_true(s$contains(10L))
  expect_true(s$contains(20L))
  expect_true(s$contains(30L))
  expect_false(s$contains(15L))
  expect_false(s$contains(0L))
})

test_that("IntSet - empty set works", {
  s <- IntSet$RFromIter$from_vec(integer(0))
  class(s) <- "IntSet"

  expect_equal(s$RToVec$len(), 0L)
  expect_true(s$RToVec$is_empty())
  expect_equal(s$RToVec$to_vec(), integer(0))
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

  # Create two independent iterators via trait namespace
  it1_ptr <- v$RMakeIter$make_iter()
  it2_ptr <- v$RMakeIter$make_iter()

  # Wrap raw pointers
  class(it1_ptr) <- "IterableVecIter"
  class(it2_ptr) <- "IterableVecIter"

  # Advance it1 once
  expect_equal(it1_ptr$RIterator$next_item(), 1L)

  # it2 should still be at the start
  expect_equal(it2_ptr$RIterator$next_item(), 1L)
  expect_equal(it2_ptr$RIterator$next_item(), 2L)

  # Continue with it1
  expect_equal(it1_ptr$RIterator$next_item(), 2L)
  expect_equal(it1_ptr$RIterator$next_item(), 3L)
})

test_that("IterableVecIter - collect via RIterator works", {
  v <- IterableVec$new(c(100L, 200L, 300L))
  it_ptr <- v$RMakeIter$make_iter()
  class(it_ptr) <- "IterableVecIter"

  result <- it_ptr$RIterator$collect_n(10L)
  expect_equal(result, c(100L, 200L, 300L))
})
