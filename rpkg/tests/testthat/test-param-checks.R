# Per-parameter `inherits` / `not_inherits` / `no_na` checks
# (src/rust/param_check_tests.rs).

mx_obj <- function() structure(list(a = 1, b = 2), class = "mx_obj")

test_that("inherits = \"cls\" accepts the class and refuses anything else", {
  expect_identical(miniextendr:::param_inherits_one(mx_obj()), 2L)
  expect_error(
    miniextendr:::param_inherits_one(list(a = 1)),
    "'x' must inherit from 'mx_obj'",
    fixed = TRUE
  )
  # The class check runs first.
  expect_error(miniextendr:::param_inherits_one(1), "'x' must inherit from 'mx_obj'", fixed = TRUE)
  # Without a message, the type check keeps its generated one.
  expect_error(
    miniextendr:::param_inherits_one(structure(1, class = "mx_obj")),
    "'x' must be a list",
    fixed = TRUE
  )
})

test_that("inherits(\"a\", \"b\") accepts any of the classes", {
  expect_true(miniextendr:::param_inherits_any(structure(1, class = "mx_a")))
  expect_true(miniextendr:::param_inherits_any(structure(1, class = c("sub", "mx_b"))))
  expect_error(
    miniextendr:::param_inherits_any(1),
    "'x' must inherit from 'mx_a' or 'mx_b'",
    fixed = TRUE
  )
})

test_that("inherits on an Option<List> lets NULL through", {
  expect_false(miniextendr:::param_inherits_optional())
  expect_false(miniextendr:::param_inherits_optional(NULL))
  expect_true(miniextendr:::param_inherits_optional(mx_obj()))
  expect_error(
    miniextendr:::param_inherits_optional(list()),
    "'x' must inherit from 'mx_obj'",
    fixed = TRUE
  )
  expect_error(
    miniextendr:::param_inherits_optional(1),
    "'x' must inherit from 'mx_obj'",
    fixed = TRUE
  )
})

test_that("no_na refuses NA and NaN on a double scalar", {
  expect_identical(miniextendr:::param_no_na_scalar(1.5), 1.5)
  expect_error(miniextendr:::param_no_na_scalar(NA_real_), "'x' must not be NA", fixed = TRUE)
  expect_error(miniextendr:::param_no_na_scalar(NaN), "'x' must not be NA", fixed = TRUE)
  # Without `no_na` the same type accepts NA.
  expect_identical(miniextendr:::test_f64_identity(NA_real_), NA_real_)
})

test_that("no_na refuses a vector containing NA", {
  expect_identical(miniextendr:::param_no_na_vec(c(1, 2)), 3)
  expect_identical(miniextendr:::param_no_na_vec(double()), 0)
  expect_error(
    miniextendr:::param_no_na_vec(c(1, NA)),
    "'x' must not contain NA",
    fixed = TRUE
  )
})

test_that("no_na on a Missing<f64> lets an omitted argument through", {
  expect_false(miniextendr:::param_no_na_missing())
  expect_true(miniextendr:::param_no_na_missing(1))
  expect_error(miniextendr:::param_no_na_missing(NA_real_), "'x' must not be NA", fixed = TRUE)
})

test_that("both checks on one parameter run the class first, then NA", {
  x <- structure(c(1, 2), class = "mx_num")
  expect_identical(miniextendr:::param_checks_both(x), 3)
  expect_error(
    miniextendr:::param_checks_both(structure(c(1, NA), class = "mx_num")),
    "'x' must not contain NA",
    fixed = TRUE
  )
  expect_error(miniextendr:::param_checks_both(c(1, NA)), "'x' must inherit from 'mx_num'", fixed = TRUE)
  expect_error(miniextendr:::param_checks_both("a"), "'x' must inherit from 'mx_num'", fixed = TRUE)
  expect_error(
    miniextendr:::param_checks_both(structure("a", class = "mx_num")),
    "'x' must be double",
    fixed = TRUE
  )
  expect_error(miniextendr:::param_checks_both(c(1, 2)), "'x' must inherit from 'mx_num'", fixed = TRUE)
})

