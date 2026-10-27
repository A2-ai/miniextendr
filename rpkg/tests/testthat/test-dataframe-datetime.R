# Datetime fields in `#[derive(DataFrameRow)]` rows: jiff and time types
# spelled by crate path are datetime columns, `Option<T>` NA-able ones
# (features = "jiff", "time").

test_that("DataFrameRow datetime columns get R datetime classes", {
  df <- miniextendr:::df_datetime_rows()
  expect_s3_class(df, "data.frame")
  expect_equal(
    names(df),
    c("id", "at", "seen", "zoned", "day", "lag", "stamp", "due")
  )

  for (col in c("at", "seen", "zoned", "stamp")) {
    expect_s3_class(df[[col]], "POSIXct")
  }
  expect_identical(attr(df$at, "tzone"), "UTC")
  expect_identical(attr(df$stamp, "tzone"), "UTC")
  expect_identical(attr(df$zoned, "tzone"), "Europe/Oslo")
  expect_equal(as.numeric(df$at), c(0, 60))
  expect_equal(as.numeric(df$seen), c(86400, NA))
  expect_equal(as.numeric(df$zoned), c(0, NA))
  expect_equal(as.numeric(df$stamp), c(3600, NA))

  expect_s3_class(df$day, "Date")
  expect_s3_class(df$due, "Date")
  expect_equal(df$day, as.Date(c("2024-02-29", NA)))
  expect_equal(df$due, as.Date(c("2024-03-01", NA)))

  expect_s3_class(df$lag, "difftime")
  expect_identical(units(df$lag), "secs")
  expect_equal(as.numeric(df$lag), c(90, NA))
})

test_that("DataFrameRow datetime columns read back", {
  df <- miniextendr:::df_datetime_rows()
  expect_equal(miniextendr:::df_datetime_roundtrip(df), df)
})

test_that("enum variants spelling one datetime type two ways share a column", {
  df <- miniextendr:::df_datetime_enum_rows()
  expect_equal(names(df), c("kind", "at", "day"))
  expect_s3_class(df$at, "POSIXct")
  expect_identical(attr(df$at, "tzone"), "UTC")
  expect_equal(as.numeric(df$at), c(0, 7200))
  expect_s3_class(df$day, "Date")
  expect_equal(df$day, as.Date(c(NA, "1970-01-02")))
  expect_equal(miniextendr:::df_datetime_enum_roundtrip(df), df)
})
