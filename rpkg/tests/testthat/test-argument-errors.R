# One argument-error condition whichever side catches a bad argument (#1591):
# src/rust/argument_error_tests.rs. An R-side check (type / length, `no_na`,
# `inherits`, choices) and a failed Rust conversion raise the same classes,
# `kind = "conversion"` and `e$param`; the conversion adds `e$rust_type`.
# rpkg sets no `conversion_error_class`, so the classes are the plain
# `rust_error` layering here; the configured case is in
# tests/cross-package/producer.pkg/tests/testthat/test-conversion-error-class.R.

layers <- c("rust_error", "simpleError", "error", "condition")
caught <- function(expr) tryCatch(expr, error = identity)

# region: the issue's two paths

test_that("the R-side check and the Rust conversion raise the same condition", {
  # A scalar AsNumeric given two values: caught by the R-side length check.
  e1 <- caught(miniextendr:::arg_error_ratio(c(1, 2), 3))
  # An AsNumericVec given a non-number: caught by the Rust conversion.
  e2 <- caught(miniextendr:::arg_error_peak(c("1", "BLQ")))

  expect_identical(class(e1), layers)
  expect_identical(class(e2), class(e1))
  expect_identical(e1$kind, "conversion")
  expect_identical(e2$kind, "conversion")
  expect_identical(e1$param, "num")
  expect_identical(e2$param, "dv")
  expect_identical(conditionMessage(e1), "'num' must have length 1")
  expect_identical(
    conditionMessage(e2),
    "'dv' must be numeric: non-numeric value(s): \"BLQ\" (element 2)"
  )
  # The Rust type is for the package author, kept out of the message; the
  # R-side check has none.
  expect_null(e1$rust_type)
  expect_identical(e2$rust_type, "AsNumericVec")
  # Both name the wrapper's call as written (what stopifnot() reports).
  expect_equal(conditionCall(e1), quote(miniextendr:::arg_error_ratio(c(1, 2), 3)))
  expect_equal(conditionCall(e2), quote(miniextendr:::arg_error_peak(c("1", "BLQ"))))
})

test_that("one handler catches both paths", {
  param_of <- function(expr) tryCatch(expr, rust_error = function(e) e$param)
  expect_identical(param_of(miniextendr:::arg_error_ratio(c(1, 2), 3)), "num")
  expect_identical(param_of(miniextendr:::arg_error_peak(c("1", "BLQ"))), "dv")
  # The base classes stay, so a simpleError handler still sees both.
  expect_s3_class(caught(miniextendr:::arg_error_ratio(1, list())), "simpleError")
  expect_identical(caught(miniextendr:::arg_error_ratio(1, list()))$param, "den")
})

test_that("a scalar AsNumeric that fails its conversion reads in R terms", {
  e <- caught(miniextendr:::arg_error_ratio("BLQ", 3))
  expect_identical(class(e), layers)
  expect_identical(e$param, "num")
  expect_identical(e$rust_type, "AsNumeric")
  expect_identical(
    conditionMessage(e),
    "'num' must be a single number: non-numeric value(s): \"BLQ\" (element 1)"
  )
  expect_identical(miniextendr:::arg_error_ratio("3", 2L), 1.5)
  expect_identical(miniextendr:::arg_error_peak(c("1", "4")), 4)
})

test_that("under call = caller both paths name the caller's call", {
  e1 <- caught(miniextendr:::arg_error_ratio_caller(c(1, 2), 3))
  e2 <- caught(miniextendr:::arg_error_peak_caller(c("1", "BLQ")))
  expect_identical(class(e1), layers)
  expect_identical(class(e2), layers)
  expect_identical(e1$param, "num")
  expect_identical(e2$param, "dv")
  expect_equal(conditionCall(e1), quote(miniextendr:::arg_error_ratio_caller(c(1, 2), 3)))
  expect_equal(conditionCall(e2), quote(miniextendr:::arg_error_peak_caller(c("1", "BLQ"))))
})