test_that("no_preconditions keeps no_na but drops the type checks", {
  expect_identical(miniextendr:::param_no_na_no_preconditions(2), 2)
  expect_error(miniextendr:::param_no_na_no_preconditions(NA_real_), "'x' must not be NA", fixed = TRUE)
  # "a" reaches Rust: the conversion error, not an R-side type check.
  e <- tryCatch(miniextendr:::param_no_na_no_preconditions("a"), error = identity)
  expect_s3_class(e, "rust_error")
  expect_false(grepl("must be double", conditionMessage(e), fixed = TRUE))
})

test_that("under call = caller the checks are attributed to the caller", {
  obj <- mx_obj()
  expect_identical(miniextendr:::param_checks_caller(obj, 1), 1)
  e <- tryCatch(miniextendr:::param_checks_caller(list(), 1), error = identity)
  expect_identical(conditionMessage(e), "'x' must inherit from 'mx_obj'")
  expect_equal(conditionCall(e), quote(miniextendr:::param_checks_caller(list(), 1)))
  e <- tryCatch(miniextendr:::param_checks_caller(obj, NA_real_), error = identity)
  expect_identical(conditionMessage(e), "'y' must not be NA")
  expect_equal(conditionCall(e), quote(miniextendr:::param_checks_caller(obj, NA_real_)))
})

test_that("impl methods take inherits(...) / no_na(...) at method level", {
  h <- ParamCheckHolder$new()
  expect_identical(h$add(mx_obj(), 2), 2)
  expect_identical(h$add(structure(list(), class = "mx_other"), 3), 5)
  expect_error(h$add(list(), 1), "'x' must inherit from 'mx_obj' or 'mx_other'", fixed = TRUE)
  expect_error(h$add(1, 1), "'x' must inherit from 'mx_obj' or 'mx_other'", fixed = TRUE)
  expect_error(h$add(mx_obj(), NA_real_), "'y' must not be NA", fixed = TRUE)
})

test_that("trait methods take no_na(...) at method level", {
  obj <- ScalerR6$new(2)
  expect_equal(ScalerR6$Scaler$scale(obj, x_factor = 3), 6)
  expect_error(ScalerR6$Scaler$scale(obj, x_factor = NA_real_), "'x_factor' must not be NA", fixed = TRUE)
})

# region: where an inherits message reaches
#
# The class check runs before the type checks, and an `inherits` message also
# replaces the type checks' messages. Without an R type check (under
# `no_preconditions`, or for `Missing<T>`), a value of the right class that
# the Rust conversion refuses gets the conversion's message.

model_msg <- "`model` must be an `mx_model` object; create one with `mx_model()`."

test_that("no_preconditions keeps the class check; the Rust conversion judges the type", {
  expect_identical(
    miniextendr:::param_model_custom_no_preconditions(structure(list(1), class = "mx_model")),
    1L
  )
  e <- tryCatch(miniextendr:::param_model_custom_no_preconditions(1), error = identity)
  expect_identical(conditionMessage(e), model_msg)
  e <- tryCatch(
    miniextendr:::param_model_custom_no_preconditions(structure(1, class = "mx_model")),
    error = identity
  )
  expect_s3_class(e, "rust_error")
  expect_identical(e$rust_type, "List")
  expect_false(grepl(model_msg, conditionMessage(e), fixed = TRUE))
})

test_that("on a Missing<NamedList> the class check is the only R guard", {
  expect_false(miniextendr:::param_model_custom_missing())
  expect_true(miniextendr:::param_model_custom_missing(structure(list(a = 1), class = "mx_model")))
  e <- tryCatch(miniextendr:::param_model_custom_missing(1), error = identity)
  expect_identical(conditionMessage(e), model_msg)
  e <- tryCatch(
    miniextendr:::param_model_custom_missing(structure(1, class = "mx_model")),
    error = identity
  )
  expect_s3_class(e, "rust_error")
  expect_identical(e$rust_type, "Missing<NamedList>")
  expect_false(grepl(model_msg, conditionMessage(e), fixed = TRUE))
})

test_that("an inherits message covers a scalar's storage and length checks", {
  unit_msg <- "`x` must be an `mx_unit` number"
  expect_identical(miniextendr:::param_classed_scalar(structure(2.5, class = "mx_unit")), 2.5)
  for (x in list(2.5, structure(c(1, 2), class = "mx_unit"), structure(1L, class = "mx_unit"))) {
    e <- tryCatch(miniextendr:::param_classed_scalar(x), error = identity)
    expect_identical(conditionMessage(e), unit_msg)
    expect_identical(e$param, "x")
  }
})

