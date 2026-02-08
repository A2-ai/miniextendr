# Class system tests
#
# Tests all 5 class systems exported by producer.pkg:
# Env-style, R6-style, S3-style, S4-style, S7-style

# =============================================================================
# SharedData (Env-style)
# =============================================================================

test_that("SharedData create and accessors work", {
  d <- SharedData$create(1.5, 2.5, "hello")
  expect_equal(d$get_x(), 1.5)
  expect_equal(d$get_y(), 2.5)
  expect_equal(d$get_label(), "hello")
})

test_that("SharedData has correct class", {
  d <- SharedData$create(0, 0, "origin")
  expect_true("SharedData" %in% class(d))
})

# =============================================================================
# EnvPoint (Env-style, default)
# =============================================================================

test_that("EnvPoint construction and read accessors work", {
  p <- EnvPoint$new(3.0, 4.0)
  expect_equal(p$x(), 3.0)
  expect_equal(p$y(), 4.0)
  expect_equal(p$distance_from_origin(), 5.0)
})

test_that("EnvPoint mutation via add works", {
  p <- EnvPoint$new(1.0, 2.0)
  p$add(10.0, 20.0)
  expect_equal(p$x(), 11.0)
  expect_equal(p$y(), 22.0)
})

test_that("EnvPoint has correct class", {
  p <- EnvPoint$new(0, 0)
  expect_true("EnvPoint" %in% class(p))
})

# =============================================================================
# R6Point (R6-style)
# =============================================================================

test_that("R6Point construction and read accessors work", {
  p <- R6Point$new(3.0, 4.0)
  expect_equal(p$x(), 3.0)
  expect_equal(p$y(), 4.0)
  expect_equal(p$distance_from_origin(), 5.0)
})

test_that("R6Point mutation via add works", {
  p <- R6Point$new(1.0, 2.0)
  p$add(10.0, 20.0)
  expect_equal(p$x(), 11.0)
  expect_equal(p$y(), 22.0)
})

test_that("R6Point is an R6 object", {
  p <- R6Point$new(0, 0)
  expect_true(inherits(p, "R6"))
  expect_true(inherits(p, "R6Point"))
})

# =============================================================================
# S3Point (S3-style)
# =============================================================================

test_that("S3Point construction and read accessors work", {
  p <- new_s3point(3.0, 4.0)
  expect_equal(s3point_x(p), 3.0)
  expect_equal(s3point_y(p), 4.0)
  expect_equal(s3point_distance(p), 5.0)
})

test_that("S3Point mutation via s3point_add works", {
  p <- new_s3point(1.0, 2.0)
  s3point_add(p, 10.0, 20.0)
  expect_equal(s3point_x(p), 11.0)
  expect_equal(s3point_y(p), 22.0)
})

test_that("S3Point has correct class", {
  p <- new_s3point(0, 0)
  expect_true(inherits(p, "S3Point"))
})

# =============================================================================
# S4Point (S4-style)
# =============================================================================

test_that("S4Point construction and read accessors work", {
  p <- S4Point(3.0, 4.0)
  expect_equal(s4_x(p), 3.0)
  expect_equal(s4_y(p), 4.0)
  expect_equal(s4_distance(p), 5.0)
})

test_that("S4Point mutation via s4_add works", {
  p <- S4Point(1.0, 2.0)
  s4_add(p, 10.0, 20.0)
  expect_equal(s4_x(p), 11.0)
  expect_equal(s4_y(p), 22.0)
})

test_that("S4Point is an S4 object", {
  p <- S4Point(0, 0)
  expect_true(isS4(p))
  expect_true(is(p, "S4Point"))
})

# =============================================================================
# S7Point (S7-style)
# =============================================================================

test_that("S7Point construction and read accessors work", {
  p <- S7Point(3.0, 4.0)
  expect_equal(s7point_x(p), 3.0)
  expect_equal(s7point_y(p), 4.0)
  expect_equal(s7point_distance(p), 5.0)
})

test_that("S7Point mutation via s7point_add works", {
  p <- S7Point(1.0, 2.0)
  s7point_add(p, 10.0, 20.0)
  expect_equal(s7point_x(p), 11.0)
  expect_equal(s7point_y(p), 22.0)
})

test_that("S7Point is an S7 object", {
  p <- S7Point(0, 0)
  expect_true(S7::S7_inherits(p, S7Point))
})