# endregion

# region: R wording of the conversion errors

test_that("built-in conversion errors say what the argument must be", {
  e <- caught(miniextendr:::arg_error_int_unchecked("a"))
  expect_identical(conditionMessage(e), "'x' must be a single integer: got character")
  expect_identical(e$rust_type, "i32")
  expect_identical(e$param, "x")
  expect_identical(class(e), layers)
  expect_identical(
    conditionMessage(caught(miniextendr:::arg_error_int_unchecked(1:2))),
    "'x' must be a single integer: got length 2"
  )
  expect_identical(
    conditionMessage(caught(miniextendr:::arg_error_int_unchecked(NA_integer_))),
    "'x' must be a single integer: NA is not allowed"
  )
  expect_identical(
    conditionMessage(caught(miniextendr:::arg_error_flag_unchecked(NA))),
    "'flag' must be TRUE or FALSE: NA is not allowed"
  )
  expect_identical(
    conditionMessage(caught(miniextendr:::arg_error_string_unchecked(1))),
    "'s' must be a single string: got numeric"
  )
  expect_identical(
    conditionMessage(caught(miniextendr:::arg_error_strings_unchecked(1:2))),
    "'xs' must be character: got integer"
  )
  # No SEXPTYPE names and no variant prefix reach the message.
  for (msg in c(
    conditionMessage(caught(miniextendr:::arg_error_int_unchecked("a"))),
    conditionMessage(caught(miniextendr:::arg_error_peak(c("1", "BLQ"))))
  )) {
    expect_no_match(msg, "SXP|invalid value|failed to convert")
  }
})

test_that("a selection of the wrong length for a fixed-size array is an argument error", {
  e <- caught(match_arg_multi_mode_array("Fast"))
  expect_identical(class(e), layers)
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "modes")
  expect_identical(conditionMessage(e), "'modes' must be of length 2: got length 1")
})

# endregion

# region: the checks the author names

test_that("no_na, inherits and choices failures are argument errors", {
  x <- structure(c(1, 2), class = "mx_num")
  expect_identical(miniextendr:::arg_error_named_checks(x, "slow"), "slow: 3")

  e <- caught(miniextendr:::arg_error_named_checks(structure(c(1, NA), class = "mx_num"), "fast"))
  expect_identical(class(e), layers)
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "x")
  expect_identical(conditionMessage(e), "'x' must not contain NA")

  e <- caught(miniextendr:::arg_error_named_checks(c(1, 2), "fast"))
  expect_identical(class(e), layers)
  expect_identical(e$param, "x")
  expect_identical(conditionMessage(e), "'x' must inherit from 'mx_num'")

  e <- caught(miniextendr:::arg_error_named_checks(x, "medium"))
  expect_identical(class(e), layers)
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "mode")
  expect_identical(conditionMessage(e), "'mode' should be one of \"fast\", \"slow\"")
  expect_equal(conditionCall(e), quote(miniextendr:::arg_error_named_checks(x, "medium")))
})

test_that("match_arg and several_ok failures are argument errors", {
  e <- caught(match_arg_multi_mode(c("Fast", "zzz")))
  expect_identical(class(e), layers)
  expect_identical(e$param, "modes")
  expect_match(conditionMessage(e), "^'modes' element 2 \\(\"zzz\"\\) should be one of")
  e <- caught(match_arg_multi_mode(1L))
  expect_identical(e$param, "modes")
  expect_identical(conditionMessage(e), "'modes' must be NULL or a character vector")
})

test_that("method and trait-method checks raise the same condition", {
  h <- ParamCheckHolder$new()
  e <- caught(h$add(list(), 1))
  expect_identical(class(e), layers)
  expect_identical(e$param, "x")
  e <- caught(h$add(structure(list(), class = "mx_obj"), NA_real_))
  expect_identical(class(e), layers)
  expect_identical(e$param, "y")
  expect_identical(conditionMessage(e), "'y' must not be NA")

  obj <- ScalerR6$new(2)
  e <- caught(ScalerR6$Scaler$scale(obj, x_factor = NA_real_))
  expect_identical(class(e), layers)
  expect_identical(e$param, "x_factor")
})

