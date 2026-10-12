# What `#[derive(DataFrameRow)]` asks of field types: only what turning a field
# into columns needs (rpkg/src/rust/dataframe_bounds_test.rs).

test_that("a skipped field without Default still produces a data.frame", {
  df <- bounds_skip_no_default()
  expect_s3_class(df, "data.frame")
  expect_identical(colnames(df), c("id", "label"))
  expect_equal(df$id, c(1L, 2L))
  expect_equal(df$label, c("a", "b"))
})

test_that("a row that is not Send builds through the sequential fill under rayon", {
  skip_if_not(exists("bounds_skip_no_default_par", mode = "function"),
              "bounds_skip_no_default_par not available (rayon feature not compiled)")
  expect_identical(bounds_skip_no_default_par(), bounds_skip_no_default())
})

test_that("expanded fields move elements that are neither Clone nor Default", {
  df <- bounds_moved_elems()
  expect_identical(
    colnames(df),
    c("id", "pair_1", "pair_2", "firsts_1", "firsts_2", "firsts_3", "rest_1", "rest_2")
  )
  expect_equal(df$pair_1, c(1, 3))
  expect_equal(df$pair_2, c(2, 4))
  expect_equal(df$firsts_1, c(10, 20))
  expect_equal(df$firsts_2, c(NA, 21))
  expect_equal(df$firsts_3, c(NA, 22))
  expect_equal(df$rest_1, c(100, 300))
  expect_equal(df$rest_2, c(200, NA))
})

test_that("the parallel fill of moved elements matches the sequential one", {
  skip_if_not(exists("bounds_moved_elems_par", mode = "function"),
              "bounds_moved_elems_par not available (rayon feature not compiled)")
  expect_identical(bounds_moved_elems_par(), bounds_moved_elems())
})

test_that("enum expanded fields move elements that are neither Clone nor Default", {
  df <- bounds_moved_event()
  expect_identical(colnames(df), c("kind", "id", "pair_1", "pair_2", "vals_1", "vals_2"))
  expect_equal(df$kind, c("Pair", "Spread"))
  expect_equal(df$id, c(1L, 2L))
  expect_equal(df$pair_1, c(1, NA))
  expect_equal(df$pair_2, c(2, NA))
  expect_equal(df$vals_1, c(NA, 5))
  expect_equal(df$vals_2, c(NA_real_, NA_real_))
})

test_that("the parallel fill of enum moved elements matches the sequential one", {
  skip_if_not(exists("bounds_moved_event_par", mode = "function"),
              "bounds_moved_event_par not available (rayon feature not compiled)")
  expect_identical(bounds_moved_event_par(), bounds_moved_event())
})

test_that("companions are Debug and Clone when their columns are", {
  expect_identical(
    bounds_companion_debug_clone(),
    'DebugPointDataFrame { x: [1.5, 2.5], label: ["a", "b"] }'
  )
  expect_identical(
    bounds_enum_companion_debug_clone(),
    'DebugEventDataFrame { _tag: ["A", "B"], x: [Some(1), None], y: [None, Some("b")] }'
  )
})
