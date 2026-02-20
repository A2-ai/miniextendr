# Comprehensive conversions tests for #[miniextendr] args and returns

test_that("all scalar type round-trips work", {
  expect_equal(conv_i32_arg(1L), 1L)
  expect_equal(conv_i32_ret(), 1L)

  expect_equal(conv_f64_arg(1.25), 1.25)
  expect_equal(conv_f64_ret(), 1.25)

  expect_equal(conv_u8_arg(as.raw(7)), as.raw(7))
  expect_equal(conv_u8_ret(), as.raw(7))

  expect_true(conv_rbool_arg(TRUE))
  expect_true(conv_rbool_ret())

  expect_true(conv_rlog_arg(TRUE))
  expect_true(conv_rlog_ret())

  expect_equal(conv_string_arg("hi"), "hi")
  expect_equal(conv_string_ret(), "hi")

  expect_equal(conv_str_ret(), "hi")

  expect_equal(conv_sexp_arg(1L), 1L)
  expect_equal(conv_sexp_ret(), 1L)

  expect_equal(conv_i64_arg(1), 1)
  expect_equal(conv_i64_ret(), 1)

  expect_equal(conv_u64_arg(1), 1)
  expect_equal(conv_u64_ret(), 1)

  expect_equal(conv_isize_arg(1L), 1)
  expect_equal(conv_isize_ret(), 1)

  expect_equal(conv_usize_arg(1L), 1)
  expect_equal(conv_usize_ret(), 1)

  expect_equal(conv_i8_arg(1L), 1L)
  expect_equal(conv_i8_arg(1), 1L)
  expect_equal(conv_i8_arg(as.raw(1)), 1L)
  expect_equal(conv_i8_ret(), 1L)

  expect_equal(conv_i16_arg(1L), 1L)
  expect_equal(conv_i16_ret(), 1L)

  expect_equal(conv_u16_arg(1L), 1L)
  expect_equal(conv_u16_ret(), 1L)

  expect_equal(conv_u32_arg(1L), 1)
  expect_equal(conv_u32_ret(), 1)

  expect_equal(conv_f32_arg(1.5), 1.5)
  expect_equal(conv_f32_ret(), 1.5)
})

test_that("NULL and NA map to None for optional scalar arguments", {
  expect_equal(conv_opt_i32_is_some(NULL), 0L)
  expect_equal(conv_opt_i32_is_some(NA_integer_), 0L)
  expect_equal(conv_opt_i32_is_some(5L), 1L)
  expect_equal(conv_opt_i32_some(), 10L)
  expect_true(is.na(conv_opt_i32_none()))

  expect_equal(conv_opt_f64_is_some(NULL), 0L)
  expect_equal(conv_opt_f64_is_some(NA_real_), 0L)
  expect_equal(conv_opt_f64_is_some(2.5), 1L)
  expect_equal(conv_opt_f64_some(), 2.5)
  expect_true(is.na(conv_opt_f64_none()))

  expect_equal(conv_opt_bool_is_some(NULL), 0L)
  expect_equal(conv_opt_bool_is_some(NA), 0L)
  expect_equal(conv_opt_bool_is_some(TRUE), 1L)
  expect_true(conv_opt_bool_some())
  expect_true(is.na(conv_opt_bool_none()))

  expect_equal(conv_opt_string_is_some(NULL), 0L)
  expect_equal(conv_opt_string_is_some(NA_character_), 0L)
  expect_equal(conv_opt_string_is_some("opt"), 1L)
  expect_equal(conv_opt_string_some(), "opt")
  expect_true(is.na(conv_opt_string_none()))
})

