# #1112: the coerce flag must preserve the default converter's input domain.
coerce_input_fixture <- function(kind, type) {
  get(paste0("coerce_input_", kind, "_", type), asNamespace("miniextendr"))
}

test_that("coerced numeric scalars accept all existing numeric source types", {
  sources <- list(integer = 1L, double = 1, logical = TRUE, raw = as.raw(1))
  for (type in c("i8", "i16", "u16", "u32", "i64", "u64", "isize", "usize", "f32")) {
    fun <- coerce_input_fixture("scalar", type)
    for (source in sources) {
      expect_equal(as.numeric(fun(source)), 1, info = type)
    }
  }
  for (type in c("i64", "u64")) {
    # A valid R double outside INTSXP range must not be routed through i32.
    expect_equal(as.numeric(coerce_input_fixture("scalar", type)(2^40)), 2^40)
  }
})

test_that("coerced numeric vectors accept all existing source types and empties", {
  sources <- list(integer = c(1L, 0L, 1L), double = c(1, 0, 1),
                  logical = c(TRUE, FALSE, TRUE), raw = as.raw(c(1, 0, 1)))
  for (type in c("i8", "i16", "u16", "u32", "i64", "u64", "isize", "usize", "f32")) {
    fun <- coerce_input_fixture("vector", type)
    for (source in sources) {
      expect_equal(as.numeric(fun(source)), c(1, 0, 1), info = type)
      expect_length(fun(source[FALSE]), 0L)
    }
  }
})

test_that("coerced bools retain logical inputs while accepting integer zero and one", {
  scalar <- coerce_input_fixture("scalar", "bool")
  vector <- coerce_input_fixture("vector", "bool")
  for (x in list(TRUE, 1L)) expect_true(scalar(x))
  for (x in list(FALSE, 0L)) expect_false(scalar(x))
  for (x in list(c(TRUE, FALSE), c(1L, 0L))) {
    expect_identical(vector(x), c(TRUE, FALSE))
    expect_identical(vector(x[FALSE]), logical())
  }
  for (x in list(NA, NA_integer_, 2L, -1L, 1, as.raw(1))) {
    expect_error(scalar(x))
    expect_error(vector(x))
  }
  for (x in list(integer(), logical(), c(TRUE, FALSE))) expect_error(scalar(x))
})

test_that("per-argument, worker, and optional bool coercion agree with their R gates", {
  fun <- miniextendr:::coerce_input_per_arg
  expect_identical(fun(TRUE, FALSE), c(TRUE, FALSE))
  expect_identical(fun(1L, TRUE), c(TRUE, TRUE))
  expect_error(fun(1L, 1L))
  if (miniextendr_has_feature("worker-thread")) {
    expect_true(miniextendr:::coerce_input_worker(TRUE))
    expect_true(miniextendr:::coerce_input_worker(1L))
  }
  expect_true(miniextendr:::coerce_input_optional(TRUE))
  expect_false(miniextendr:::coerce_input_optional(NA))
  expect_false(miniextendr:::coerce_input_optional(NULL))
  expect_error(miniextendr:::coerce_input_optional(1L))
})

test_that("newly accepted vectors retain batched conversion diagnostics", {
  err <- expect_error(miniextendr:::coerce_input_fast(c(-1, 5, 70000, 1.5, NaN)))
  msg <- conditionMessage(err)
  expect_s3_class(err, "rust_error")
  expect_match(msg, "Vec<u16> conversion failed", fixed = TRUE)
  for (i in c(0, 2, 3, 4)) {
    expect_match(msg, paste("invalid value at index", i), fixed = TRUE)
  }
  logical_error <- expect_error(coerce_input_fixture("vector", "bool")(c(NA, TRUE, NA)))
  for (i in c(0, 2)) {
    expect_match(conditionMessage(logical_error), paste("invalid value at index", i), fixed = TRUE)
  }
})

test_that("strict retains precedence over coerce", {
  for (fun in list(miniextendr:::coerce_input_strict,
                   miniextendr:::coerce_input_strict_vector)) {
    expect_equal(as.numeric(fun(1L)), 1)
    expect_equal(as.numeric(fun(1)), 1)
    expect_error(fun(TRUE))
    expect_error(fun(as.raw(1)))
  }
})