test_that("no_preconditions keeps the named check and the conversion error is the same kind", {
  e <- caught(miniextendr:::param_no_na_no_preconditions(NA_real_))
  expect_identical(class(e), layers)
  expect_identical(e$param, "x")
  e <- caught(miniextendr:::param_no_na_no_preconditions("a"))
  expect_identical(class(e), layers)
  expect_identical(e$param, "x")
  expect_identical(e$rust_type, "f64")
  expect_identical(conditionMessage(e), "'x' must be a single double: got character")
})

# endregion

# region: the author's messages (`message = ` on inherits / no_na)

# src/rust/param_check_tests.rs: a custom message replaces the generated
# `'<p>' must ...` text verbatim; the condition is otherwise the one every
# argument error raises.

mx_model <- function() structure(list(a = 1), class = "mx_model")

test_that("a custom message is the only difference from the generated one", {
  # Two fixtures that differ only in the message.
  e1 <- caught(miniextendr:::param_model_default(list()))
  e2 <- caught(miniextendr:::param_model_custom(list()))
  expect_identical(conditionMessage(e1), "'model' must inherit from 'mx_model' or 'mx_model2'")
  expect_identical(
    conditionMessage(e2),
    "`model` must be an `mx_model` object; create one with `mx_model()`."
  )
  expect_identical(class(e1), layers)
  expect_identical(class(e2), class(e1))
  expect_identical(e1$kind, "conversion")
  expect_identical(e2$kind, e1$kind)
  expect_identical(e1$param, "model")
  expect_identical(e2$param, e1$param)
  expect_null(e2$rust_type)
  expect_equal(conditionCall(e1), quote(miniextendr:::param_model_default(list())))
  expect_equal(conditionCall(e2), quote(miniextendr:::param_model_custom(list())))
  # One message covers both classes, either of which passes.
  expect_identical(miniextendr:::param_model_custom(mx_model()), 1L)
  expect_identical(miniextendr:::param_model_custom(structure(list(), class = "mx_model2")), 0L)
  # The message covers every value that is not a list of the class: the class
  # check runs first, and its message is the type check's too.
  for (model in list(
    1,
    NULL,
    data.frame(),
    structure(list(), class = "other"),
    structure(1, class = "mx_model")
  )) {
    e <- caught(miniextendr:::param_model_custom(model))
    expect_identical(conditionMessage(e), conditionMessage(e2))
    expect_identical(class(e), class(e2))
    expect_identical(e$kind, e2$kind)
    expect_identical(e$param, e2$param)
    expect_null(e$rust_type)
  }
  # Without a message, the type check keeps its generated one.
  expect_identical(
    conditionMessage(caught(miniextendr:::param_model_default(structure(1, class = "mx_model")))),
    "'model' must be a list"
  )
})

test_that("the keyed spelling inherits(class = , message = ) works the same", {
  expect_identical(miniextendr:::param_model_class_key(mx_model()), 1L)
  e <- caught(miniextendr:::param_model_class_key(list()))
  expect_identical(conditionMessage(e), "need an mx_model")
  expect_identical(class(e), layers)
  expect_identical(e$param, "model")
})