test_that("optional scalars accept coerced types (i8, i16, u16, f32, etc.)", {
  expect_equal(conv_opt_i8_is_some(1L), 1L)
  expect_equal(conv_opt_i8_is_some(1.0), 1L)
  expect_equal(conv_opt_i8_is_some(as.raw(1)), 1L)
  expect_equal(conv_opt_i8_is_some(NA_integer_), 0L)
  expect_equal(conv_opt_i16_is_some(1L), 1L)
  expect_equal(conv_opt_i64_is_some(1L), 1L)
  expect_equal(conv_opt_isize_is_some(1L), 1L)
  expect_equal(conv_opt_u16_is_some(1L), 1L)
  expect_equal(conv_opt_u32_is_some(1L), 1L)
  expect_equal(conv_opt_u64_is_some(1L), 1L)
  expect_equal(conv_opt_usize_is_some(1L), 1L)
  expect_equal(conv_opt_f32_is_some(1.5), 1L)

  expect_equal(conv_opt_u8_is_some(as.raw(1)), 1L)
  expect_equal(conv_opt_rbool_is_some(TRUE), 1L)
  expect_equal(conv_opt_rbool_is_some(NA), 0L)
  expect_equal(conv_opt_rlog_is_some(TRUE), 1L)
  expect_equal(conv_opt_rlog_is_some(NA), 0L)
})

test_that("conversions slices work", {
  expect_equal(conv_slice_i32_len(1:3), 3L)
  expect_equal(conv_slice_f64_len(c(1.0, 2.0)), 2L)
  expect_equal(conv_slice_u8_len(as.raw(1:3)), 3L)
  expect_equal(conv_slice_rlog_len(c(TRUE, FALSE, TRUE)), 3L)
})

test_that("conversions references work", {
  expect_equal(conv_ref_i32_arg(1L), 1L)
  expect_equal(conv_ref_mut_i32_add_one(1L), 2L)

  x <- 1:3
  conv_slice_mut_i32_add_one(x)
  expect_equal(x, 2:4)

  r <- as.raw(c(1, 2))
  conv_slice_mut_u8_add_one(r)
  expect_equal(r, as.raw(c(2, 3)))

  expect_equal(conv_opt_ref_i32_is_some(NULL), 0L)
  expect_equal(conv_opt_ref_i32_is_some(1L), 1L)
  expect_equal(conv_opt_mut_slice_i32_is_some(NULL), 0L)
  expect_equal(conv_opt_mut_slice_i32_is_some(1:2), 1L)

  expect_equal(conv_vec_ref_i32_len(list(1L, 2L, 3L)), 3L)
  expect_equal(conv_vec_slice_i32_total_len(list(1:3, 4:5)), 5L)

  lst <- list(1:2, 3:4)
  conv_vec_mut_slice_i32_add_one(lst)
  expect_equal(lst[[1]], 2:3)
  expect_equal(lst[[2]], 4:5)
})

test_that("conversions vec and vec-option work", {
  expect_equal(conv_vec_i32_len(1:3), 3L)
  expect_equal(conv_vec_i32_ret(), 1:3)

  expect_equal(conv_vec_f64_len(c(1.0, 2.0)), 2L)
  expect_equal(conv_vec_f64_ret(), c(1.0, 2.0, 3.0))

  expect_equal(conv_vec_u8_len(as.raw(1:3)), 3L)
  expect_equal(conv_vec_u8_ret(), as.raw(c(1, 2, 3)))

  expect_equal(conv_vec_rlog_len(c(TRUE, FALSE)), 2L)
  expect_equal(conv_vec_rlog_ret(), c(TRUE, FALSE))

  expect_equal(conv_vec_bool_len(c(TRUE, FALSE, TRUE)), 3L)
  expect_equal(conv_vec_bool_ret(), c(TRUE, FALSE, TRUE))

  expect_equal(conv_vec_string_len(c("a", "b")), 2L)
  expect_equal(conv_vec_string_ret(), c("a", "b"))

  expect_equal(conv_vec_i8_len(1:3), 3L)
  expect_equal(conv_vec_i8_len(c(1, 2, 3)), 3L)
  expect_equal(conv_vec_i8_len(as.raw(c(1, 2, 3))), 3L)
  expect_equal(conv_vec_i16_len(1:3), 3L)
  expect_equal(conv_vec_i64_len(1:3), 3L)
  expect_equal(conv_vec_isize_len(1:3), 3L)
  expect_equal(conv_vec_u16_len(1:3), 3L)
  expect_equal(conv_vec_u32_len(1:3), 3L)
  expect_equal(conv_vec_u64_len(1:3), 3L)
  expect_equal(conv_vec_usize_len(1:3), 3L)
  expect_equal(conv_vec_f32_len(c(1.5, 2.5)), 2L)

  v_i32 <- conv_vec_opt_i32_ret()
  expect_equal(length(v_i32), 3L)
  expect_true(is.na(v_i32[2]))

  v_f64 <- conv_vec_opt_f64_ret()
  expect_equal(length(v_f64), 3L)
  expect_true(is.na(v_f64[2]))

  v_bool <- conv_vec_opt_bool_ret()
  expect_equal(length(v_bool), 3L)
  expect_true(is.na(v_bool[2]))

  v_str <- conv_vec_opt_string_ret()
  expect_equal(length(v_str), 3L)
  expect_true(is.na(v_str[2]))

  v_rlog <- conv_vec_opt_rlog_ret()
  expect_equal(length(v_rlog), 3L)
  expect_true(is.na(v_rlog[2]))

  v_rbool <- conv_vec_opt_rbool_ret()
  expect_equal(length(v_rbool), 3L)
  expect_true(is.na(v_rbool[2]))

  expect_equal(conv_vec_opt_i8_len(c(1L, NA_integer_, 2L)), 3L)
  expect_equal(conv_vec_opt_i8_len(c(1.0, NA_real_, 2.0)), 3L)
  expect_equal(conv_vec_opt_i8_len(as.raw(c(1, 2, 3))), 3L)
  expect_equal(conv_vec_opt_u8_len(as.raw(c(1, 2, 3))), 3L)
})

