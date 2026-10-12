# A type's methods split across two `#[miniextendr]` impl blocks: the class is
# defined once and the methods of both blocks attach to it (#1776). Fixtures in
# src/rust/split_impl_tests.rs; the first block of each type has the
# constructor, the second adds methods.

test_that("an env class keeps the methods of both impl blocks", {
  obj <- SplitEnv$new(5L)
  expect_identical(obj$value(), 5L)
  obj$add(2L)
  expect_identical(obj$value(), 7L)
  expect_identical(obj$doubled(), 14L)
  expect_identical(SplitEnv$origin(), 0L)
})

test_that("an R6 class keeps the members of both impl blocks", {
  obj <- SplitR6$new(5L)
  expect_identical(obj$value(), 5L)
  obj$add(2L)
  expect_identical(obj$value(), 7L)
  expect_identical(obj$doubled, 14L)
  expect_identical(obj$.__enclos_env__$private$predecessor(), 6L)
  expect_true(is.function(obj$.__enclos_env__$private$finalize))
  expect_identical(SplitR6$origin(), 0L)
})

test_that("an R6 deep_clone from a second impl block runs on a deep clone", {
  obj <- SplitR6Clone$new(1L)
  deep <- obj$clone(deep = TRUE)
  deep$set(5L)
  # The handler gave the deep clone its own Rust value.
  expect_identical(obj$value(), 1L)
  expect_identical(deep$value(), 5L)
  # A shallow clone shares the pointer: the handler did not run.
  shallow <- obj$clone()
  shallow$set(9L)
  expect_identical(obj$value(), 9L)
})

test_that("an S3 class keeps the methods of both impl blocks", {
  obj <- SplitS3$new(5L)
  expect_s3_class(obj, "SplitS3")
  expect_identical(split3_value(obj), 5L)
  expect_identical(split3_doubled(obj), 10L)
  expect_identical(splits3_origin(), 0L)
})

test_that("an S4 class keeps the methods of both impl blocks", {
  obj <- SplitS4(5L)
  expect_true(methods::is(obj, "SplitS4"))
  expect_identical(s4_split4_value(obj), 5L)
  expect_identical(s4_split4_doubled(obj), 10L)
  expect_identical(SplitS4_origin(), 0L)
})

test_that("an S7 class keeps the methods of both impl blocks", {
  obj <- SplitS7(5L)
  expect_true(S7::S7_inherits(obj, SplitS7))
  expect_identical(split7_value(obj), 5L)
  expect_identical(split7_doubled(obj), 10L)
  expect_identical(SplitS7_split7_doubled(obj), 10L)
  expect_identical(SplitS7_origin(), 0L)
})

test_that("a vctrs class keeps the functions of both impl blocks", {
  v <- new_splitvctrs(c(1, 2.5))
  expect_s3_class(v, "SplitVctrs")
  expect_identical(vctrs::vec_ptype_abbr(v), "split")
  expect_identical(vctrs::vec_c(v, v), new_splitvctrs(c(1, 2.5, 1, 2.5)))
  expect_identical(splitvctrs_total(c(1, 2.5)), 3.5)
})
