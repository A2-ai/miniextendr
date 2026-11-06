# A plain `NA`, R's untyped missing value, given to an optional scalar
# argument reads as `None`, as `NULL` does (#2305):
# src/rust/plain_na_argument_tests.rs. Every other value keeps its check.

caught <- function(expr) tryCatch(expr, error = identity)

fixture <- function(name) getFromNamespace(name, "miniextendr")

# The C entry point, called without the R wrapper and its checks.
native <- function(name) {
  getFromNamespace(paste0("C_miniextendr_", name), "miniextendr")
}

# Each optional scalar fixture, with values it refuses. `c(NA, NA)` and
# `logical(0)` are logical but no plain `NA`, so every fixture refuses them.
optional_scalars <- list(
  plain_na_i32 = list(c(NA, NA), logical(0), TRUE, 2),
  plain_na_f64 = list(c(NA, NA), logical(0), TRUE, "a"),
  plain_na_string = list(c(NA, NA), logical(0), TRUE, 1),
  plain_na_str = list(c(NA, NA), logical(0), TRUE, 1),
  plain_na_bool = list(c(NA, NA), logical(0), 1L, "a"),
  plain_na_raw = list(c(NA, NA), logical(0), TRUE, 1L),
  plain_na_complex = list(c(NA, NA), logical(0), TRUE, 1),
  plain_na_i64 = list(c(NA, NA), logical(0), "a"),
  plain_na_u32 = list(c(NA, NA), logical(0), "a"),
  plain_na_i32_nullable = list(c(NA, NA), logical(0), TRUE, 2),
  plain_na_f64_nullable = list(c(NA, NA), logical(0), TRUE, "a"),
  plain_na_string_nullable = list(c(NA, NA), logical(0), TRUE, 1),
  plain_na_bool_nullable = list(c(NA, NA), logical(0), 1L, "a"),
  plain_na_f64_unchecked = list(c(NA, NA), logical(0), TRUE, "a"),
  plain_na_string_unchecked = list(c(NA, NA), logical(0), TRUE, 1),
  plain_na_bool_unchecked = list(c(NA, NA), logical(0), "a")
)

test_that("a plain NA reads as None, as NULL does", {
  for (name in names(optional_scalars)) {
    f <- fixture(name)
    expect_identical(f(NULL), "none", info = name)
    expect_identical(f(NA), "none", info = name)
    # Attributes are not looked at: a named or classed `NA` is a plain `NA`.
    expect_identical(f(c(a = NA)), "none", info = name)
    expect_identical(f(structure(NA, class = "x")), "none", info = name)
  }
})

test_that("a value still reads as Some", {
  expect_identical(fixture("plain_na_i32")(3L), "3")
  expect_identical(fixture("plain_na_f64")(1.5), "1.5")
  expect_identical(fixture("plain_na_string")("a"), "\"a\"")
  expect_identical(fixture("plain_na_str")("a"), "\"a\"")
  expect_identical(fixture("plain_na_bool")(TRUE), "true")
  expect_identical(fixture("plain_na_bool")(FALSE), "false")
  expect_identical(fixture("plain_na_raw")(as.raw(7)), "7")
  expect_identical(fixture("plain_na_complex")(1+2i), "(1.0, 2.0)")
  expect_identical(fixture("plain_na_i64")(5), "5")
  expect_identical(fixture("plain_na_u32")(5L), "5")
  # A typed `NA` still reads as None where the type has one.
  expect_identical(fixture("plain_na_i32")(NA_integer_), "none")
  expect_identical(fixture("plain_na_f64")(NA_real_), "none")
  expect_identical(fixture("plain_na_string")(NA_character_), "none")
})

test_that("every other value keeps its check and its message", {
  for (name in names(optional_scalars)) {
    f <- fixture(name)
    for (x in optional_scalars[[name]]) {
      e <- caught(f(x))
      expect_s3_class(e, "rust_error")
      expect_match(conditionMessage(e), "^'x' must be NULL", info = name)
      expect_identical(e$param, "x", info = name)
    }
  }
  # The R-side check words the refusal as before.
  expect_identical(
    conditionMessage(caught(fixture("plain_na_i32")(c(NA, NA)))),
    "'x' must be NULL or integer"
  )
  expect_identical(
    conditionMessage(caught(fixture("plain_na_i32")(TRUE))),
    "'x' must be NULL or integer"
  )
  # A double is no integer without `coerce`.
  expect_identical(
    conditionMessage(caught(fixture("plain_na_i32")(2))),
    "'x' must be NULL or integer"
  )
  # Without the R-side check, the Rust conversion names the type.
  expect_match(
    conditionMessage(caught(fixture("plain_na_f64_unchecked")(c(NA, NA)))),
    "^'x' must be NULL or a single double: "
  )
  expect_match(
    conditionMessage(caught(fixture("plain_na_string_unchecked")(TRUE))),
    "^'x' must be NULL or a single string: "
  )
})

