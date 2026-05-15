# Tests for call attribution behaviour documented in docs/CALL_ATTRIBUTION.md.
# Both fixtures are internal (:::); this file makes the transcript machine-verifiable.

# region: call_attr_with (wrapped path — with call attribution)

test_that("call_attr_with produces rust_error with call attribution", {
  e <- tryCatch(
    miniextendr:::call_attr_with(1L, 2L),
    error = function(e) e
  )
  expect_s3_class(e, "rust_error")
  call_str <- deparse(conditionCall(e))
  expect_match(call_str, "call_attr_with")
})

test_that("call_attr_with conditionCall includes formal parameter names", {
  # match.call() captures named formals: left = ..., right = ...
  e <- tryCatch(
    miniextendr:::call_attr_with(1L, 2L),
    error = function(e) e
  )
  call_str <- deparse(conditionCall(e))
  expect_match(call_str, "left")
  expect_match(call_str, "right")
})

# endregion

# region: unsafe_C_call_attr_without (unwrapped path — no call attribution)

test_that("unsafe_C_call_attr_without produces simpleError without rust_error class", {
  e <- tryCatch(
    miniextendr:::unsafe_C_call_attr_without(1L, 2L),
    error = function(e) e
  )
  expect_false(inherits(e, "rust_error"))
  expect_s3_class(e, "simpleError")
})

test_that("wrapped and unwrapped paths differ in call attribution", {
  # The wrapped path (call_attr_with) goes through error_in_r and match.call(),
  # so conditionCall includes formal names. The unwrapped path (Rf_error) does not.
  e_with <- tryCatch(
    miniextendr:::call_attr_with(1L, 2L),
    error = function(e) e
  )
  e_without <- tryCatch(
    miniextendr:::unsafe_C_call_attr_without(1L, 2L),
    error = function(e) e
  )
  # Only the wrapped path emits rust_error class
  expect_true(inherits(e_with, "rust_error"))
  expect_false(inherits(e_without, "rust_error"))
  # Wrapped path conditionCall includes formal parameter names
  call_with_str <- deparse(conditionCall(e_with))
  expect_match(call_with_str, "left")
})

# endregion

# region: export check

test_that("call attribution fixtures are not exported", {
  exports <- getNamespaceExports("miniextendr")
  expect_false("call_attr_with" %in% exports)
  expect_false("unsafe_C_call_attr_without" %in% exports)
})

# endregion