test_that("a no_na message reaches R unchanged", {
  e1 <- caught(miniextendr:::param_no_na_scalar(NA_real_))
  e2 <- caught(miniextendr:::param_no_na_custom(NA_real_))
  # Quotes, backticks, a backslash, `%` (no sprintf() on the way), a newline
  # and a non-ASCII character.
  expect_identical(
    conditionMessage(e2),
    "`x` can't be NA: it's \"required\" \\ 100% sure\nsee café()"
  )
  expect_identical(class(e2), class(e1))
  expect_identical(e2$kind, e1$kind)
  expect_identical(e2$param, "x")
  expect_equal(conditionCall(e2), quote(miniextendr:::param_no_na_custom(NA_real_)))
  expect_identical(conditionMessage(caught(miniextendr:::param_no_na_custom(NaN))), conditionMessage(e2))
  expect_identical(miniextendr:::param_no_na_custom(1.5), 1.5)
})

test_that("no_preconditions keeps a check that has a message", {
  e <- caught(miniextendr:::param_no_na_custom_no_preconditions(NA_real_))
  expect_identical(conditionMessage(e), "no NA here")
  expect_identical(class(e), layers)
  expect_identical(e$param, "x")
})

test_that("under call = caller a custom message keeps the caller's call", {
  obj <- structure(list(), class = "mx_obj")
  expect_identical(miniextendr:::param_checks_caller_msg(obj, 1), 1)
  e1 <- caught(miniextendr:::param_checks_caller(list(), 1))
  e2 <- caught(miniextendr:::param_checks_caller_msg(list(), 1))
  expect_identical(conditionMessage(e2), "`x` must be an `mx_obj`")
  # A value that is not a list gets the class message, with the caller's call.
  e3 <- caught(miniextendr:::param_checks_caller_msg(1, 1))
  expect_identical(conditionMessage(e3), "`x` must be an `mx_obj`")
  expect_equal(conditionCall(e3), quote(miniextendr:::param_checks_caller_msg(1, 1)))
  expect_identical(class(e3), class(e1))
  expect_identical(e3$param, e1$param)
  expect_identical(class(e2), class(e1))
  expect_identical(e2$kind, e1$kind)
  expect_identical(e2$param, e1$param)
  expect_equal(conditionCall(e1), quote(miniextendr:::param_checks_caller(list(), 1)))
  expect_equal(conditionCall(e2), quote(miniextendr:::param_checks_caller_msg(list(), 1)))
  e <- caught(miniextendr:::param_checks_caller_msg(obj, NA_real_))
  expect_identical(conditionMessage(e), "`y` must not be NA")
  expect_identical(e$param, "y")
  expect_equal(conditionCall(e), quote(miniextendr:::param_checks_caller_msg(obj, NA_real_)))
})

test_that("impl and trait methods take the method-level messages", {
  h <- ParamCheckHolder$new()
  expect_identical(h$add_checked(structure(list(), class = "mx_other"), 2), 2)
  e1 <- caught(h$add(list(), 1))
  e2 <- caught(h$add_checked(list(), 1))
  expect_identical(conditionMessage(e1), "'x' must inherit from 'mx_obj' or 'mx_other'")
  expect_identical(conditionMessage(e2), "`x` must be an `mx_obj` or an `mx_other`")
  expect_identical(class(e2), class(e1))
  expect_identical(e2$param, e1$param)
  # The message covers the type check too.
  for (x in list(1, structure(1, class = "mx_obj"))) {
    expect_identical(conditionMessage(caught(h$add_checked(x, 1))), conditionMessage(e2))
  }
  e <- caught(h$add_checked(structure(list(), class = "mx_obj"), NA_real_))
  expect_identical(conditionMessage(e), "`y` must be a number, not NA")
  expect_identical(class(e), layers)
  expect_identical(e$param, "y")

  obj <- ScalerS7(2)
  e <- caught(s7_trait_Scaler_scale(obj, x_factor = NA_real_))
  expect_identical(conditionMessage(e), "`x_factor` must be a number, not NA")
  expect_identical(class(e), layers)
  expect_identical(e$param, "x_factor")
  # The fast-path shortcut runs the same check.
  e <- caught(ScalerS7_scale(obj, x_factor = NA_real_))
  expect_identical(conditionMessage(e), "`x_factor` must be a number, not NA")
  expect_identical(e$param, "x_factor")
  expect_equal(s7_trait_Scaler_scale(obj, x_factor = 3), 6)
})