# endregion

# region: no_na on the reading markers
#
# `AsNumeric*` / `AsCharacter*` read inputs as missing that `anyNA()` passes
# (the text "NA", blank strings, a factor `NA` level). The R guard refuses R's
# `NA` first; the C wrapper checks the converted value after the conversion
# and raises the same condition.

caught_msg <- function(expr) conditionMessage(tryCatch(expr, error = identity))

test_that("no_na on AsNumeric accepts numbers in every form", {
  for (x in list(2.5, "2.5", " 2.5 ", factor("2.5"))) {
    expect_identical(miniextendr:::param_no_na_number(x), 2.5)
  }
  expect_identical(miniextendr:::param_no_na_number(Inf), Inf)
})

test_that("no_na on AsNumeric refuses R's NA and NaN in the R guard", {
  for (x in list(NA_real_, NA_character_, factor(NA), NaN, NA)) {
    expect_identical(caught_msg(miniextendr:::param_no_na_number(x)), "'x' must not be NA")
  }
})

test_that("no_na on AsNumeric refuses what the marker reads as missing", {
  for (x in list("NA", " NA ", "", "   ", "　")) {
    expect_identical(caught_msg(miniextendr:::param_no_na_number(x)), "'x' must not be NA")
  }
  for (x in list(factor("NA"), factor(""), factor(NA, exclude = NULL))) {
    expect_identical(caught_msg(miniextendr:::param_no_na_number(x)), "'x' must not be NA")
  }
})

test_that("no_na on AsNumeric refuses NaN read from text, as numeric NaN is", {
  expect_identical(caught_msg(miniextendr:::param_no_na_number("NaN")), "'x' must not be NA")
  expect_identical(caught_msg(miniextendr:::param_no_na_number(factor("NaN"))), "'x' must not be NA")
})

test_that("a value that is not a number stays the conversion error", {
  expect_identical(
    caught_msg(miniextendr:::param_no_na_number("n/a")),
    "'x' must be a single number: non-numeric value(s): \"n/a\" (element 1)"
  )
  expect_match(
    caught_msg(miniextendr:::param_no_na_numbers(c("NA", "n/a"))),
    "non-numeric value(s): \"n/a\" (element 2)",
    fixed = TRUE
  )
})

test_that("no_na on AsNumericVec says 'must not contain NA'", {
  expect_identical(miniextendr:::param_no_na_numbers(c("1", " 2 ", "0x10")), 0L)
  expect_identical(miniextendr:::param_no_na_numbers(character(0)), 0L)
  for (x in list(c("1", "NA", "", " "), factor(c("1", "")), c("1", "NaN"), c(1, NA))) {
    expect_identical(caught_msg(miniextendr:::param_no_na_numbers(x)), "'x' must not contain NA")
  }
})

test_that("no_na on Option<AsNumeric> lets NULL through as not given", {
  expect_identical(miniextendr:::param_no_na_number_opt(), "NULL")
  expect_identical(miniextendr:::param_no_na_number_opt(NULL), "NULL")
  expect_identical(miniextendr:::param_no_na_number_opt("3"), "3")
  for (x in list("NA", "", NA)) {
    expect_identical(caught_msg(miniextendr:::param_no_na_number_opt(x)), "'x' must not be NA")
  }
})

test_that("no_na on Missing<AsNumeric> lets an omitted argument through", {
  expect_identical(miniextendr:::param_no_na_number_missing(), "absent")
  expect_identical(miniextendr:::param_no_na_number_missing(" 2 "), "2")
  expect_identical(caught_msg(miniextendr:::param_no_na_number_missing("")), "'x' must not be NA")
})

test_that("no_preconditions drops the type checks but keeps both no_na checks", {
  expect_identical(miniextendr:::param_no_na_numbers_no_preconditions(c("1", "2")), 0L)
  expect_identical(
    caught_msg(miniextendr:::param_no_na_numbers_no_preconditions(c(1, NA))),
    "'x' must not contain NA"
  )
  expect_identical(
    caught_msg(miniextendr:::param_no_na_numbers_no_preconditions(c("1", "NA"))),
    "'x' must not contain NA"
  )
})