test_that("the Nullable twins read as their Option fixtures", {
  twins <- c(
    plain_na_i32_nullable = "plain_na_i32",
    plain_na_f64_nullable = "plain_na_f64",
    plain_na_string_nullable = "plain_na_string",
    plain_na_bool_nullable = "plain_na_bool"
  )
  for (name in names(twins)) {
    nullable <- fixture(name)
    option <- fixture(twins[[name]])
    for (x in list(NULL, NA, c(a = NA))) {
      expect_identical(nullable(x), option(x), info = name)
    }
    for (x in optional_scalars[[name]]) {
      expect_identical(
        conditionMessage(caught(nullable(x))),
        conditionMessage(caught(option(x))),
        info = name
      )
    }
  }
})

test_that("the Rust conversion alone reads a plain NA as None", {
  # `.Call()` skips the wrapper and its checks: the conversion decides.
  names <- c(
    names(optional_scalars),
    "plain_na_mode", "plain_na_mode_nullable", "plain_na_color"
  )
  for (name in names) {
    expect_identical(.Call(native(name), NULL, NA), "none", info = name)
    expect_identical(.Call(native(name), NULL, NULL), "none", info = name)
    # A refused value comes back as the tagged conversion condition.
    refused <- .Call(native(name), NULL, c(NA, NA))
    expect_identical(refused$kind, "conversion", info = name)
  }
  # The unchecked fixtures agree with the checked ones.
  for (type in c("f64", "string", "bool")) {
    checked <- fixture(paste0("plain_na_", type))
    unchecked <- fixture(paste0("plain_na_", type, "_unchecked"))
    for (x in list(NULL, NA, c(a = NA))) {
      expect_identical(unchecked(x), checked(x), info = type)
    }
  }
})

test_that("an optional match_arg choice reads a plain NA as None", {
  for (name in c("plain_na_mode", "plain_na_mode_nullable")) {
    f <- fixture(name)
    expect_identical(f(), "none", info = name)
    expect_identical(f(NULL), "none", info = name)
    expect_identical(f(NA), "none", info = name)
    expect_identical(f("Safe"), "Safe", info = name)
    expect_identical(f("D"), "Debug", info = name)
    # A typed `NA` is still matched, and refused.
    expect_s3_class(caught(f(NA_character_)), "rust_error")
    expect_s3_class(caught(f(c(NA, NA))), "rust_error")
    expect_s3_class(caught(f("nope")), "rust_error")
  }
  expect_identical(
    conditionMessage(caught(fixture("plain_na_mode_nullable")(c(NA, NA)))),
    conditionMessage(caught(fixture("plain_na_mode")(c(NA, NA))))
  )
  # Under Missing, an omitted argument is still Absent.
  f <- fixture("plain_na_missing_mode")
  expect_identical(f(), "absent")
  expect_identical(f(NULL), "none")
  expect_identical(f(NA), "none")
  expect_identical(f("Fast"), "Fast")
  # A choices(...) string too.
  f <- fixture("plain_na_color")
  expect_identical(f(NULL), "none")
  expect_identical(f(NA), "none")
  expect_identical(f("gr"), "\"green\"")
  expect_s3_class(caught(f(NA_character_)), "rust_error")
})

test_that("optional vectors still read a plain NA as a value", {
  # `Option<Vec<i32>>` refuses a logical vector, `NA` included.
  e <- caught(miniextendr:::option_arg_i32_vec(NA))
  expect_s3_class(e, "rust_error")
  expect_identical(conditionMessage(e), "'x' must be NULL or an integer vector")
  expect_identical(
    conditionMessage(caught(miniextendr:::nullable_arg_i32_vec(NA))),
    conditionMessage(e)
  )
})