# endregion

# region: omittable and Either choices (#1551)

test_that("an omittable choice raises the argument error", {
  # The choice check behind the `Missing<..>` guard (src/rust/match_arg_tests.rs).
  e <- caught(match_arg_omitted_mode("nope"))
  expect_identical(class(e), layers)
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "mode")
  expect_null(e$rust_type)
  expect_identical(conditionMessage(e), "'mode' should be one of \"Fast\", \"Safe\", \"Debug\"")
  expect_equal(conditionCall(e), quote(match_arg_omitted_mode("nope")))
  # Under call = caller the guarded check keeps the caller's call.
  e <- caught(miniextendr:::call_attr_omitted(mode = "bogus"))
  expect_identical(class(e), layers)
  expect_identical(e$param, "mode")
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_omitted(mode = "bogus")))
})

test_that("an Either choice raises the argument error on both paths", {
  skip_if_not(exists("match_arg_either_route"), "either feature not compiled in")
  # A misspelled choice fails in the R prelude and never reaches the other arm.
  e <- caught(match_arg_either_route("iv"))
  expect_identical(class(e), layers)
  expect_identical(e$param, "route")
  expect_null(e$rust_type)
  # Input that is neither character nor factor goes to the other arm, whose
  # failure is the Rust conversion's argument error.
  e <- caught(match_arg_either_route(1:3))
  expect_identical(class(e), layers)
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "route")
  expect_identical(e$rust_type, "Either<Route, DataFrame>")
  # The refusal names the whole parameter, in the words of its @param line;
  # the data frame arm gives only the reason, in R terms.
  expect_identical(
    conditionMessage(e),
    "'route' must be one of \"oral\", \"bolus\", \"infusion\", or a data frame: got integer"
  )
  expect_equal(conditionCall(e), quote(match_arg_either_route(1:3)))
  route_error <- function(expr) conditionMessage(caught(expr))
  expect_identical(
    route_error(match_arg_either_route(NULL)),
    "'route' must be one of \"oral\", \"bolus\", \"infusion\", or a data frame: got NULL"
  )
  # A list that is not a data frame is not a data frame at all.
  expect_identical(
    route_error(match_arg_either_route(list(1))),
    "'route' must be one of \"oral\", \"bolus\", \"infusion\", or a data frame: got list"
  )
  # A data frame that fails later keeps its own reason.
  expect_identical(
    route_error(match_arg_either_route(structure(list(1), class = "data.frame"))),
    paste0(
      "'route' must be one of \"oral\", \"bolus\", \"infusion\", or a data frame: ",
      "data.frame has no column names"
    )
  )
  # several_ok, and `NULL` under an `Option` layer.
  expect_identical(
    route_error(match_arg_either_routes(1:3)),
    "'routes' must be one or more of \"oral\", \"bolus\", \"infusion\", or a data frame: got integer"
  )
  expect_identical(
    route_error(match_arg_either_route_optional(1:3)),
    paste0(
      "'maybe_route' must be one of \"oral\", \"bolus\", \"infusion\", ",
      "a data frame, or NULL: got integer"
    )
  )
  # A method takes the same branch.
  e <- caught(EitherRoutePlanner$new()$plan(1:3))
  expect_identical(e$param, "route")
  expect_identical(
    conditionMessage(e),
    "'route' must be one of \"oral\", \"bolus\", \"infusion\", or a data frame: got integer"
  )
  # `choices(...)` names its list and the other arm's @param noun.
  e <- caught(choices_either_level(TRUE))
  expect_identical(class(e), layers)
  expect_identical(e$param, "level")
  expect_identical(e$rust_type, "Either<String, f64>")
  expect_identical(
    conditionMessage(e),
    "'level' must be one of \"low\", \"mid\", \"high\", or a number: got logical"
  )
  # `f64` stays storage-strict: an integer is not a number here.
  expect_identical(
    route_error(choices_either_level(1L)),
    "'level' must be one of \"low\", \"mid\", \"high\", or a number: got integer"
  )
})