test_that("no_na on AsCharacter refuses an NA the conversion produces", {
  expect_identical(miniextendr:::param_no_na_label("NA"), "NA")
  expect_identical(miniextendr:::param_no_na_label(factor("a")), "a")
  expect_identical(
    caught_msg(miniextendr:::param_no_na_label(factor(NA, exclude = NULL))),
    "'x' must not be NA"
  )
})

test_that("a custom no_na message is the same text on both sides", {
  from_r <- caught_msg(miniextendr:::param_no_na_number_custom(NA))
  from_rust <- caught_msg(miniextendr:::param_no_na_number_custom("NA"))
  expect_identical(from_rust, from_r)
  expect_identical(from_rust, caught_msg(miniextendr:::param_no_na_custom(NA_real_)))
})

test_that("on the worker path the check runs before dispatch", {
  skip_if_not(miniextendr_has_feature("worker-thread"), "worker-thread feature off")
  expect_identical(miniextendr:::param_no_na_numbers_worker(c("1", "2")), 0L)
  expect_identical(
    caught_msg(miniextendr:::param_no_na_numbers_worker(c("1", ""))),
    "'x' must not contain NA"
  )
})

test_that("under call = caller the Rust check names the caller", {
  expect_identical(miniextendr:::param_no_na_number_caller(" 4 "), 4)
  e <- tryCatch(miniextendr:::param_no_na_number_caller("NA"), error = identity)
  expect_identical(conditionMessage(e), "'x' must not be NA")
  expect_equal(conditionCall(e), quote(miniextendr:::param_no_na_number_caller("NA")))
  e <- tryCatch(miniextendr:::param_no_na_number_caller(NA), error = identity)
  expect_equal(conditionCall(e), quote(miniextendr:::param_no_na_number_caller(NA)))
})

test_that("no_na follows the type through an alias and a derived newtype", {
  expect_identical(miniextendr:::param_no_na_alias("1.5"), 1.5)
  expect_identical(caught_msg(miniextendr:::param_no_na_alias("NA")), "'x' must not be NA")
  expect_identical(miniextendr:::param_no_na_newtype(factor("7")), 7)
  expect_identical(caught_msg(miniextendr:::param_no_na_newtype("")), "'x' must not be NA")
  expect_identical(miniextendr:::param_no_na_newtype_opt(), "NULL")
  expect_identical(miniextendr:::param_no_na_newtype_opt("2"), "2")
  expect_identical(
    caught_msg(miniextendr:::param_no_na_newtype_opt("NaN")),
    "'x' must not be NA"
  )
})

test_that("no_na on a map of markers reads each top-level value", {
  expect_identical(miniextendr:::param_no_na_map(list(a = "1", b = 2)), 0L)
  expect_identical(
    caught_msg(miniextendr:::param_no_na_map(list(a = "NA", b = "2"))),
    "'x' must not contain NA"
  )
})

test_that("no_na on Result<AsNumeric, ()> lets NULL through and checks a value", {
  expect_identical(miniextendr:::param_no_na_number_result(), "NULL")
  expect_identical(miniextendr:::param_no_na_number_result(" 3 "), "3")
  expect_identical(
    caught_msg(miniextendr:::param_no_na_number_result("NA")),
    "'x' must not be NA"
  )
})

test_that("no_na on Either<AsNumericVec, R> checks the side the value took", {
  skip_if_not(miniextendr_has_feature("either"), "either feature off")
  expect_identical(miniextendr:::param_no_na_numbers_either(c("1", "2")), 0L)
  expect_identical(miniextendr:::param_no_na_numbers_either(as.raw(1:2)), -1L)
  expect_identical(
    caught_msg(miniextendr:::param_no_na_numbers_either("NA")),
    "'x' must not contain NA"
  )
})

test_that("impl methods check the converted value of a no_na marker", {
  h <- ParamCheckHolder$new()
  expect_identical(h$add_dose(" 2 "), 2)
  expect_identical(caught_msg(h$add_dose(" NA ")), "'dose' must not be NA")
  expect_identical(h$add_doses(c("1", "2")), 5)
  expect_identical(caught_msg(h$add_doses(c("1", ""))), "every dose must be a number")
  expect_identical(caught_msg(h$add_doses(c(1, NA))), "every dose must be a number")
})

