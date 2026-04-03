# Tests for zero-copy conversions (Cow, Arrow pointer recovery, ProtectedStrVec)

# region: Cow<[T]> round-trip identity

test_that("Cow<[f64]> round-trip returns same R object (zero-copy)", {
  x <- c(1.0, 2.0, 3.0)
  expect_true(zero_copy_cow_f64_identity(x))
})

test_that("Cow<[i32]> round-trip returns same R object (zero-copy)", {
  # Use c() not 1:5 — the colon operator creates ALTREP compact sequences
  # whose data isn't at a fixed offset from the SEXP header.
  x <- c(1L, 2L, 3L, 4L, 5L)
  expect_true(zero_copy_cow_i32_identity(x))
})

test_that("Cow<[f64]> round-trip with NAs returns same object", {
  x <- c(1.0, NA, 3.0)
  expect_true(zero_copy_cow_f64_identity(x))
})

test_that("Cow<[i32]> round-trip with NAs returns same object", {
  x <- c(1L, NA, 3L)
  expect_true(zero_copy_cow_i32_identity(x))
})

# endregion

# region: Cow<str> scalar

test_that("Cow<str> from R is zero-copy (Borrowed)", {
  expect_true(zero_copy_cow_str_is_borrowed("hello"))
  expect_true(zero_copy_cow_str_is_borrowed(""))
  expect_true(zero_copy_cow_str_is_borrowed("unicode: \u00e9\u00e0\u00fc"))
})

# endregion

# region: Vec<Cow<str>>

test_that("Vec<Cow<str>> elements are all zero-copy (Borrowed)", {
  expect_true(zero_copy_vec_cow_str_all_borrowed(c("a", "b", "c")))
  expect_true(zero_copy_vec_cow_str_all_borrowed(c("hello", "world")))
  # NA maps to Cow::Borrowed("") in non-Option variant
  expect_true(zero_copy_vec_cow_str_all_borrowed(c("a", NA, "c")))
})

# endregion

# region: Arrow array identity (pointer recovery)

test_that("Float64Array round-trip returns same R object (zero-copy)", {
  x <- c(1.0, 2.0, 3.0)
  expect_true(zero_copy_arrow_f64_identity(x))
})

test_that("Float64Array with NAs round-trip returns same object", {
  x <- c(1.0, NA, 3.0)
  expect_true(zero_copy_arrow_f64_identity(x))
})

test_that("Int32Array round-trip returns same R object (zero-copy)", {
  x <- c(1L, 2L, 3L, 4L, 5L)
  expect_true(zero_copy_arrow_i32_identity(x))
})

test_that("Int32Array with NAs round-trip returns same object", {
  x <- c(1L, NA, 3L)
  expect_true(zero_copy_arrow_i32_identity(x))
})

test_that("UInt8Array round-trip returns same R object (zero-copy)", {
  x <- as.raw(c(1, 2, 3))
  expect_true(zero_copy_arrow_u8_identity(x))
})

test_that("Computed Arrow array is NOT the same object (different memory)", {
  x <- c(1.0, 2.0, 3.0)
  expect_false(zero_copy_arrow_f64_computed_is_different(x))
})

# endregion

# region: SEXPREC data offset

test_that("SEXPREC data offset was computed at init", {
  offset <- zero_copy_sexprec_offset()
  expect_true(offset > 0)
  # On 64-bit systems, sizeof(SEXPREC_ALIGN) is typically 48 or 56 bytes
  expect_true(offset >= 32 && offset <= 128)
})

# endregion

# region: ProtectedStrVec

test_that("ProtectedStrVec counts unique strings", {
  expect_equal(zero_copy_protected_strvec_unique(c("a", "b", "c")), 3L)
  expect_equal(zero_copy_protected_strvec_unique(c("a", "a", "b")), 2L)
})

test_that("ProtectedStrVec handles NA", {
  expect_equal(zero_copy_protected_strvec_unique(c("a", NA_character_, "b")), 2L)
  expect_equal(zero_copy_protected_strvec_unique(c(NA_character_, NA_character_)), 0L)
})

# endregion
