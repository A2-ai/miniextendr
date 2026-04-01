test_that("streaming integer ALTREP generates correct range", {
  v <- streaming_int_range(10L)
  expect_equal(length(v), 10L)
  expect_equal(v[1], 1L)
  expect_equal(v[10], 10L)
  for (i in 1:10) expect_equal(v[i], i)
})

test_that("streaming integer ALTREP handles large range", {
  v <- streaming_int_range(1000L)
  expect_equal(length(v), 1000L)
  expect_equal(v[1], 1L)
  expect_equal(v[500], 500L)
  expect_equal(v[1000], 1000L)
})

test_that("streaming real ALTREP generates correct squares", {
  v <- streaming_real_squares(5L)
  expect_equal(length(v), 5L)
  expect_equal(v[1], 1.0)
  expect_equal(v[2], 4.0)
  expect_equal(v[3], 9.0)
  expect_equal(v[4], 16.0)
  expect_equal(v[5], 25.0)
})

test_that("streaming real ALTREP works with sum", {
  v <- streaming_real_squares(4L)
  expect_equal(sum(v), 1 + 4 + 9 + 16)
})