test_that("Vec<Option<T>> length includes NA elements", {
  expect_equal(conv_vec_opt_i32_len(c(4L, NA_integer_, 6L)), 3L)
  expect_equal(conv_vec_opt_f64_len(c(1.5, NA_real_, 2.5)), 3L)
  expect_equal(conv_vec_opt_bool_len(c(FALSE, NA, TRUE)), 3L)
  expect_equal(conv_vec_opt_string_len(c("a", NA_character_, "b")), 3L)
})

test_that("conversions sets work", {
  expect_equal(conv_hashset_i32_len(c(1L, 1L, 2L)), 2L)
  expect_true(setequal(conv_hashset_i32_ret(), c(1L, 2L, 3L)))

  expect_equal(conv_hashset_u8_len(as.raw(c(1, 1, 2))), 2L)
  expect_true(setequal(conv_hashset_u8_ret(), as.raw(c(1, 2, 3))))

  expect_equal(conv_hashset_string_len(c("a", "a", "b")), 2L)
  expect_true(setequal(conv_hashset_string_ret(), c("a", "b")))

  expect_equal(conv_hashset_rlog_len(c(TRUE, TRUE, FALSE)), 2L)
  expect_true(setequal(conv_hashset_rlog_ret(), c(TRUE, FALSE)))

  expect_equal(conv_btreeset_i32_len(c(1L, 1L, 2L)), 2L)
  expect_true(setequal(conv_btreeset_i32_ret(), c(1L, 2L, 3L)))

  expect_equal(conv_btreeset_u8_len(as.raw(c(1, 1, 2))), 2L)
  expect_true(setequal(conv_btreeset_u8_ret(), as.raw(c(1, 2, 3))))

  expect_equal(conv_btreeset_string_len(c("a", "a", "b")), 2L)
  expect_true(setequal(conv_btreeset_string_ret(), c("a", "b")))
})

