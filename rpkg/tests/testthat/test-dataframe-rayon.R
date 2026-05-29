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

test_that("parallel from-R reader agrees with sequential and round-trips", {
  df <- data.frame(
    x = c(1.5, 2.5, 3.5),
    y = c(10, 20, 30),
    label = c("a", "bb", "ccc"),
    stringsAsFactors = FALSE
  )
  expected <- sum(df$x + df$y + nchar(df$label))
  expect_equal(par_read_points_checksum(df), expected)
  expect_equal(seq_read_points_checksum(df), expected)
})

test_that("parallel from-R reader round-trips a generated data.frame", {
  df <- create_large_par_points(5000L)
  expected <- sum(df$x + df$y + nchar(df$label))
  expect_equal(par_read_points_checksum(df), expected)
  # Parallel and sequential readers must agree on every row.
  expect_equal(par_read_points_checksum(df), seq_read_points_checksum(df))
})

test_that("parallel from-R reader handles a zero-row data.frame", {
  df <- create_large_par_points(0L)
  expect_equal(nrow(df), 0)
  expect_equal(par_read_points_checksum(df), 0)
  expect_equal(seq_read_points_checksum(df), 0)
})
