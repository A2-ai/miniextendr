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

  # Without the R gate (no_preconditions), the Rust side batches every failing element,
  # by reason and 1-based position; the NA (element 5) is a valid NA_integer_.
  err <- expect_error(miniextendr:::coerce_input_no_preconditions_i32(c(1, 2.5, NaN, 2^31, NA)))
  expect_identical(
    conditionMessage(err),
    paste0(
      "'x' must be integer or whole-number numeric: precision loss (element 2); ",
      "NaN cannot be converted (element 3); value out of range (element 4)"
    )
  )
  expect_identical(err$rust_type, "Vec<i32>")
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

test_that("per-argument bool coercion agrees with its R gate", {
  fun <- miniextendr:::coerce_input_per_arg
  expect_identical(fun(TRUE, FALSE), c(TRUE, FALSE))
  expect_identical(fun(1L, TRUE), c(TRUE, TRUE))
  expect_error(fun(1L, 1L))
})

# #2274: `coerce` widens the NA-carrying forms too. NULL and NA read None
# (returned as NA), other values convert as the plain coerced type does.
test_that("coerced Option<i32> widens and reads NULL and every NA as None", {
  fun <- coerce_input_fixture("optional", "i32")
  for (x in list(3L, 3, as.raw(3))) expect_identical(fun(x), 3L, info = typeof(x))
  expect_identical(fun(TRUE), 1L)
  for (x in list(NULL, NA_integer_, NA_real_, NA)) expect_identical(fun(x), NA_integer_)
  # The R gate is the scalar's widened check behind is.null().
  err <- expect_error(fun(3.5))
  expect_identical(
    conditionMessage(err),
    "'x' must be NULL or integer or whole-number numeric"
  )
  for (x in list("3", c(1L, 2L), integer(), 2^31, Inf, NaN)) expect_error(fun(x))
  # Without the gate, the Rust conversion refuses the same values.
  raw_fun <- miniextendr:::coerce_input_no_preconditions_optional_scalar_i32
  expect_identical(raw_fun(NA_real_), NA_integer_)
  expect_identical(raw_fun(NULL), NA_integer_)
  expect_identical(raw_fun(-2), -2L)
  err <- expect_error(raw_fun(3.5))
  expect_match(conditionMessage(err), "precision loss", fixed = TRUE)
  expect_identical(err$rust_type, "Option<i32>")
  expect_error(raw_fun(NaN))
})

test_that("coerced Option<f64> widens and reads NULL and every NA as None", {
  fun <- coerce_input_fixture("optional", "f64")
  for (x in list(2L, 2, as.raw(2))) expect_identical(fun(x), 2, info = typeof(x))
  expect_identical(fun(TRUE), 1)
  is_none <- miniextendr:::coerce_input_optional_f64_is_none
  for (x in list(NULL, NA_real_, NA_integer_, NA)) expect_true(is_none(x))
  # NaN stays a value, as for the bare Option<f64>.
  expect_false(is_none(NaN))
  expect_identical(fun(NaN), NaN)
  err <- expect_error(fun("2"))
  expect_identical(conditionMessage(err), "'x' must be NULL or numeric, logical, or raw")
  for (x in list(c(1, 2), numeric())) expect_error(fun(x))
})

test_that("coerced Option<bool> widens to integer zero and one", {
  fun <- coerce_input_fixture("optional", "bool")
  for (x in list(TRUE, 1L)) expect_true(fun(x))
  for (x in list(FALSE, 0L)) expect_false(fun(x))
  for (x in list(NULL, NA, NA_integer_)) expect_identical(fun(x), NA)
  err <- expect_error(fun(1))
  expect_identical(conditionMessage(err), "'x' must be NULL or logical or integer")
  for (x in list(2L, -1L, as.raw(1), c(TRUE, FALSE))) expect_error(fun(x))
})

