test_that("objects can be returned as pointer, list, or native R type", {
  expect_identical(typeof(hybrid_as_ptr(2L)), "externalptr")
  expect_identical(typeof(hybrid_as_list(2L)), "list")
  expect_identical(typeof(hybrid_as_native(2L)), "integer")
  expect_identical(hybrid_as_native(2L), 2L)
})

test_that("return-type wrappers work with subset of representations", {
  expect_identical(typeof(ptr_list_as_ptr(5L)), "externalptr")
  expect_identical(typeof(ptr_list_as_list(5L)), "list")
  expect_identical(ptr_list_as_list(5L)[[1]], 5L)

  expect_identical(typeof(native_list_as_list(6L)), "list")
  expect_identical(native_list_as_list(6L)[[1]], 6L)
  expect_identical(typeof(native_list_as_native(6L)), "integer")
  expect_identical(native_list_as_native(6L), 6L)
})

test_that("prefer = 'list' attribute returns a list identical to explicit AsList wrapper", {
  result <- attr_prefer_list(1L)
  expect_identical(typeof(result), "list")
  expect_identical(result, hybrid_as_list(1L))
})

test_that("prefer = 'externalptr' attribute returns an externalptr identical to explicit AsExternalPtr wrapper", {
  result <- attr_prefer_externalptr(1L)
  expect_identical(typeof(result), "externalptr")
  expect_identical(typeof(result), typeof(hybrid_as_ptr(1L)))
})

test_that("prefer = 'native' attribute returns an integer scalar identical to explicit AsRNative wrapper", {
  result <- attr_prefer_native(1L)
  expect_identical(typeof(result), "integer")
  expect_identical(result, hybrid_as_native(1L))
})

test_that("AsDataFrame wrapper forces a data.frame return", {
  df <- pref_rows_as_data_frame()
  expect_s3_class(df, "data.frame")
  expect_identical(nrow(df), 2L)
  expect_identical(names(df), c("id", "label"))
  expect_identical(df$id, c(1L, 2L))
  expect_identical(df$label, c("a", "b"))
})

# `prefer = "list"` on a function returning `Option<T>` used to be a silent no-op (the
# attribute was accepted and dropped by `apply_return_pref`). It is now a hard compile
# error from `#[miniextendr]` — see `miniextendr-macros/tests/ui/fn_prefer_option_return.rs`
# for the compile-fail regression test (BUG4).

