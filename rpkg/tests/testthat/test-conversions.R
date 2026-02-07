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
