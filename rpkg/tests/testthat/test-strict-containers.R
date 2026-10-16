# Strict returns of slices, boxed slices and arrays (#1684). Fixture:
# src/rust/strict_container_tests.rs. Each fixture takes `big`: FALSE keeps
# every value in R's integer range, TRUE adds values outside it (3e9, or
# i32::MIN for the isize rows, since a 32-bit isize cannot hold 3e9).

strict_message <- function(expr) {
  e <- tryCatch(expr, error = identity)
  expect_s3_class(e, "rust_error")
  conditionMessage(e)
}

test_that("strict slices, boxes and arrays return integer vectors", {
  expect_identical(miniextendr:::strict_boxed_i64(FALSE), 1:3)
  expect_identical(miniextendr:::strict_slice_u64(FALSE), 1:2)
  expect_identical(miniextendr:::strict_array_usize(FALSE), 1:2)
  expect_identical(miniextendr:::strict_optional_boxed_i64(FALSE), 1:3)
  expect_null(miniextendr:::strict_optional_boxed_i64(NULL))
})

test_that("strict slices, boxes and arrays name the type and every failing element", {
  msg <- strict_message(miniextendr:::strict_boxed_i64(TRUE))
  expect_match(msg, "strict conversion failed for Box<[i64]>:", fixed = TRUE)
  expect_match(msg, "i64 value 3000000000 is outside R integer range", fixed = TRUE)
  expect_match(msg, "(element 2)", fixed = TRUE)
  expect_match(msg, "(element 3)", fixed = TRUE)

  msg <- strict_message(miniextendr:::strict_slice_u64(TRUE))
  expect_match(msg, "strict conversion failed for &'static [u64]:", fixed = TRUE)
  expect_match(msg, "u64 value 3000000000 exceeds R integer max (2147483647) (element 2)", fixed = TRUE)

  msg <- strict_message(miniextendr:::strict_array_usize(TRUE))
  expect_match(msg, "strict conversion failed for [usize; 2]:", fixed = TRUE)

  # Option<Box<[T]>> checks the boxed slice it holds.
  msg <- strict_message(miniextendr:::strict_optional_boxed_i64(TRUE))
  expect_match(msg, "strict conversion failed for Box<[i64]>:", fixed = TRUE)
})

test_that("strict rows held in boxes, arrays and slices return integer rows", {
  expect_identical(miniextendr:::strict_boxed_rows_i64(FALSE), list(1:3, 1:3))
  expect_identical(miniextendr:::strict_array_rows_isize(FALSE), list(1:2, 3:4))
  # A missing row is NULL; an empty row is an empty integer vector.
  expect_identical(miniextendr:::strict_optional_slice_rows_u64(FALSE), list(1:2, NULL, integer()))
})

test_that("strict rows report the row and the position inside it", {
  msg <- strict_message(miniextendr:::strict_boxed_rows_i64(TRUE))
  expect_match(msg, "strict conversion failed for Vec<Box<[i64]>>:", fixed = TRUE)
  expect_match(msg, "at inner position 2 (element 2)", fixed = TRUE)
  expect_match(msg, "at inner position 3 (element 2)", fixed = TRUE)

  msg <- strict_message(miniextendr:::strict_array_rows_isize(TRUE))
  expect_match(msg, "strict conversion failed for Vec<[isize; 2]>:", fixed = TRUE)
  expect_match(msg, "at inner position 2 (element 2)", fixed = TRUE)

  msg <- strict_message(miniextendr:::strict_optional_slice_rows_u64(TRUE))
  expect_match(msg, "strict conversion failed for Vec<Option<&'static [u64]>>:", fixed = TRUE)
  expect_match(msg, "at inner position 1 (element 3)", fixed = TRUE)
})

test_that("without strict the same shapes still widen to doubles", {
  big <- c(1, 3e9, -3e9)
  expect_identical(miniextendr:::lax_boxed_i64(FALSE), 1:3)
  expect_identical(miniextendr:::lax_boxed_i64(TRUE), big)
  expect_identical(miniextendr:::lax_slice_u64(TRUE), c(1, 3e9))
  expect_identical(miniextendr:::lax_array_usize(FALSE), 1:2)
  expect_identical(miniextendr:::lax_array_usize(TRUE), c(1, 3e9))
  expect_identical(miniextendr:::lax_optional_boxed_i64(TRUE), big)
  expect_null(miniextendr:::lax_optional_boxed_i64(NULL))
  expect_identical(miniextendr:::lax_boxed_rows_i64(TRUE), list(1:3, big))
  expect_identical(miniextendr:::lax_array_rows_isize(TRUE), list(1:2, c(3, -2147483648)))
})