test_that("conversions maps work", {
  expect_equal(conv_hashmap_i32_len(list(a = 1L, b = 2L)), 2L)
  res_i32 <- conv_hashmap_i32_ret()
  expect_equal(res_i32$a, 1L)
  expect_equal(res_i32$b, 2L)

  expect_equal(conv_hashmap_f64_len(list(a = 1.5, b = 2.5)), 2L)
  res_f64 <- conv_hashmap_f64_ret()
  expect_equal(res_f64$a, 1.5)
  expect_equal(res_f64$b, 2.5)

  expect_equal(conv_hashmap_string_len(list(a = "x", b = "y")), 2L)
  res_str <- conv_hashmap_string_ret()
  expect_equal(res_str$a, "x")
  expect_equal(res_str$b, "y")

  expect_equal(conv_hashmap_rlog_len(list(a = TRUE, b = FALSE)), 2L)
  res_rlog <- conv_hashmap_rlog_ret()
  expect_true(res_rlog$a)
  expect_false(res_rlog$b)

  expect_equal(conv_btreemap_i32_len(list(a = 1L, b = 2L)), 2L)
  res_bi32 <- conv_btreemap_i32_ret()
  expect_equal(res_bi32$a, 1L)
  expect_equal(res_bi32$b, 2L)

  expect_equal(conv_btreemap_f64_len(list(a = 1.5, b = 2.5)), 2L)
  res_bf64 <- conv_btreemap_f64_ret()
  expect_equal(res_bf64$a, 1.5)
  expect_equal(res_bf64$b, 2.5)

  expect_equal(conv_btreemap_string_len(list(a = "x", b = "y")), 2L)
  res_bstr <- conv_btreemap_string_ret()
  expect_equal(res_bstr$a, "x")
  expect_equal(res_bstr$b, "y")

  expect_equal(conv_btreemap_rlog_len(list(a = TRUE, b = FALSE)), 2L)
  res_blog <- conv_btreemap_rlog_ret()
  expect_true(res_blog$a)
  expect_false(res_blog$b)
})

test_that("in-place list modification works", {
  x <- list(1L, 2L)
  conv_list_mut_set_first(x)
  expect_equal(x[[1]], 99L)
})

test_that("Result conversions work for args and returns", {
  expect_equal(conv_result_i32_arg(1L), 1L)
  expect_equal(conv_result_i32_arg(NULL), 0L)
  expect_equal(conv_result_vec_i32_arg(1:2), 1L)
  expect_equal(conv_result_vec_i32_arg(NULL), 0L)

  expect_equal(conv_result_i32_ok(), 9L)
  expect_true(is.null(conv_result_i32_err()))

  expect_equal(conv_result_f64_ok(), 9.5)
  expect_true(is.null(conv_result_f64_err()))

  expect_equal(conv_result_string_ok(), "ok")
  expect_true(is.null(conv_result_string_err()))

  expect_equal(conv_result_vec_i32_ok(), c(1L, 2L))
  expect_true(is.null(conv_result_vec_i32_err()))
})

# =============================================================================
# Extended conversions tests
# =============================================================================

test_that("char conversions work", {
  # char arg: single-character string to Rust char
  expect_equal(conv_char_arg("a"), "a")
  expect_equal(conv_char_arg("α"), "α")  # Unicode

  # char ret: Rust char to single-character string
  expect_equal(conv_char_ret(), "α")
  expect_equal(nchar(conv_char_ret()), 1L)
})

test_that("Vec coercion (i8/i16/u16/f32) returns work", {
  # Vec<i8> → integer (coerced to i32)
  expect_equal(conv_vec_i8_ret(), c(1L, -1L, 127L))

  # Vec<i16> → integer (coerced to i32)
  expect_equal(conv_vec_i16_ret(), c(1L, -1L, 32767L))

  # Vec<u16> → integer (coerced to i32)
  expect_equal(conv_vec_u16_ret(), c(1L, 100L, 65535L))

  # Vec<f32> → double (coerced to f64)
  v <- conv_vec_f32_ret()
  expect_equal(length(v), 3L)
  expect_true(is.double(v))
  expect_equal(v[1], 1.5, tolerance = 1e-6)
  expect_equal(v[2], 2.5, tolerance = 1e-6)
  expect_equal(v[3], -0.5, tolerance = 1e-6)
})

test_that("HashSet/BTreeSet coercion (i8/i16/u16) returns work", {
  # HashSet<i8> → integer (coerced to i32, order not guaranteed)
  expect_true(setequal(conv_hashset_i8_ret(), c(1L, 2L, -1L)))

  # BTreeSet<i8> → integer (coerced to i32, sorted)
  expect_equal(sort(conv_btreeset_i8_ret()), c(-1L, 1L, 2L))

  # HashSet<i16> → integer
  expect_true(setequal(conv_hashset_i16_ret(), c(1L, 2L, -1L)))

  # BTreeSet<i16> → integer
  expect_equal(sort(conv_btreeset_i16_ret()), c(-1L, 1L, 2L))

  # HashSet<u16> → integer
  expect_true(setequal(conv_hashset_u16_ret(), c(1L, 2L, 100L)))

  # BTreeSet<u16> → integer
  expect_equal(sort(conv_btreeset_u16_ret()), c(1L, 2L, 100L))
})

