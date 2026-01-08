test_that("S3Counter lifecycle works", {
  c <- new_s3counter(1L)
  expect_equal(s3_value(c), 1L)
  expect_equal(s3_inc(c), 2L)
  expect_equal(s3_add(c, 5L), 7L)
  expect_equal(s3_value(s3counter_default_counter()), 0L)
})

test_that("S3Counter S3 helpers exist through method names", {
  c <- new_s3counter(2L)
  expect_equal(s3_value.S3Counter(c), 2L)
  expect_equal(s3_inc.S3Counter(c), 3L)
  expect_equal(s3_add.S3Counter(c, 4L), 7L)
})

test_that("S4Counter lifecycle works", {
  c <- S4Counter(2L)
  expect_equal(s4_value(c), 2L)
  expect_equal(s4_inc(c), 3L)
  expect_equal(s4_add(c, 4L), 7L)
  expect_equal(s4_value(S4Counter_default_counter()), 0L)
})

test_that("S7Counter lifecycle works", {
  c <- S7Counter(3L)
  expect_equal(s7_value(c), 3L)
  expect_equal(s7_inc(c), 4L)
  expect_equal(s7_add(c, 6L), 10L)
  expect_equal(s7_value(S7Counter_default_counter()), 0L)
})

test_that("ReceiverCounter env-style methods work", {
  rc <- ReceiverCounter$new(5L)
  expect_equal(rc$value(), 5L)
  expect_equal(rc$inc(), 6L)
  expect_equal(rc$add(4L), 10L)
  expect_equal(ReceiverCounter$default_counter()$value(), 0L)
})

test_that("R6Counter class works", {
  c <- R6Counter$new(10L)
  expect_equal(c$value(), 10L)
  expect_equal(c$inc(), 11L)
  expect_equal(c$add(9L), 20L)
  expect_equal(R6Counter$default_counter()$value(), 0L)
})

test_that("R6Accumulator tracks totals", {
  acc <- R6Accumulator$new()
  expect_equal(acc$total(), 0)
  expect_equal(acc$count(), 0)
  expect_equal(acc$accumulate(1.5), 1.5)
  expect_equal(acc$accumulate(2.5), 4.0)
  expect_equal(acc$count(), 2)
  expect_equal(acc$average(), 2.0)
})

test_that("Calculator defaults and methods work", {
  calc <- Calculator$new()
  expect_equal(calc$get(), 0.0)
  expect_equal(calc$add(), 1.0)
  calc$set(10.5)
  expect_equal(calc$get(), 10.5)
  expect_equal(calc$add(0.5), 11.0)
})

test_that("r6_standalone_add sums", {
  expect_equal(r6_standalone_add(2L, 3L), 5L)
})