# endregion

# region: the S7 fallback receiver check

test_that("a class_any method given a non-S7 receiver raises the argument error", {
  # `describe_any` is registered for S7::class_any (src/rust/s7_tests.rs); the
  # receiver check runs before the pointer is taken.
  e <- caught(describe_any(1L))
  expect_identical(class(e), layers)
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "x")
  expect_identical(conditionMessage(e), "'x' must be an S7 object, got integer")
  expect_null(e$rust_type)
  expect_true(is.call(conditionCall(e)))
  expect_identical(describe_any(S7Strict(123L)), "S7Strict with value 123")
})

# endregion

# region: per-element failures, R-facing expectations, strict and no_na wording

test_that("a vector conversion lists every bad element, 1-based and batched", {
  # c(NA, 70000, 1, -5) passes the R-side whole-number check; the conversion
  # refuses the NA and the two values outside u16, each reason once with the
  # positions that failed with it.
  e <- caught(miniextendr:::arg_error_u16s(c(NA, 70000, 1, -5)))
  expect_identical(class(e), layers)
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "counts")
  expect_identical(e$rust_type, "Vec<u16>")
  expect_identical(
    conditionMessage(e),
    paste0(
      "'counts' must be integer or whole-number numeric: ",
      "NA is not allowed (element 1); value out of range (elements 2, 4)"
    )
  )
  # No 0-based index and no Rust type in the text.
  expect_no_match(conditionMessage(e), "index|Vec<|conversion failed")
  # Past ten failures the rest are counted.
  expect_identical(
    conditionMessage(caught(miniextendr:::arg_error_u16s(rep(70000, 12)))),
    paste0(
      "'counts' must be integer or whole-number numeric: ",
      "value out of range (elements 1, 2, 3, 4, 5, 6, 7, 8, 9, 10); and 2 more"
    )
  )
  expect_identical(miniextendr:::arg_error_u16s(c(1, 2)), 3L)
})

test_that("Either, AsFromStr and tuple arguments say what they accept", {
  e <- caught(either_int_or_str(1:2))
  expect_identical(class(e), layers)
  expect_identical(e$rust_type, "Either<i32, String>")
  # The integer branch took the type and refused the length: that reason.
  expect_identical(
    conditionMessage(e),
    "'value' must be a single integer or a single string: got length 2"
  )
  expect_identical(
    conditionMessage(caught(either_int_or_str(list()))),
    "'value' must be a single integer or a single string: got list"
  )

  # A data frame argument given a list that is not a data frame.
  e <- caught(df_option_scalar_none_count(list()))
  expect_identical(e$param, "df")
  expect_identical(conditionMessage(e), "'df' must be a data frame: got list")
  # Behind a newtype the macro knows nothing about, the class error says what
  # the value should have been.
  e <- caught(miniextendr:::frame_arg(1))
  expect_identical(e$param, "x")
  expect_identical(conditionMessage(e), "'x' must be a data frame: got numeric")

  e <- caught(miniextendr:::test_fromstr_ip(1))
  expect_identical(e$param, "addr")
  expect_identical(conditionMessage(e), "'addr' must be a single string: got numeric")
  expect_identical(
    conditionMessage(caught(miniextendr:::test_fromstr_ip("x"))),
    "'addr' must be a single string: \"x\": invalid IP address syntax"
  )

  e <- caught(miniextendr:::arg_error_pair(list("a", 1L)))
  expect_identical(class(e), layers)
  expect_identical(e$param, "pair")
  expect_identical(e$rust_type, "(i32, String)")
  expect_identical(
    conditionMessage(e),
    paste0(
      "'pair' must be a list of length 2: expected integer, got character (element 1); ",
      "expected character, got integer (element 2)"
    )
  )
  expect_identical(
    conditionMessage(caught(miniextendr:::arg_error_pair(list(1L)))),
    "'pair' must be a list of length 2: got length 1"
  )
  expect_identical(miniextendr:::arg_error_pair(list(3L, "x")), "3:x")
})