test_that("optional reference return gives value or NULL", {
  # Some(&i32) → integer (copied)
  expect_equal(conv_opt_ref_i32_some_ret(), 42L)

  # None → NULL (not NA, since there's no reference)
  expect_true(is.null(conv_opt_ref_i32_none_ret()))
})

test_that("Option<Vec<T>> conversions work", {
  # Arg: Some(vec) passes through
  expect_equal(conv_opt_vec_i32_arg(c(1L, 2L, 3L)), 6L)  # sum

  # Arg: NULL → None
  expect_equal(conv_opt_vec_i32_arg(NULL), -999L)

  # Ret: Some → vector
  expect_equal(conv_opt_vec_i32_some_ret(), c(1L, 2L, 3L))

  # Ret: None → NULL
  expect_true(is.null(conv_opt_vec_i32_none_ret()))

  # String vec
  expect_equal(conv_opt_vec_string_arg(c("a", "b")), 2L)  # length
  expect_equal(conv_opt_vec_string_arg(NULL), -999L)
  expect_equal(conv_opt_vec_string_some_ret(), c("a", "b"))
  expect_true(is.null(conv_opt_vec_string_none_ret()))
})

test_that("Option<HashMap> conversions work", {
  # Arg: Some(map) passes through
  expect_equal(conv_opt_hashmap_i32_arg(list(a = 1L, b = 2L)), 3L)  # sum

  # Arg: NULL → None
  expect_equal(conv_opt_hashmap_i32_arg(NULL), -999L)

  # Ret: Some → named list
  res <- conv_opt_hashmap_i32_some_ret()
  expect_true(is.list(res))
  expect_true(all(c("a", "b") %in% names(res)))
  expect_equal(sum(unlist(res)), 3L)

  # Ret: None → NULL
  expect_true(is.null(conv_opt_hashmap_i32_none_ret()))
})

test_that("Option<HashSet> conversions work", {
  # Arg: Some(set) passes through
  expect_equal(conv_opt_hashset_i32_arg(c(1L, 2L, 3L)), 6L)  # sum

  # Arg: NULL → None
  expect_equal(conv_opt_hashset_i32_arg(NULL), -999L)

  # Ret: Some → vector
  expect_true(setequal(conv_opt_hashset_i32_some_ret(), c(1L, 2L, 3L)))

  # Ret: None → NULL
  expect_true(is.null(conv_opt_hashset_i32_none_ret()))
})

test_that("Vec<Vec<T>> (list of vectors) conversions work", {
  # Arg: list of integer vectors
  expect_equal(conv_vec_vec_i32_arg(list(c(1L, 2L), c(3L, 4L, 5L))), 15L)  # sum of sums

  # Ret: Vec<Vec<i32>> → list of integer vectors
  res <- conv_vec_vec_i32_ret()
  expect_true(is.list(res))
  expect_equal(length(res), 2L)
  expect_equal(res[[1]], c(1L, 2L))
  expect_equal(res[[2]], c(3L, 4L, 5L))

  # Ret: Vec<Vec<String>> → list of character vectors
  res_str <- conv_vec_vec_string_ret()
  expect_true(is.list(res_str))
  expect_equal(length(res_str), 2L)
  expect_equal(res_str[[1]], c("a", "b"))
  expect_equal(res_str[[2]], "c")
})

