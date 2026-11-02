# Borrowed rows -> data.frame (#2307): a `&self` method converts the rows it
# holds from `&[Row]` / `&Vec<Row>` without cloning the rows, and builds
# exactly the frame the owned `Vec<Row>` conversion builds.

cases <- c("full", "na", "empty")

test_that("borrowed struct rows build the owned conversion's frame", {
  for (case in cases) {
    store <- BorrowedRowsStore$new(case)
    owned <- df_borrowed_obs_owned(case)
    expect_identical(store$obs_slice(), owned, info = case)
    expect_identical(store$obs_vec(), owned, info = case)
  }
})

test_that("borrowed enum rows build the owned conversion's frame", {
  for (case in cases) {
    store <- BorrowedRowsStore$new(case)
    owned <- df_borrowed_events_owned(case)
    expect_identical(store$events_slice(), owned, info = case)
    expect_identical(store$events_vec(), owned, info = case)
  }
})

test_that("borrowed rows stay in the store and convert again", {
  store <- BorrowedRowsStore$new("full")
  first <- store$obs_slice()
  expect_identical(store$n_obs(), 3L)
  expect_identical(store$obs_slice(), first)
  expect_identical(store$obs_vec(), first)
})

test_that("String and Option<String> columns are written from &str", {
  df <- BorrowedRowsStore$new("full")$obs_slice()
  expect_identical(df$kind, rep("BorrowedObs", 3L))
  expect_identical(df$label, c("one", "", "three"))
  expect_identical(df$note, c("first", NA, ""))
  expect_identical(df$inner_tag, c("in1", "in2", "in3"))
  expect_identical(df$words_1, c("a", "b", NA))
  expect_identical(df$words_2, c(NA, "c", NA))
  expect_identical(df$weight, c(1.5, -2, NA))
  expect_identical(df$flag, c(TRUE, NA, FALSE))
  expect_identical(df$extra_1, c(1L, NA, 4L))
  expect_identical(levels(df$level), c("Low", "Mid", "High"))
  expect_identical(as.character(df$level), c("High", "Mid", "Low"))
  expect_identical(df$counts[[1]], list(x = 1L, y = 2L))
  expect_identical(df$listed[[3]], list(p = 1.5))
})

test_that("all-NA columns keep their types", {
  df <- BorrowedRowsStore$new("na")$obs_vec()
  expect_identical(df$note, c(NA_character_, NA_character_))
  expect_identical(df$weight, c(NA_real_, NA_real_))
  expect_identical(df$flag, c(NA, NA))
  expect_identical(df$scores_1, c(NA_real_, NA_real_))

  events <- BorrowedRowsStore$new("na")$events_vec()
  expect_identical(events$variant, c("Empty", "Empty"))
  expect_identical(events$id, c(NA_integer_, NA_integer_))
  expect_identical(events$label, c(NA_character_, NA_character_))
  expect_identical(events$coords_1, c(NA_real_, NA_real_))
  expect_identical(events$inner_q, c(NA_integer_, NA_integer_))
  expect_s3_class(events$level, "factor")
  expect_identical(levels(events$level), c("Low", "Mid", "High"))
})

test_that("an empty store builds a 0-row frame with typed columns", {
  store <- BorrowedRowsStore$new("empty")
  df <- store$obs_slice()
  expect_s3_class(df, "data.frame")
  expect_identical(nrow(df), 0L)
  expect_identical(df$label, character(0))
  expect_identical(df$weight, numeric(0))
  expect_identical(df$flag, logical(0))
  expect_identical(levels(df$level), c("Low", "Mid", "High"))

  events <- store$events_slice()
  expect_identical(nrow(events), 0L)
  expect_identical(events$variant, character(0))
  expect_identical(events$id, integer(0))
})

test_that("borrowed enum rows: payload, expansion, flatten and factor columns", {
  df <- BorrowedRowsStore$new("full")$events_slice()
  expect_identical(df$variant, c("Point", "Tagged", "Nested", "Empty", "Tagged"))
  expect_identical(df$id, c(1L, 2L, NA, NA, 5L))
  expect_identical(df$label, c("p", NA, NA, NA, NA))
  expect_identical(df$tags_1, c(NA, "t1", NA, NA, NA))
  expect_identical(df$inner_tag, c(NA, NA, "nested", NA, NA))
  expect_identical(as.character(df$level), c(NA, "High", NA, NA, "Low"))
  expect_identical(df$counts_keys[[3]], "k")
  expect_identical(df$counts_values[[3]], 3L)
})

test_that("gc_stress_dataframe_borrowed_rows survives gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  cells <- miniextendr:::gc_stress_dataframe_borrowed_rows()
  gctorture(FALSE)
  expect_identical(cells, 340L)
})
