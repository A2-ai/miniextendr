test_that("Box<[f64]> roundtrip works", {
  expect_equal(box_slice_f64_roundtrip(c(1.5, 2.5, 3.5)), c(1.5, 2.5, 3.5))
  expect_equal(box_slice_f64_roundtrip(numeric(0)), numeric(0))
})

test_that("Box<[i32]> roundtrip works", {
  expect_equal(box_slice_i32_roundtrip(1:5), 1:5)
  expect_equal(box_slice_i32_roundtrip(integer(0)), integer(0))
})

test_that("Box<[String]> roundtrip works", {
  expect_equal(box_slice_string_roundtrip(c("a", "b", "c")), c("a", "b", "c"))
  expect_equal(box_slice_string_roundtrip(character(0)), character(0))
})

test_that("Box<[bool]> roundtrip works", {
  expect_equal(box_slice_bool_roundtrip(c(TRUE, FALSE, TRUE)), c(TRUE, FALSE, TRUE))
  expect_equal(box_slice_bool_roundtrip(logical(0)), logical(0))
})

test_that("Box<[u8]> roundtrip works", {
  expect_equal(box_slice_raw_roundtrip(as.raw(1:5)), as.raw(1:5))
  expect_equal(box_slice_raw_roundtrip(raw(0)), raw(0))
})

test_that("Box<[f64]> transform works", {
  expect_equal(box_slice_double(c(1, 2, 3)), c(2, 4, 6))
})

test_that("Box<[Option<f64>]> with NA works", {
  result <- box_slice_option_f64_roundtrip(c(1.5, NA, 3.5))
  expect_equal(result, c(1.5, NA, 3.5))
})

test_that("Box<[Option<i32>]> with NA works", {
  result <- box_slice_option_i32_roundtrip(c(1L, NA, 3L))
  expect_equal(result, c(1L, NA, 3L))
})

test_that("Box<[Option<String>]> with NA works", {
  result <- box_slice_option_string_roundtrip(c("a", NA, "c"))
  expect_equal(result, c("a", NA, "c"))
})