test_that("Vec<Option<T>> extended numeric return types", {
  # Vec<Option<i64>> with small values → INTSXP
  v_small <- conv_vec_option_i64_ret_small()
  expect_true(is.integer(v_small))
  expect_equal(v_small, c(1L, NA_integer_, 3L))

  # Vec<Option<i64>> with overflow values → REALSXP
  v_big <- conv_vec_option_i64_ret_big()
  expect_true(is.double(v_big))
  expect_true(is.na(v_big[2]))
  expect_equal(v_big[3], 1.0)

  # Vec<Option<u32>> → delegates to smart i64 path → INTSXP
  v_u32 <- conv_vec_option_u32_ret()
  expect_true(is.integer(v_u32))
  expect_equal(v_u32, c(1L, NA_integer_, 42L))

  # Vec<Option<f32>> → coerces to Vec<Option<f64>> → REALSXP
  v_f32 <- conv_vec_option_f32_ret()
  expect_true(is.double(v_f32))
  expect_equal(v_f32[1], 1.5)
  expect_true(is.na(v_f32[2]))
  expect_equal(v_f32[3], 3.0)

  # Vec<Option<u64>> with small values → INTSXP
  v_u64_small <- conv_vec_option_u64_ret_small()
  expect_true(is.integer(v_u64_small))
  expect_equal(v_u64_small, c(1L, NA_integer_, 3L))

  # Vec<Option<u64>> with overflow values → REALSXP
  v_u64_big <- conv_vec_option_u64_ret_big()
  expect_true(is.double(v_u64_big))
  expect_true(is.na(v_u64_big[2]))
  expect_equal(v_u64_big[3], 1.0)

  # Vec<Option<isize>> with small values → INTSXP
  v_isize <- conv_vec_option_isize_ret_small()
  expect_true(is.integer(v_isize))
  expect_equal(v_isize, c(1L, NA_integer_, 3L))

  # Vec<Option<usize>> with small values → INTSXP
  v_usize <- conv_vec_option_usize_ret_small()
  expect_true(is.integer(v_usize))
  expect_equal(v_usize, c(1L, NA_integer_, 3L))

  # Vec<Option<i8>> → coerces to Vec<Option<i32>> → INTSXP
  v_i8 <- conv_vec_option_i8_ret()
  expect_true(is.integer(v_i8))
  expect_equal(v_i8, c(1L, NA_integer_, -1L))

  # Vec<Option<i16>> → coerces to Vec<Option<i32>> → INTSXP
  v_i16 <- conv_vec_option_i16_ret()
  expect_true(is.integer(v_i16))
  expect_equal(v_i16, c(1L, NA_integer_, -1L))

  # Vec<Option<u16>> → coerces to Vec<Option<i32>> → INTSXP
  v_u16 <- conv_vec_option_u16_ret()
  expect_true(is.integer(v_u16))
  expect_equal(v_u16, c(1L, NA_integer_, 100L))
})

test_that("Vec<Option<i64>> roundtrip preserves NA", {
  # Small values: roundtrip through INTSXP
  result <- conv_vec_option_i64_roundtrip(c(1L, NA_integer_, 3L))
  expect_true(is.integer(result))
  expect_equal(result, c(1L, NA_integer_, 3L))

  # All NA: roundtrip should work
  result_na <- conv_vec_option_i64_roundtrip(c(NA_integer_, NA_integer_))
  expect_true(all(is.na(result_na)))
  expect_equal(length(result_na), 2L)

  # Empty vector: roundtrip
  result_empty <- conv_vec_option_i64_roundtrip(integer(0))
  expect_equal(length(result_empty), 0L)
})

test_that("Scalar Option<T> extended numeric return types", {
  # Option<i64> with small value -> integer
  expect_true(is.integer(conv_option_i64_some_small()))
  expect_equal(conv_option_i64_some_small(), 42L)

  # Option<i64> with large value -> double
  expect_true(is.double(conv_option_i64_some_big()))

  # Option<i64> None -> NA (integer)
  expect_true(is.na(conv_option_i64_none()))

  # Option<f32> -> double
  expect_true(is.double(conv_option_f32_some()))
  expect_equal(conv_option_f32_some(), 1.5)

  # Option<u32> -> integer (fits)
  expect_true(is.integer(conv_option_u32_some()))
  expect_equal(conv_option_u32_some(), 100L)
})

# ── AsNamedList / AsNamedVector ─────────────────────────────────────────────

test_that("AsNamedList<Vec<(K,V)>> creates named R list", {
  res <- conv_as_named_list_vec()
  expect_true(is.list(res))
  expect_equal(names(res), c("width", "height", "depth"))
  expect_equal(res$width, 100L)
  expect_equal(res$height, 200L)
  expect_equal(res$depth, 300L)
})

test_that("AsNamedList<[(K,V); N]> creates named R list from array", {
  res <- conv_as_named_list_array()
  expect_true(is.list(res))
  expect_equal(names(res), c("pi", "e"))
  expect_equal(res$pi, pi)
  expect_equal(res$e, exp(1))
})

