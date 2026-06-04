test_that("time::OffsetDateTime roundtrips through POSIXct", {
  # Unix epoch
  epoch <- as.POSIXct("1970-01-01 00:00:00", tz = "UTC")
  result <- time_roundtrip_posixct(epoch)
  expect_s3_class(result, "POSIXct")
  expect_equal(as.numeric(result), as.numeric(epoch))

  # A known date
  t1 <- as.POSIXct("2024-06-04 12:34:56", tz = "UTC")
  result1 <- time_roundtrip_posixct(t1)
  expect_s3_class(result1, "POSIXct")
  expect_equal(as.numeric(result1), as.numeric(t1))
})

test_that("time_epoch_posixct returns the Unix epoch", {
  result <- time_epoch_posixct()
  expect_s3_class(result, "POSIXct")
  expect_equal(as.numeric(result), 0)
})

test_that("time::Date roundtrips through R Date", {
  d <- as.Date("2024-06-04")
  result <- time_roundtrip_date(d)
  expect_s3_class(result, "Date")
  expect_equal(as.numeric(result), as.numeric(d))
})

test_that("time_epoch_date returns 1970-01-01", {
  result <- time_epoch_date()
  expect_s3_class(result, "Date")
  expect_equal(as.numeric(result), 0)
})

test_that("time_distant_past returns 1900-01-01", {
  result <- time_distant_past()
  expect_s3_class(result, "Date")
  expected <- as.Date("1900-01-01")
  expect_equal(as.numeric(result), as.numeric(expected))
})

test_that("time_get_year/month/day extract components correctly", {
  d <- as.Date("2024-06-15")
  expect_equal(time_get_year(d), 2024L)
  expect_equal(time_get_month(d), 6L)
  expect_equal(time_get_day(d), 15L)
})

test_that("time_format_date formats as YYYY-MM-DD", {
  d <- as.Date("2024-06-04")
  expect_equal(time_format_date(d), "2024-06-04")
})
