# Test jiff datetime integration (feature = "jiff")

# region: Timestamp (POSIXct UTC)

test_that("jiff: Timestamp roundtrips through POSIXct (UTC)", {
  ts <- as.POSIXct("2024-01-15 12:30:00", tz = "UTC")
  result <- jiff_roundtrip_timestamp(ts)
  expect_s3_class(result, "POSIXct")
  expect_equal(as.numeric(result), as.numeric(ts), tolerance = 1e-6)
})

test_that("jiff: Timestamp roundtrips epoch correctly", {
  epoch <- jiff_epoch_timestamp()
  expect_s3_class(epoch, "POSIXct")
  expect_equal(as.numeric(epoch), 0.0)
})

test_that("jiff: negative Timestamp (pre-1970) roundtrips correctly", {
  neg <- jiff_negative_timestamp()
  expect_s3_class(neg, "POSIXct")
  expect_true(as.numeric(neg) < 0)
})

test_that("jiff: fractional-second Timestamp roundtrips with subsecond precision", {
  frac <- jiff_fractional_timestamp()
  expect_s3_class(frac, "POSIXct")
  # Should be 1.5 seconds after epoch
  expect_equal(as.numeric(frac), 1.5, tolerance = 1e-6)
})

test_that("jiff: -0.5 second Timestamp roundtrips correctly (floor-based split)", {
  half_before <- jiff_half_second_before_epoch()
  expect_s3_class(half_before, "POSIXct")
  # Should be -0.5 seconds (floor(-0.5) = -1, subsec = 0.5)
  expect_equal(as.numeric(half_before), -0.5, tolerance = 1e-6)
})

test_that("jiff: Vec<Timestamp> roundtrips as POSIXct vector", {
  now <- Sys.time()
  ts_vec <- c(now, now + 3600, now - 3600)
  attr(ts_vec, "tzone") <- "UTC"
  result <- jiff_roundtrip_timestamp_vec(ts_vec)
  expect_s3_class(result, "POSIXct")
  expect_length(result, 3L)
  expect_equal(as.numeric(result), as.numeric(ts_vec), tolerance = 1e-6)
})

test_that("jiff: Option<Timestamp> maps NA to NA POSIXct and roundtrips Some", {
  # NULL / NA_real_ → None → NA POSIXct (not NULL)
  na_result <- jiff_option_timestamp(as.POSIXct(NA))
  expect_s3_class(na_result, "POSIXct")
  expect_true(is.na(as.numeric(na_result)))

  ts <- as.POSIXct("2024-06-01", tz = "UTC")
  some_result <- jiff_option_timestamp(ts)
  expect_s3_class(some_result, "POSIXct")
  expect_equal(as.numeric(some_result), as.numeric(ts), tolerance = 1e-6)
})

test_that("jiff: jiff_timestamp_seconds extracts correct value", {
  ts <- as.POSIXct("1970-01-01 00:00:01.5", tz = "UTC")
  secs <- jiff_timestamp_seconds(ts)
  expect_equal(secs, 1.5, tolerance = 1e-6)
})

# endregion

# region: Zoned (POSIXct + tzone)

test_that("jiff: Zoned roundtrips America/New_York timezone", {
  skip_if_not(exists("jiff_roundtrip_zoned"), "jiff_roundtrip_zoned not available")
  ts <- as.POSIXct("2024-06-15 10:00:00", tz = "America/New_York")
  result <- jiff_roundtrip_zoned(ts)
  expect_s3_class(result, "POSIXct")
  expect_equal(as.numeric(result), as.numeric(ts), tolerance = 1e-6)
  expect_equal(attr(result, "tzone"), "America/New_York")
})

test_that("jiff: Zoned roundtrips UTC", {
  ts <- as.POSIXct("2024-01-01 00:00:00", tz = "UTC")
  result <- jiff_roundtrip_zoned(ts)
  expect_s3_class(result, "POSIXct")
  expect_equal(as.numeric(result), as.numeric(ts), tolerance = 1e-6)
})

test_that("jiff: jiff_zoned_tz_name extracts timezone name", {
  ts <- as.POSIXct("2024-06-15 10:00:00", tz = "Europe/London")
  tz_name <- jiff_zoned_tz_name(ts)
  expect_equal(tz_name, "Europe/London")
})