test_that("no_na on an Either says contain when either arm holds several values", {
  skip_if_not(miniextendr_has_feature("either"), "either feature off")
  expect_identical(miniextendr:::either_no_na_raw_or_number(as.raw(1:2)), "raw:2")
  expect_identical(miniextendr:::either_no_na_raw_or_number(3), "number:3")
  # An Either has no R guard, so a value that neither arm reads gets the
  # conversion's refusal, which names both arms, before any NA check.
  expect_identical(
    caught_msg(miniextendr:::either_no_na_raw_or_number(c(1, NA))),
    "'x' must be raw or a single number: got length 2"
  )
  expect_identical(
    caught_msg(miniextendr:::either_no_na_raw_or_number(NA_real_)),
    "'x' must not contain NA"
  )
})

test_that("no_na on an Either is checked after the conversion, by the arm taken", {
  skip_if_not(miniextendr_has_feature("either"), "either feature off")
  f <- miniextendr:::value_or_table
  expect_identical(f(3), "value Some(3.0)")
  expect_identical(f(data.frame(id = 1:2, v = c(1, 2))), "table")
  # The data frame arm keeps its NA cells: no anyNA() runs on the whole value.
  expect_identical(f(data.frame(id = 1:2, v = c(1, NA))), "table")
  # A missing number is refused with the same condition as the R guard's.
  refused <- tryCatch(f(NA), error = identity)
  guarded <- tryCatch(miniextendr:::param_no_na_scalar(NA_real_), error = identity)
  expect_identical(conditionMessage(refused), "'x' must not be NA")
  expect_identical(class(refused), class(guarded))
  expect_identical(refused$param, "x")
  expect_identical(conditionCall(refused), quote(f(NA)))
  for (x in list(NA_real_, NaN, "NA")) {
    expect_identical(caught_msg(f(x)), "'x' must not be NA")
  }
})

test_that("no_na on an Either reads a newtype or Result arm by what it wraps", {
  skip_if_not(miniextendr_has_feature("either"), "either feature off")
  table <- data.frame(id = 1:2, v = c(1, NA))
  for (f in list(miniextendr:::value_or_wrapped_table, miniextendr:::value_or_result_table)) {
    expect_identical(f(3), "value Some(3.0)")
    expect_identical(f(data.frame(id = 1:2, v = c(1, 2))), "table 2x2")
    # The data frame behind the wrapper keeps its NA cells.
    expect_identical(f(table), "table 2x2")
    # The number arm still refuses what value_or_table() refuses, with the
    # same condition.
    for (x in list(NA, NA_real_, NaN, "NA")) {
      refused <- tryCatch(f(x), error = identity)
      plain <- tryCatch(miniextendr:::value_or_table(x), error = identity)
      expect_identical(conditionMessage(refused), "'x' must not be NA")
      expect_identical(class(refused), class(plain))
      expect_identical(refused$param, "x")
    }
  }
  # NULL is Err(()) on the Result arm: not given, and it holds no NA.
  expect_identical(miniextendr:::value_or_result_table(NULL), "nothing")
})

test_that("no_na on a Missing<Either> passes an omitted argument", {
  skip_if_not(miniextendr_has_feature("either"), "either feature off")
  f <- miniextendr:::value_or_table_optional
  expect_identical(f(), "nothing")
  expect_identical(f(3), "value Some(3.0)")
  expect_identical(f(data.frame(v = c(1, NA))), "table")
  expect_identical(caught_msg(f(NA)), "'x' must not be NA")
})

