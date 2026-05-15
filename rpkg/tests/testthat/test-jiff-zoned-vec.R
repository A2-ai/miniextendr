# Tests for JiffZonedVec ALTREP (feature = "jiff") — single-timezone strict.

test_that("JiffZonedVec accepts a single-timezone Vec<Zoned>", {
  v <- jiff_zoned_vec_new(c(
    "2025-01-01T00:00:00[America/New_York]",
    "2025-06-15T12:30:00[America/New_York]"
  ))
  expect_s3_class(v, "POSIXct")
  expect_identical(attr(v, "tzone"), "America/New_York")
  expect_identical(length(v), 2L)
})

test_that("JiffZonedVec rejects heterogeneous timezones", {
  expect_error(
    jiff_zoned_vec_new(c(
      "2025-01-01T00:00:00[America/New_York]",
      "2025-01-01T05:00:00[Europe/London]"
    )),
    "timezone"
  )
})

test_that("JiffZonedVec with a single element has correct tzone", {
  v <- jiff_zoned_vec_new("2025-03-15T10:00:00[Asia/Tokyo]")
  expect_s3_class(v, "POSIXct")
  expect_identical(attr(v, "tzone"), "Asia/Tokyo")
  expect_identical(length(v), 1L)
})

test_that("JiffZonedVec with UTC timezone", {
  v <- jiff_zoned_vec_new(c(
    "2025-01-01T00:00:00[UTC]",
    "2025-07-04T12:00:00[UTC]"
  ))
  expect_s3_class(v, "POSIXct")
  expect_identical(attr(v, "tzone"), "UTC")
  expect_identical(length(v), 2L)
})

test_that("JiffZonedVec element roundtrips through first_element fixture", {
  v <- jiff_zoned_vec_new("2025-03-15T10:00:00[Asia/Tokyo]")
  back <- jiff_zoned_vec_first_element(v)
  # Should round-trip to the same RFC 9557 representation
  expect_match(back, "2025-03-15T10:00:00")
  expect_match(back, "Asia/Tokyo")
})

test_that("JiffZonedVec element values are correct UTC epoch seconds", {
  # 2025-01-01T00:00:00Z is 1735689600 seconds after Unix epoch
  v <- jiff_zoned_vec_new("2025-01-01T00:00:00[UTC]")
  expect_equal(as.numeric(v), 1735689600, tolerance = 1e-6)
})