test_that("jiff: jiff_zoned_year extracts year", {
  ts <- as.POSIXct("2024-06-15 10:00:00", tz = "UTC")
  yr <- jiff_zoned_year(ts)
  expect_equal(yr, 2024L)
})

test_that("jiff: Vec<Zoned> roundtrips as POSIXct vector", {
  ts1 <- as.POSIXct("2024-01-01 00:00:00", tz = "UTC")
  ts2 <- as.POSIXct("2024-06-15 12:00:00", tz = "UTC")
  ts_vec <- c(ts1, ts2)
  result <- jiff_roundtrip_zoned_vec(ts_vec)
  expect_s3_class(result, "POSIXct")
  expect_length(result, 2L)
  expect_equal(as.numeric(result), as.numeric(ts_vec), tolerance = 1e-6)
})

# endregion

# region: civil::Date (R Date)

test_that("jiff: civil::Date roundtrips through R Date", {
  d <- as.Date("2024-03-20")
  result <- jiff_roundtrip_date(d)
  expect_s3_class(result, "Date")
  expect_equal(as.numeric(result), as.numeric(d))
})

test_that("jiff: civil::Date vector roundtrips correctly", {
  dates <- as.Date(c("2024-01-01", "2000-06-15", "1970-01-01"))
  result <- jiff_roundtrip_date_vec(dates)
  expect_s3_class(result, "Date")
  expect_equal(as.numeric(result), as.numeric(dates))
})

test_that("jiff: epoch Date (1970-01-01) roundtrips", {
  epoch <- jiff_epoch_date()
  expect_s3_class(epoch, "Date")
  expect_equal(as.numeric(epoch), 0.0)
})

test_that("jiff: distant past Date (1900-03-01) roundtrips", {
  past <- jiff_distant_past_date()
  expect_s3_class(past, "Date")
  expect_true(as.numeric(past) < -25000)
})

test_that("jiff: jiff_date_year/month/day extract components", {
  d <- as.Date("2024-07-04")
  expect_equal(jiff_date_year(d), 2024L)
  expect_equal(jiff_date_month(d), 7L)
  expect_equal(jiff_date_day(d), 4L)
})

# endregion

# region: SignedDuration (difftime)

test_that("jiff: SignedDuration roundtrips 0", {
  dur <- structure(0, class = "difftime", units = "secs")
  result <- jiff_roundtrip_duration(dur)
  expect_s3_class(result, "difftime")
  expect_equal(as.numeric(result, units = "secs"), 0.0, tolerance = 1e-9)
})

test_that("jiff: jiff_one_hour_duration returns 3600s difftime", {
  dur <- jiff_one_hour_duration()
  expect_s3_class(dur, "difftime")
  expect_equal(as.numeric(dur, units = "secs"), 3600.0, tolerance = 1e-9)
})

test_that("jiff: jiff_negative_duration returns -0.5s difftime", {
  dur <- jiff_negative_duration()
  expect_s3_class(dur, "difftime")
  expect_equal(as.numeric(dur, units = "secs"), -0.5, tolerance = 1e-9)
})

test_that("jiff: jiff_duration_secs extracts seconds correctly", {
  dur <- structure(42.75, class = "difftime", units = "secs")
  secs <- jiff_duration_secs(dur)
  expect_equal(secs, 42.75, tolerance = 1e-9)
})

# endregion

# region: ALTREP (JiffTimestampVec)

test_that("jiff: ALTREP JiffTimestampVec has correct length", {
  altrep <- jiff_altrep_timestamps(10L)
  expect_length(altrep, 10L)
  expect_equal(jiff_altrep_len(altrep), 10L)
})

test_that("jiff: ALTREP JiffTimestampVec elements are correct", {
  altrep <- jiff_altrep_timestamps(5L)
  # Elements should be 0, 1, 2, 3, 4 seconds after epoch
  for (i in 0:4) {
    elt <- jiff_altrep_elt(altrep, i)
    expect_equal(elt, as.numeric(i), tolerance = 1e-9)
  }
})

test_that("jiff: ALTREP JiffTimestampVec is POSIXct class", {
  altrep <- jiff_altrep_timestamps(3L)
  expect_s3_class(altrep, "POSIXct")
})

test_that("jiff: empty ALTREP JiffTimestampVec has length 0", {
  altrep <- jiff_altrep_timestamps(0L)
  expect_length(altrep, 0L)
})

# endregion
