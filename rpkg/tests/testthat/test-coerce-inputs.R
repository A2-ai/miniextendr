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

test_that("coerced native i32 and f64 widen from the other numeric sources", {
  # The bare conversion is one SEXPTYPE (#1112): 3 reaches `x: i32` only with coerce.
  expect_error(miniextendr:::coerce_input_native_default(3))
  expect_identical(miniextendr:::coerce_input_native_default(3L), 3L)

  int_scalar <- coerce_input_fixture("scalar", "i32")
  for (x in list(3L, 3, as.raw(3))) expect_identical(int_scalar(x), 3L, info = typeof(x))
  expect_identical(int_scalar(TRUE), 1L)
  expect_identical(int_scalar(-2), -2L)
  # The R gate names the widened domain and rejects what Rust would reject.
  err <- expect_error(int_scalar(3.5))
  expect_match(conditionMessage(err), "must be integer or whole-number numeric", fixed = TRUE)
  # A scalar i32 cannot carry NA, so every NA stays an error, as without coerce.
  for (x in list(NA_integer_, NA_real_, NA, 2^31, Inf, "3", c(1, 2))) expect_error(int_scalar(x))

  int_vector <- coerce_input_fixture("vector", "i32")
  for (x in list(c(1L, 0L, 2L), c(1, 0, 2), as.raw(c(1, 0, 2)))) {
    expect_identical(int_vector(x), c(1L, 0L, 2L), info = typeof(x))
    expect_identical(int_vector(x[FALSE]), integer())
  }
  expect_identical(int_vector(c(TRUE, FALSE)), c(1L, 0L))
  # The vector can carry NA: as.integer() semantics for NA_real_ and logical NA.
  expect_identical(int_vector(c(1, NA)), c(1L, NA))
  expect_identical(int_vector(c(TRUE, NA)), c(1L, NA))
  err <- expect_error(int_vector(c(1, 2.5)))
  expect_match(conditionMessage(err), "must be integer or whole-number numeric", fixed = TRUE)

  real_scalar <- coerce_input_fixture("scalar", "f64")
  for (x in list(2L, 2, as.raw(2))) expect_identical(real_scalar(x), 2, info = typeof(x))
  expect_identical(real_scalar(TRUE), 1)
  # as.numeric() semantics: every NA arrives as NA_real_; REALSXP keeps the bare path.
  for (x in list(NA_real_, NA_integer_, NA)) expect_identical(real_scalar(x), NA_real_)
  for (x in list("2", c(1, 2), integer())) expect_error(real_scalar(x))

  real_vector <- coerce_input_fixture("vector", "f64")
  for (x in list(c(1L, 0L, 2L), c(1, 0, 2), as.raw(c(1, 0, 2)))) {
    expect_identical(real_vector(x), c(1, 0, 2), info = typeof(x))
    expect_identical(real_vector(x[FALSE]), numeric())
  }
  expect_identical(real_vector(c(TRUE, FALSE, NA)), c(1, 0, NA))
  expect_identical(real_vector(c(1L, NA_integer_)), c(1, NA))

  # Without the R gate (fast), the Rust side batches every failing index.
  err <- expect_error(miniextendr:::coerce_input_fast_i32(c(1, 2.5, NaN, 2^31, NA)))
  msg <- conditionMessage(err)
  expect_match(msg, "Vec<i32> conversion failed", fixed = TRUE)
  for (i in c(1, 2, 3)) expect_match(msg, paste("invalid value at index", i), fixed = TRUE)
  expect_false(grepl("index 4", msg, fixed = TRUE))
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
  # A non-optional numeric vector rejects NA at its index instead of letting the
  # NA_integer_ sentinel coerce to -2147483648.
  na_error <- expect_error(coerce_input_fixture("vector", "i64")(c(1L, NA, 3L, NA)))
  for (i in c(1, 3)) {
    expect_match(conditionMessage(na_error), paste("invalid value at index", i), fixed = TRUE)
  }
  expect_error(coerce_input_fixture("vector", "f32")(c(TRUE, NA)))
  expect_error(coerce_input_fixture("vector", "u16")(c(1L, NA_integer_)))
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