test_that("an Either whose arms both refuse the kind of value names both", {
  skip_if_not(miniextendr_has_feature("either"), "either feature not compiled in")
  refusal <- function(expr) conditionMessage(caught(expr))
  # Both arms known to the macro: the static prefix, and the reason for a list
  # that is not a data frame is its type.
  expect_identical(miniextendr:::either_num_or_df(c(1, 2)), "num:2")
  expect_identical(
    refusal(miniextendr:::either_num_or_df(list(4, 8, 12))),
    "'x' must be double or a data frame: got list"
  )
  expect_identical(
    refusal(miniextendr:::either_num_or_df(NULL)),
    "'x' must be double or a data frame: got NULL"
  )
  # An opaque arm: the expectation comes from the two arms' errors.
  expect_identical(miniextendr:::either_opaque_or_df(c("1", "2")), "num:2")
  expect_identical(
    refusal(miniextendr:::either_opaque_or_df(list(4, 8, 12))),
    "'x' must be numeric or a data frame: got list"
  )
  expect_identical(
    refusal(miniextendr:::either_opaque_or_df(NULL)),
    "'x' must be numeric or a data frame: got NULL"
  )
  # A `match_arg` enum decoded by its own `TryFromSexp` (no `match_arg`
  # attribute) refuses a value that is not a string by its kind too.
  expect_identical(miniextendr:::either_route_or_number("oral"), "Oral")
  expect_identical(miniextendr:::either_route_or_number(2.5), "number:2.5")
  e <- caught(miniextendr:::either_route_or_number(TRUE))
  expect_identical(class(e), layers)
  expect_identical(e$param, "x")
  expect_identical(e$rust_type, "Either<Route, f64>")
  expect_identical(
    conditionMessage(e),
    "'x' must be one of \"oral\", \"bolus\", \"infusion\", or numeric: got logical"
  )
  expect_identical(
    refusal(miniextendr:::either_route_or_number(1L)),
    "'x' must be one of \"oral\", \"bolus\", \"infusion\", or numeric: got integer"
  )
  expect_identical(
    refusal(miniextendr:::either_route_or_number(list(1))),
    "'x' must be one of \"oral\", \"bolus\", \"infusion\", or numeric: got list"
  )
  # A string that matches no choice got further: the choice's reason.
  expect_identical(
    refusal(miniextendr:::either_route_or_number("zzz")),
    "invalid 'x' argument: expected one of \"oral\", \"bolus\", \"infusion\", got \"zzz\""
  )
  # Only the number arm got as far as the length.
  expect_identical(
    refusal(miniextendr:::either_route_or_number(c(1, 2))),
    "invalid 'x' argument: expected length 1, got length 2"
  )
  # A data frame that fails later keeps its own reason.
  expect_identical(
    refusal(miniextendr:::either_num_or_df(structure(list(1), class = "data.frame"))),
    "'x' must be double or a data frame: data.frame has no column names"
  )
  # A nested `Either` whose arms both refused the value is one such arm.
  expect_identical(miniextendr:::either_flag_or_route_or_number(TRUE), "flag:true")
  expect_identical(miniextendr:::either_flag_or_route_or_number("bolus"), "Bolus")
  expect_identical(
    refusal(miniextendr:::either_flag_or_route_or_number(list())),
    "'x' must be logical or one of \"oral\", \"bolus\", \"infusion\", or numeric: got list"
  )
  # Two choice lists read as one.
  expect_identical(miniextendr:::either_route_or_mode("Safe"), "mode:Safe")
  expect_identical(
    refusal(miniextendr:::either_route_or_mode(1)),
    "'x' must be one of \"oral\", \"bolus\", \"infusion\", \"Fast\", \"Safe\", \"Debug\": got numeric"
  )
})