test_that("no_na on an Either with an Option arm passes NULL and refuses NA", {
  skip_if_not(miniextendr_has_feature("either"), "either feature off")
  f <- miniextendr:::value_or_table_nullable
  # NULL (the default, or given) converts to None on the number arm and
  # passes as not given; it never reaches the data frame arm.
  expect_identical(f(), "nothing")
  expect_identical(f(NULL), "nothing")
  expect_identical(f(3), "value Some(3.0)")
  expect_identical(f(data.frame(id = 1:2, v = c(1, NA))), "table")
  # NA is refused as on the plain number arm: R's NA by the check on the
  # input, the text "NA" by what the marker reads as missing.
  for (x in list(NA, "NA")) {
    refused <- tryCatch(f(x), error = identity)
    plain <- tryCatch(miniextendr:::value_or_table(x), error = identity)
    expect_identical(conditionMessage(refused), "'x' must not be NA")
    expect_identical(conditionMessage(refused), conditionMessage(plain))
    expect_identical(class(refused), class(plain))
    expect_identical(refused$param, "x")
    expect_identical(conditionCall(refused), quote(f(x)))
  }
  for (x in list(NA_real_, NaN, "")) {
    expect_identical(caught_msg(f(x)), "'x' must not be NA")
  }
})

test_that("no_na on an Either refuses what anyNA() sees, for the arm taken", {
  skip_if_not(miniextendr_has_feature("either"), "either feature off")
  f <- miniextendr:::either_no_na_vector_or_list
  expect_identical(f(1:2), "integer:2")
  expect_identical(f(c(1, 2)), "double:2")
  expect_identical(f(c("a", "b")), "character:2")
  expect_identical(f(integer()), "integer:0")
  expect_identical(f(list(a = 1, b = c(2, 3))), "list:2")
  # These arms carry NA through their conversion (the character arm reads it
  # as ""), so only the check on the input refuses it.
  for (x in list(c(1L, NA), c(1, NA), c(1, NaN), c("a", NA))) {
    expect_identical(caught_msg(f(x)), "'x' must not contain NA")
  }
  # A list is read as anyNA() reads it: a length-1 NA element counts, an NA
  # inside a longer element does not, and a data frame counts by its cells.
  expect_identical(caught_msg(f(list(a = NA_real_))), "'x' must not contain NA")
  expect_identical(f(list(a = c(1, NA))), "list:1")
  expect_identical(f(data.frame(v = c(1, 2))), "list:1")
  expect_identical(caught_msg(f(data.frame(v = c(1, NA)))), "'x' must not contain NA")
  # ALTREP input is read element by element: a compact sequence, and Rust
  # ALTREP vectors holding NA or NaN.
  expect_identical(f(1:10), "integer:10")
  for (x in list(into_sexp_altrep(c(1L, NA)), into_sexp_altrep(c(1, NaN)))) {
    expect_identical(caught_msg(f(x)), "'x' must not contain NA")
  }
})

# endregion

# region: not_inherits (#1815)
#
# A duration read with `AsNumeric` refuses the classes whose number drops the
# unit (difftime) or counts from 1970 (Date, POSIXct, POSIXlt). The refusal
# runs before the marker's type check, which would otherwise answer first
# with its generic message: `is.numeric()` is FALSE for all of them.

duration_msg <- paste(
  "`tau` must be a plain number in the time unit of the data,",
  "not a difftime, Date or date-time"
)
times <- list(
  difftime = as.difftime(5, units = "mins"),
  Date = as.Date("2024-01-02"),
  POSIXct = as.POSIXct("2024-01-02 03:04:05", tz = "UTC"),
  POSIXlt = as.POSIXlt("2024-01-02 03:04:05", tz = "UTC")
)

test_that("not_inherits refuses a difftime, Date or date-time with the author's message", {
  # The condition of every R-side argument check, as `inherits` raises it.
  inherits_e <- tryCatch(miniextendr:::param_inherits_one(list()), error = identity)
  for (x in times) {
    e <- tryCatch(miniextendr:::param_takes_duration(x), error = identity)
    expect_identical(conditionMessage(e), duration_msg)
    expect_identical(class(e), class(inherits_e))
    expect_s3_class(e, "rust_error")
    expect_identical(e$kind, "conversion")
    expect_identical(e$param, "tau")
    expect_null(e$rust_type)
    expect_equal(conditionCall(e), quote(miniextendr:::param_takes_duration(x)))
  }
})

test_that("not_inherits lets a number, NULL and a factor through", {
  expect_identical(miniextendr:::param_takes_duration(), "NULL")
  expect_identical(miniextendr:::param_takes_duration(NULL), "NULL")
  expect_identical(miniextendr:::param_takes_duration(2.5), "2.5")
  expect_identical(miniextendr:::param_takes_duration(3L), "3")
  expect_identical(miniextendr:::param_takes_duration("4"), "4")
  expect_identical(miniextendr:::param_takes_duration(factor("5")), "5")
})

