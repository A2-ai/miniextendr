# `Nullable<T>` arguments read as `Option<T>` (#2294):
# src/rust/nullable_argument_tests.rs. Each `nullable_arg_*` fixture has an
# `option_arg_*` twin; both take the same values and refuse a wrong one with
# the same message, condition class, `kind` and `e$param`. Only `e$rust_type`
# (set by a failed Rust conversion) names the type as written.

caught <- function(expr) tryCatch(expr, error = identity)

# `nullable(x)` and `option(x)` raise the same condition; returns it.
expect_refused_alike <- function(nullable, option, x) {
  e <- caught(nullable(x))
  o <- caught(option(x))
  expect_s3_class(e, "rust_error")
  expect_identical(class(e), class(o))
  expect_identical(conditionMessage(e), conditionMessage(o))
  expect_identical(e$kind, o$kind)
  expect_identical(e$param, o$param)
  invisible(e)
}

test_that("Nullable<i32> takes NULL, NA and an integer, as Option<i32> does", {
  nullable <- miniextendr:::nullable_arg_i32
  option <- miniextendr:::option_arg_i32
  for (x in list(NULL, NA_integer_, NA, 3L)) {
    expect_identical(nullable(x), option(x))
  }
  expect_identical(nullable(NULL), "none")
  expect_identical(nullable(NA_integer_), "none")
  # A plain `NA` is "no value" too (#2305).
  expect_identical(nullable(NA), "none")
  expect_identical(nullable(3L), "3")
})

test_that("Nullable<i32> refuses a string with Option<i32>'s R-side check", {
  e <- expect_refused_alike(
    miniextendr:::nullable_arg_i32,
    miniextendr:::option_arg_i32,
    "a"
  )
  expect_identical(conditionMessage(e), "'x' must be NULL or integer")
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "x")
  # The R-side check caught it: no Rust type.
  expect_null(e$rust_type)
  for (x in list(1.5, 1:2, TRUE, c(NA, NA))) {
    # A logical value other than a plain `NA` is no integer to either check.
    expect_refused_alike(miniextendr:::nullable_arg_i32, miniextendr:::option_arg_i32, x)
  }
})

test_that("without the R-side check, the Rust conversion words it as Option<i32>", {
  nullable <- miniextendr:::nullable_arg_i32_unchecked
  option <- miniextendr:::option_arg_i32_unchecked
  for (x in list(NULL, NA_integer_, NA, 3L)) {
    expect_identical(nullable(x), option(x))
  }
  expect_identical(nullable(NA), "none")
  for (x in list("a", 1.5, 1:2, TRUE, c(NA, NA))) {
    e <- expect_refused_alike(nullable, option, x)
    expect_match(conditionMessage(e), "^'x' must be NULL or a single integer: ")
    expect_identical(e$kind, "conversion")
  }
  e <- caught(nullable("a"))
  expect_identical(e$rust_type, "Nullable<i32>")
  expect_identical(caught(option("a"))$rust_type, "Option<i32>")
})

test_that("Nullable<String> and Nullable<Vec<i32>> read as their Option twins", {
  for (x in list(NULL, NA_character_, NA, "abc")) {
    expect_identical(
      miniextendr:::nullable_arg_string(x),
      miniextendr:::option_arg_string(x)
    )
  }
  expect_identical(miniextendr:::nullable_arg_string(NA_character_), "none")
  expect_identical(miniextendr:::nullable_arg_string(NA), "none")
  expect_identical(miniextendr:::nullable_arg_string("abc"), "\"abc\"")
  for (x in list(1, c("a", "b"), TRUE)) {
    expect_refused_alike(
      miniextendr:::nullable_arg_string,
      miniextendr:::option_arg_string,
      x
    )
  }
  for (x in list(NULL, integer(), c(1L, NA, 3L))) {
    expect_identical(
      miniextendr:::nullable_arg_i32_vec(x),
      miniextendr:::option_arg_i32_vec(x)
    )
  }
  expect_identical(miniextendr:::nullable_arg_i32_vec(NULL), "none")
  expect_identical(miniextendr:::nullable_arg_i32_vec(1:2), "[1, 2]")
  e <- expect_refused_alike(
    miniextendr:::nullable_arg_i32_vec,
    miniextendr:::option_arg_i32_vec,
    c(1, 2)
  )
  expect_identical(conditionMessage(e), "'x' must be NULL or an integer vector")
})

test_that("Missing<Nullable<i32>> tells an omitted argument from NULL", {
  f <- miniextendr:::nullable_arg_missing_i32
  expect_identical(f(), "absent")
  expect_identical(f(NULL), "none")
  expect_identical(f(NA_integer_), "none")
  expect_identical(f(4L), "4")
  # `Missing<T>` has no R-side type check: the conversion words the refusal.
  e <- caught(f("a"))
  expect_match(conditionMessage(e), "^'x' must be NULL or a single integer: ")
  expect_identical(e$rust_type, "Missing<Nullable<i32>>")
})

test_that("coerce looks through Nullable<i32>", {
  nullable <- miniextendr:::nullable_arg_coerce_i32
  option <- miniextendr:::option_arg_coerce_i32
  for (x in list(NULL, NA, NA_integer_, NA_real_, 2L, 2, TRUE, as.raw(7))) {
    expect_identical(nullable(x), option(x))
  }
  expect_identical(nullable(2), "2")
  expect_identical(nullable(NA_real_), "none")
  for (x in list(2.5, "a")) {
    expect_refused_alike(nullable, option, x)
  }
  expect_identical(
    conditionMessage(caught(nullable(2.5))),
    conditionMessage(caught(option(2.5)))
  )
  f <- miniextendr:::nullable_arg_coerce_missing_f64
  expect_identical(f(), "absent")
  for (x in list(NULL, NA, NA_integer_, NA_real_)) expect_identical(f(x), "none")
  expect_identical(f(2L), "2.0")
  expect_identical(f(TRUE), "1.0")
})

test_that("no_na looks through Nullable<Vec<f64>>", {
  nullable <- miniextendr:::nullable_arg_no_na
  option <- miniextendr:::option_arg_no_na
  expect_identical(nullable(NULL), "none")
  expect_identical(nullable(c(1, 2)), "[1.0, 2.0]")
  expect_identical(nullable(c(1, 2)), option(c(1, 2)))
  e <- expect_refused_alike(nullable, option, c(1, NA))
  expect_identical(conditionMessage(e), "'x' must not contain NA")
  expect_refused_alike(nullable, option, "a")
})

test_that("match_arg looks through Nullable<Mode>", {
  nullable <- miniextendr:::nullable_arg_mode
  option <- miniextendr:::option_arg_mode
  # The formal is NULL, which reads None.
  expect_null(formals(nullable)$mode)
  expect_identical(formals(nullable), formals(option))
  expect_identical(nullable(), "none")
  expect_identical(nullable(NULL), "none")
  for (x in list("Safe", "D", factor("Fast"))) {
    expect_identical(nullable(x), option(x))
  }
  expect_identical(nullable("D"), "Debug")
  for (x in list("nope", c("Fast", "Safe"), 1L)) {
    expect_refused_alike(nullable, option, x)
  }
  # Under Missing, the formal keeps the choices and NULL is None.
  omittable <- miniextendr:::nullable_arg_missing_mode
  expect_identical(
    eval(formals(omittable)$mode),
    c("Fast", "Safe", "Debug")
  )
  expect_identical(omittable(), "absent")
  expect_identical(omittable(NULL), "none")
  expect_identical(omittable("Sa"), "Safe")
})
