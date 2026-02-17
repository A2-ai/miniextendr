skip_if_missing_feature("rayon")

test_that("parallel struct fill (above threshold)", {
  df <- create_large_par_points(5000L)
  expect_s3_class(df, "data.frame")
  expect_equal(nrow(df), 5000)
  expect_equal(df$x[1], 0)
  expect_equal(df$x[5000], 4999)
  expect_equal(df$label[1], "pt_0")
  expect_equal(df$label[5000], "pt_4999")
})

test_that("parallel enum fill (above threshold)", {
  df <- create_large_par_events(6000L)
  expect_equal(nrow(df), 6000)
  expect_equal(df$`_kind`[1], "A")
  expect_equal(df$`_kind`[2], "B")
  expect_equal(df$value[1], 0)
  expect_true(is.na(df$value[2]))
  expect_true(is.na(df$name[1]))
  expect_equal(df$name[2], "evt_1")
})

test_that("sequential fallback (below threshold)", {
  df <- create_large_par_points(100L)
  expect_equal(nrow(df), 100)
  expect_equal(df$x, as.numeric(0:99))
})