test_that("coerced Vec<Option<T>> widens element-wise and reads NA as None", {
  int_vector <- coerce_input_fixture("optional_vector", "i32")
  for (x in list(c(1L, NA, 2L), c(1, NA, 2))) {
    expect_identical(int_vector(x), c(1L, NA, 2L), info = typeof(x))
  }
  expect_identical(int_vector(c(TRUE, NA, FALSE)), c(1L, NA, 0L))
  expect_identical(int_vector(as.raw(c(1, 2))), c(1L, 2L))
  expect_identical(int_vector(numeric()), integer())
  err <- expect_error(int_vector(c(1, 2.5)))
  expect_identical(conditionMessage(err), "'x' must be integer or whole-number numeric")
  # A vector reads NULL no more than Vec<T> does.
  expect_error(int_vector(NULL))

  real_vector <- coerce_input_fixture("optional_vector", "f64")
  for (x in list(c(1L, NA, 2L), c(1, NA, 2), c(TRUE, NA, TRUE))) {
    expect_identical(real_vector(x), c(as.numeric(x[1]), NA, as.numeric(x[3])), info = typeof(x))
  }
  expect_identical(real_vector(as.raw(c(1, 2))), c(1, 2))
  expect_identical(real_vector(c(NaN, 1)), c(NaN, 1))
  expect_error(real_vector(c("1", NA)))
  expect_error(real_vector(NULL))

  bool_vector <- coerce_input_fixture("optional_vector", "bool")
  for (x in list(c(TRUE, NA, FALSE), c(1L, NA, 0L))) {
    expect_identical(bool_vector(x), c(TRUE, NA, FALSE), info = typeof(x))
  }
  expect_identical(bool_vector(integer()), logical())
  err <- expect_error(bool_vector(c(1, 0)))
  expect_identical(conditionMessage(err), "'x' must be logical or integer")
  expect_error(bool_vector(NULL))
})

test_that("coerced Vec<Option<T>> batches every failing element", {
  err <- expect_error(
    miniextendr:::coerce_input_no_preconditions_optional_i32(c(1, 2.5, NA, NaN, 2^31, 3.5))
  )
  expect_identical(
    conditionMessage(err),
    paste0(
      "'x' must be integer or whole-number numeric: precision loss (elements 2, 6); ",
      "NaN cannot be converted (element 4); value out of range (element 5)"
    )
  )
  expect_identical(err$rust_type, "Vec<Option<i32>>")
  expect_identical(
    miniextendr:::coerce_input_no_preconditions_optional_i32(c(1, NA, 2)),
    c(1L, NA, 2L)
  )

  bool_fun <- miniextendr:::coerce_input_no_preconditions_optional_bool
  expect_identical(bool_fun(c(1L, NA, 0L)), c(TRUE, NA, FALSE))
  err <- expect_error(bool_fun(c(1L, 2L, NA, -1L)))
  expect_identical(
    conditionMessage(err),
    paste0(
      "'x' must be logical or integer: 2 is neither 0 nor 1 (element 2); ",
      "-1 is neither 0 nor 1 (element 4)"
    )
  )
  expect_identical(err$rust_type, "Vec<Option<bool>>")
})

test_that("per-argument coercion widens an optional double, not its neighbor", {
  fun <- miniextendr:::coerce_input_optional_per_arg
  expect_identical(fun(1L, 2), c(1, 2))
  expect_identical(fun(NULL, NULL), c(NA_real_, NA_real_))
  expect_identical(fun(NA, NA_real_), c(NA_real_, NA_real_))
  err <- expect_error(fun(1, 2L))
  expect_identical(conditionMessage(err), "'y' must be NULL or double")
})

test_that("newly accepted vectors retain batched conversion diagnostics", {
  err <- expect_error(miniextendr:::coerce_input_no_preconditions(c(-1, 5, 70000, 1.5, NaN)))
  msg <- conditionMessage(err)
  expect_s3_class(err, "rust_error")
  # Each reason once, with the 1-based positions that failed with it; the Rust
  # type is e$rust_type, not part of the message.
  expect_identical(
    msg,
    paste0(
      "'x' must be integer or whole-number numeric: value out of range (elements 1, 3); ",
      "precision loss (element 4); NaN cannot be converted (element 5)"
    )
  )
  expect_identical(err$rust_type, "Vec<u16>")
  # A non-optional numeric vector rejects NA at its position instead of letting the
  # NA_integer_ sentinel coerce to -2147483648.
  na_error <- expect_error(coerce_input_fixture("vector", "i64")(c(1L, NA, 3L, NA)))
  expect_match(conditionMessage(na_error), ": NA is not allowed (elements 2, 4)", fixed = TRUE)
  expect_error(coerce_input_fixture("vector", "f32")(c(TRUE, NA)))
  expect_error(coerce_input_fixture("vector", "u16")(c(1L, NA_integer_)))
  logical_error <- expect_error(coerce_input_fixture("vector", "bool")(c(NA, TRUE, NA)))
  expect_match(conditionMessage(logical_error), ": NA is not allowed (elements 1, 3)", fixed = TRUE)
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