test_that("strict input rejections are argument errors, batched over a vector", {
  e <- caught(miniextendr:::arg_error_strict_inputs(TRUE, 1L))
  expect_identical(class(e), layers)
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "n")
  expect_identical(e$rust_type, "i64")
  expect_identical(conditionMessage(e), "'n' must be a single whole number: got logical")

  e <- caught(miniextendr:::arg_error_strict_inputs(1L, c(1, NA, 1e300)))
  expect_identical(e$param, "ids")
  expect_identical(e$rust_type, "Vec<i64>")
  expect_identical(
    conditionMessage(e),
    paste0(
      "'ids' must be integer or whole-number numeric: ",
      "NA is not allowed (element 2); value out of range (element 3)"
    )
  )
  expect_identical(miniextendr:::arg_error_strict_inputs(2, c(1L, 2L)), "2:2")
})

test_that("no_na on a vector marker says 'must not contain NA'", {
  e <- caught(miniextendr:::arg_error_no_na_peak(c("1", NA)))
  expect_identical(class(e), layers)
  expect_identical(e$param, "obs")
  expect_identical(conditionMessage(e), "'obs' must not contain NA")
  expect_null(e$rust_type)
  expect_identical(miniextendr:::arg_error_no_na_peak(c("1", "4")), 4)
})

test_that("no_na's Rust check after the conversion raises the R guard's condition", {
  # `NA` is refused by the R guard; the text "NA" passes `anyNA()` and is
  # refused after the conversion, which reads it as missing.
  e_r <- caught(miniextendr:::arg_error_no_na_peak(c("1", NA)))
  e_rust <- caught(miniextendr:::arg_error_no_na_peak(c("1", "NA")))
  expect_identical(class(e_rust), class(e_r))
  expect_identical(e_rust$kind, e_r$kind)
  expect_identical(e_rust$param, e_r$param)
  expect_identical(conditionMessage(e_rust), conditionMessage(e_r))
  expect_identical(names(e_rust), names(e_r))
  expect_identical(names(e_rust), c("message", "call", "kind", "param"))
  expect_null(e_rust$rust_type)
  # Both sides report the call in the same form, as the test wrote it.
  expect_equal(conditionCall(e_rust), quote(miniextendr:::arg_error_no_na_peak(c("1", "NA"))))
  expect_equal(conditionCall(e_r), quote(miniextendr:::arg_error_no_na_peak(c("1", NA))))
  # Under `call = caller` both name the caller.
  e <- caught(miniextendr:::param_no_na_number_caller(""))
  expect_identical(class(e), layers)
  expect_equal(conditionCall(e), quote(miniextendr:::param_no_na_number_caller("")))
})

test_that("a match_arg enum's conversion error names its choices", {
  # No match_arg attribute, so no R-side match.arg(): the conversion refuses
  # the value, and the error says what it must be.
  e <- caught(miniextendr:::arg_error_plain_mode("zzz"))
  expect_identical(class(e), layers)
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "speed")
  expect_identical(e$rust_type, "Mode")
  expect_identical(
    conditionMessage(e),
    "'speed' must be one of \"Fast\", \"Safe\", \"Debug\": got \"zzz\""
  )
  expect_identical(
    conditionMessage(caught(miniextendr:::arg_error_plain_mode(1))),
    "'speed' must be one of \"Fast\", \"Safe\", \"Debug\": got numeric"
  )
  expect_identical(
    conditionMessage(caught(miniextendr:::arg_error_plain_mode(NA_character_))),
    "'speed' must be one of \"Fast\", \"Safe\", \"Debug\": NA is not allowed"
  )
  expect_identical(miniextendr:::arg_error_plain_mode("Sa"), "Safe")
})

# endregion