test_that("AsNamedList supports heterogeneous value types", {
  res <- conv_as_named_list_heterogeneous()
  expect_true(is.list(res))
  expect_equal(names(res), c("name", "age", "score"))
  expect_equal(res$name, "Alice")
  expect_equal(res$age, 30L)
  expect_equal(res$score, 95.5)
})

test_that("AsNamedList works with &str keys", {
  res <- conv_as_named_list_str_keys()
  expect_true(is.list(res))
  expect_equal(names(res), c("a", "b"))
  expect_equal(res$a, 1L)
  expect_equal(res$b, 2L)
})

test_that("AsNamedList handles empty input", {
  res <- conv_as_named_list_empty()
  expect_true(is.list(res))
  expect_equal(length(res), 0L)
})

test_that("AsNamedList preserves duplicate names", {
  res <- conv_as_named_list_duplicate_names()
  expect_true(is.list(res))
  expect_equal(names(res), c("x", "x", "x"))
  expect_equal(length(res), 3L)
  expect_equal(res[[1]], 1L)
  expect_equal(res[[2]], 2L)
  expect_equal(res[[3]], 3L)
})

test_that("AsNamedVector<Vec<(K,V)>> creates named integer vector", {
  res <- conv_as_named_vector_i32()
  expect_true(is.integer(res))
  expect_equal(names(res), c("alice", "bob", "carol"))
  expect_equal(unname(res), c(95L, 87L, 92L))
})

test_that("AsNamedVector creates named double vector", {
  res <- conv_as_named_vector_f64()
  expect_true(is.double(res))
  expect_equal(names(res), c("pi", "e"))
  expect_equal(unname(res), c(pi, exp(1)))
})

test_that("AsNamedVector creates named character vector", {
  res <- conv_as_named_vector_string()
  expect_true(is.character(res))
  expect_equal(names(res), c("greeting", "farewell"))
  expect_equal(unname(res), c("hello", "goodbye"))
})

test_that("AsNamedVector with Option<i32> preserves NA", {
  res <- conv_as_named_vector_option_i32()
  expect_true(is.integer(res))
  expect_equal(names(res), c("a", "b", "c"))
  expect_equal(res[["a"]], 1L)
  expect_true(is.na(res[["b"]]))
  expect_equal(res[["c"]], 3L)
})

test_that("AsNamedVector<[(K,V); N]> works with array input", {
  res <- conv_as_named_vector_array()
  expect_true(is.double(res))
  expect_equal(names(res), c("x", "y", "z"))
  expect_equal(unname(res), c(1.0, 2.0, 3.0))
})

test_that("AsNamedVector handles empty input", {
  res <- conv_as_named_vector_empty()
  expect_true(is.double(res))
  expect_equal(length(res), 0L)
})

test_that("extension traits (.as_named_vector / .as_named_list) work", {
  vec_res <- conv_as_named_vector_ext_trait()
  expect_true(is.integer(vec_res))
  expect_equal(names(vec_res), c("one", "two"))
  expect_equal(unname(vec_res), c(1L, 2L))

  list_res <- conv_as_named_list_ext_trait()
  expect_true(is.list(list_res))
  expect_equal(names(list_res), c("one", "two"))
  expect_equal(list_res$one, 1L)
  expect_equal(list_res$two, 2L)
})

# =============================================================================
# Map conversion edge cases (Phase A1/A2)
# =============================================================================

test_that("HashMap rejects duplicate non-empty names", {
  expect_error(
    conv_hashmap_i32_len(list(a = 1L, b = 2L, a = 3L)),
    "DuplicateName"
  )
})

test_that("BTreeMap rejects duplicate non-empty names", {
  expect_error(
    conv_btreemap_i32_len(list(a = 1L, b = 2L, a = 3L)),
    "DuplicateName"
  )
})

test_that("Option<HashMap> non-NULL path rejects duplicate names", {
  expect_error(
    conv_opt_hashmap_i32_arg(list(a = 1L, b = 2L, a = 3L)),
    "DuplicateName"
  )
})

test_that("HashMap uses index keys for completely unnamed lists", {
  # list(1L, 2L) has no names → keys "0" and "1"
  result <- conv_hashmap_i32_len(list(1L, 2L))
  expect_equal(result, 2L)
})