test_that("a value of another class still gets the marker's own checks", {
  expect_identical(
    caught_msg(miniextendr:::param_takes_duration(list(1))),
    "'tau' must be NULL or numeric, logical, character, or factor"
  )
  expect_identical(
    caught_msg(miniextendr:::param_takes_duration(c(1, 2))),
    "'tau' must be NULL or have length 1"
  )
  expect_identical(caught_msg(miniextendr:::param_takes_duration(NA_real_)), "'tau' must not be NA")
  expect_identical(caught_msg(miniextendr:::param_takes_duration("NA")), "'tau' must not be NA")
})

test_that("without a message, not_inherits names the refused classes", {
  expect_identical(miniextendr:::param_not_inherits_default(2), "2")
  for (x in times) {
    expect_identical(
      caught_msg(miniextendr:::param_not_inherits_default(x)),
      "'x' must not inherit from 'difftime', 'Date' or 'POSIXt'"
    )
  }
})

test_that("inherits and not_inherits compose on one parameter", {
  expect_identical(miniextendr:::param_not_inherits_with_inherits(mx_obj()), 2L)
  expect_identical(
    caught_msg(miniextendr:::param_not_inherits_with_inherits(list())),
    "'x' must inherit from 'mx_obj'"
  )
  expect_identical(
    caught_msg(miniextendr:::param_not_inherits_with_inherits(
      structure(list(), class = c("mx_old", "mx_obj"))
    )),
    "'x' must not inherit from 'mx_old'"
  )
})

test_that("no_preconditions keeps not_inherits and drops the type checks", {
  f <- miniextendr:::param_takes_duration_no_preconditions
  expect_identical(f(2), "2")
  for (x in times) {
    expect_identical(caught_msg(f(x)), duration_msg)
  }
  # A list now reaches the Rust conversion and gets its message.
  e <- tryCatch(f(list(1)), error = identity)
  expect_s3_class(e, "rust_error")
  expect_identical(e$param, "tau")
  expect_false(grepl("numeric, logical, character, or factor", conditionMessage(e), fixed = TRUE))
})

test_that("not_inherits runs on an Either, which has no R type check", {
  skip_if_not(miniextendr_has_feature("either"), "either feature off")
  f <- miniextendr:::param_takes_duration_or_table
  expect_identical(f(), "NULL")
  expect_identical(f(2), "2")
  expect_identical(f(data.frame(v = 1)), "table")
  for (x in times) {
    expect_identical(caught_msg(f(x)), duration_msg)
  }
})

test_that("impl methods take not_inherits(...) at method level", {
  h <- ParamCheckHolder$new()
  expect_identical(h$add_duration(2), 2)
  expect_identical(h$add_duration(NULL), 2)
  expect_identical(h$add_duration(factor("1")), 3)
  for (x in times) {
    e <- tryCatch(h$add_duration(x), error = identity)
    expect_identical(conditionMessage(e), duration_msg)
    expect_identical(e$param, "tau")
    expect_s3_class(e, "rust_error")
  }
  expect_identical(
    caught_msg(h$add_duration(list(1))),
    "'tau' must be NULL or numeric, logical, character, or factor"
  )
})

# endregion

# region: private R6 methods

test_that("a private R6 method keeps its argument checks", {
  obj <- PrivateCheckR6$new()
  priv <- obj$.__enclos_env__$private
  mx <- structure(list(), class = "mx_obj")
  expect_identical(priv$add_private(mx, 2), 2)
  expect_identical(obj$total(), 2)
  # Rust converts a classless list and an NA double without complaint, so
  # these failures come from the R-side checks of the private method.
  e <- tryCatch(priv$add_private(list(), 1), error = identity)
  expect_s3_class(e, "rust_error")
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "x")
  expect_identical(conditionMessage(e), "'x' must inherit from 'mx_obj'")
  expect_identical(caught_msg(priv$add_private(mx, NA_real_)), "'y' must not be NA")
  expect_identical(obj$total(), 2)
})

# endregion
