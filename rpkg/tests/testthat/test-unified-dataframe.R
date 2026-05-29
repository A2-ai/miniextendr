test_that("unified DataFrame verbs round-trip a data.frame", {
  df <- data.frame(
    x = c(1.0, 2.0, 3.0),
    y = c(2.0, 4.0, 6.0),
    label = c("a", "b", "c"),
    stringsAsFactors = FALSE
  )

  # FromDataFrame (read) → IntoDataFrame (write), composed in Rust.
  out <- unified_roundtrip(df)
  expect_s3_class(out, "data.frame")
  expect_equal(nrow(out), 3L)
  expect_equal(out$x, df$x)
  expect_equal(out$y, df$y)
  expect_equal(out$label, df$label)

  # FromDataFrame alone, returning the row count.
  expect_equal(unified_roundtrip_count(df), 3L)
})

test_that("unified verbs handle an empty data.frame", {
  df <- data.frame(
    x = numeric(0),
    y = numeric(0),
    label = character(0),
    stringsAsFactors = FALSE
  )
  expect_equal(unified_roundtrip_count(df), 0L)
  out <- unified_roundtrip(df)
  expect_s3_class(out, "data.frame")
  expect_equal(nrow(out), 0L)
})

test_that("gc_stress_unified_dataframe drives both verbs without error", {
  expect_no_error(gc_stress_unified_dataframe())
})