test_that("HashMap NA/empty names collapse to empty string with last-write-wins", {
  # Explicit NA name and empty name both → "" key, last write wins
  x <- list(1L, 2L)
  names(x) <- c(NA, "")
  result <- conv_hashmap_i32_len(x)
  expect_equal(result, 1L)  # collapsed to single "" key
})

test_that("BTreeMap uses index keys for completely unnamed lists", {
  result <- conv_btreemap_i32_len(list(1L, 2L))
  expect_equal(result, 2L)
})

test_that("BTreeMap NA/empty names collapse to empty string with last-write-wins", {
  x <- list(1L, 2L)
  names(x) <- c(NA, "")
  result <- conv_btreemap_i32_len(x)
  expect_equal(result, 1L)
})

test_that("HashMap mixed named/unnamed: empty names collapse to empty string", {
  # list(1L, a = 2L, 3L): "" entries (positions 1 and 3) → key ""
  # last-write-wins gives key "" → 3, key "a" → 2 → total length 2
  result <- conv_hashmap_i32_len(list(1L, a = 2L, 3L))
  expect_equal(result, 2L)
})

# =============================================================================
# Option<BTreeMap> conversions (Phase A3)
# =============================================================================

test_that("Option<BTreeMap> arg works for Some and None", {
  expect_equal(conv_opt_btreemap_i32_arg(list(a = 1L, b = 2L)), 3L)
  expect_equal(conv_opt_btreemap_i32_arg(NULL), -999L)
})

test_that("Option<BTreeMap> return Some produces named list", {
  res <- conv_opt_btreemap_i32_some_ret()
  expect_true(is.list(res))
  expect_equal(res$a, 1L)
  expect_equal(res$b, 2L)
})

test_that("Option<BTreeMap> return None produces NULL", {
  expect_null(conv_opt_btreemap_i32_none_ret())
})

test_that("Option<BTreeMap> non-NULL path rejects duplicate names", {
  expect_error(
    conv_opt_btreemap_i32_arg(list(a = 1L, b = 2L, a = 3L)),
    "DuplicateName"
  )
})

# =============================================================================
# Vec<HashMap> / Vec<BTreeMap> conversions (Phase A3)
# =============================================================================

test_that("Vec<HashMap> arg sums correctly", {
  input <- list(list(a = 1L, b = 2L), list(c = 3L))
  expect_equal(conv_vec_hashmap_i32_arg(input), 6L)
})

test_that("Vec<HashMap> ret produces list of named lists", {
  res <- conv_vec_hashmap_i32_ret()
  expect_true(is.list(res))
  expect_equal(length(res), 2L)
  expect_equal(res[[1]]$a, 1L)
  expect_equal(length(res[[2]]), 2L)
})

test_that("Vec<BTreeMap> arg sums correctly", {
  input <- list(list(x = 10L, y = 20L), list(z = 30L))
  expect_equal(conv_vec_btreemap_i32_arg(input), 60L)
})

test_that("Vec<BTreeMap> ret produces list of named lists", {
  res <- conv_vec_btreemap_i32_ret()
  expect_true(is.list(res))
  expect_equal(length(res), 2L)
  expect_equal(res[[1]]$x, 10L)
  expect_equal(res[[2]]$y, 20L)
  expect_equal(res[[2]]$z, 30L)
})

test_that("Vec<HashMap> inner list rejects duplicate names", {
  expect_error(
    conv_vec_hashmap_i32_arg(list(list(a = 1L, a = 2L))),
    "DuplicateName"
  )
})

test_that("AsNamedList from borrowed slice creates named list", {
  res <- conv_as_named_list_slice()
  expect_true(is.list(res))
  expect_equal(names(res), c("x", "y", "z"))
  expect_equal(res$x, 10L)
  expect_equal(res$y, 20L)
  expect_equal(res$z, 30L)
})

test_that("AsNamedVector from borrowed slice creates named atomic vector", {
  res <- conv_as_named_vector_slice()
  expect_true(is.double(res))
  expect_equal(names(res), c("a", "b"))
  expect_equal(unname(res), c(1.5, 2.5))
})
