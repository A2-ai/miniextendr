# Classed Result errors (RConditionError / RError), class vectors and the
# reserved condition-data names (#1434, #1435, #1440).

test_that("Result<T, E: RConditionError> raises member + family classes with data", {
  e <- tryCatch(classed_result_missing(""), error = function(e) e)
  expect_equal(class(e), c("pkg_error_missing_field", "pkg_error", "rust_error",
                           "simpleError", "error", "condition"))
  expect_equal(conditionMessage(e), "field `id` is missing")
  expect_equal(e$field, "id")
  expect_equal(e$kind, "result_err")

  # Member handler and family handler both dispatch.
  expect_equal(tryCatch(classed_result_missing(""), pkg_error_missing_field = function(e) e$field), "id")
  expect_equal(tryCatch(classed_result_range(150), pkg_error = function(e) e$max), 100)
  expect_equal(tryCatch(classed_result_range(150), pkg_error_out_of_range = function(e) e$value), 150)

  # Ok path untouched.
  expect_equal(classed_result_missing("abc"), 3L)
  expect_equal(classed_result_range(5), 5)

  # A data field whose value is an explicit NULL stays a present field: the
  # splice must not drop it (utils::modifyList's default keep.null = FALSE did).
  e <- tryCatch(classed_result_null_field(), error = function(e) e)
  expect_s3_class(e, "pkg_error_no_value")
  expect_true("optional" %in% names(e))
  expect_null(e$optional)
  expect_equal(e$present, 1)
})

test_that("classed Result errors work on the unit-return and impl-method arms", {
  e <- tryCatch(classed_result_unit(500), error = function(e) e)
  expect_s3_class(e, "pkg_error_out_of_range")
  expect_equal(e$value, 500)
  expect_invisible(classed_result_unit(1))

  chk <- new_classedchecker(10)
  expect_equal(check_bound(chk, 3), 3)
  e <- tryCatch(check_bound(chk, 30), error = function(e) e)
  expect_s3_class(e, "pkg_error_out_of_range")
  expect_s3_class(e, "pkg_error")
  expect_equal(e$max, 10)
})

test_that("RError: From<Error> keeps the message, builders add class and data", {
  expect_equal(rerror_parse("42"), 42L)
  e <- tryCatch(rerror_parse("x"), error = function(e) e)
  expect_equal(class(e)[1:3], c("pkg_bad_number", "pkg_error", "rust_error"))
  expect_match(conditionMessage(e), "invalid digit")
  expect_equal(e$input, "x")

  e <- tryCatch(rerror_plain(), error = function(e) e)
  expect_equal(class(e)[1], "rust_error")
  expect_equal(conditionMessage(e), "plain rerror")
  expect_equal(e$kind, "result_err")
})

test_that("a computed reserved field name is rejected, not spliced", {
  e <- tryCatch(rerror_reserved_runtime("kind"), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_match(conditionMessage(e), "reserved")
  expect_match(conditionMessage(e), "rename the field")
  expect_equal(e$kind, "panic")

  e <- tryCatch(reserved_data_macro_runtime("message"), error = function(e) e)
  expect_match(conditionMessage(e), "reserved")
  expect_equal(e$kind, "panic")

  # Non-reserved computed names still work.
  e <- tryCatch(rerror_reserved_runtime("weight"), error = function(e) e)
  expect_equal(e$weight, 1L)
})

test_that("macro class vectors layer member before family", {
  e <- tryCatch(classed_error_vec("pkg_err_member", "pkg_err_family"), error = function(e) e)
  expect_equal(class(e)[1:3], c("pkg_err_member", "pkg_err_family", "rust_error"))
  expect_equal(tryCatch(classed_error_vec("a_member", "a_family"), a_family = function(e) "family"), "family")

  w <- tryCatch(classed_warning_vec(3L), warning = function(w) w)
  expect_equal(class(w)[1:3], c("pkg_warning_dropped", "pkg_warning", "rust_warning"))
  expect_equal(w$dropped, 3L)

  # `rust_condition!` unwinds the Rust call, so the wrapper returns NULL after
  # signalling; the class vector is what a handler sees.
  seen <- tryCatch(
    classed_condition_vec(c("pkg_cond_member", "pkg_cond_family")),
    pkg_cond_family = function(c) class(c)[1:3]
  )
  expect_equal(seen, c("pkg_cond_member", "pkg_cond_family", "rust_condition"))
})

# Argument-conversion errors: a `TryFromSexp::Error` implementing
# `RConditionError` classes the conversion condition; `kind` stays
# "conversion", the message keeps the parameter context and `e$param` names
# the R parameter.

test_that("a classed TryFromSexp error classes the conversion condition", {
  e <- tryCatch(hyperparams_total(1:3), error = function(e) e)
  expect_equal(class(e), c("mx_fixture_bad_arg", "mx_fixture_error", "rust_error",
                           "simpleError", "error", "condition"))
  expect_equal(e$kind, "conversion")
  expect_equal(e$param, "hyper")
  expect_match(e$reason, "names attribute", fixed = TRUE)
  expect_match(
    conditionMessage(e),
    "^failed to convert parameter 'hyper' to Hyperparams: expected a named numeric vector \\("
  )
  expect_equal(conditionCall(e), quote(hyperparams_total(hyper = 1:3)))

  # Family and member handlers both dispatch.
  expect_equal(tryCatch(hyperparams_total("a"), mx_fixture_error = function(e) e$param), "hyper")
  expect_equal(tryCatch(hyperparams_total("a"), mx_fixture_bad_arg = function(e) e$kind), "conversion")

  # A value that converts is untouched.
  expect_equal(hyperparams_total(c(alpha = 1, beta = 0.5)), 1.5)
})

test_that("the error type's own `param` field wins over the parameter name", {
  e <- tryCatch(hyperparams_total(c(alpha = 1, beta = -2)), error = function(e) e)
  expect_s3_class(e, "mx_fixture_negative_hyperparam")
  expect_s3_class(e, "mx_fixture_error")
  expect_equal(e$kind, "conversion")
  # The type's data names the hyperparameter; the parameter stays in the message.
  expect_equal(e$param, "beta")
  expect_equal(sum(names(e) == "param"), 1L)
  expect_equal(e$value, -2)
  expect_equal(
    conditionMessage(e),
    "failed to convert parameter 'hyper' to Hyperparams: hyperparameter 'beta' must be non-negative, got -2"
  )
})

test_that("classed conversion errors reach the worker path", {
  skip_if_not(
    exists("hyperparams_total_worker", envir = asNamespace("miniextendr"), inherits = FALSE),
    "worker-thread feature not enabled"
  )
  e <- tryCatch(hyperparams_total_worker(1:3), error = function(e) e)
  expect_s3_class(e, "mx_fixture_bad_arg")
  expect_equal(e$kind, "conversion")
  expect_equal(e$param, "hyper")
  expect_equal(conditionCall(e), quote(hyperparams_total_worker(hyper = 1:3)))
  expect_equal(hyperparams_total_worker(c(alpha = 2)), 2)
})

test_that("classed conversion errors reach the call = caller path", {
  e <- tryCatch(miniextendr:::hyperparams_total_caller(c(a = -1)), error = function(e) e)
  expect_s3_class(e, "mx_fixture_negative_hyperparam")
  expect_equal(e$param, "a")
  expect_equal(
    conditionCall(e),
    quote(miniextendr:::hyperparams_total_caller(hyper = c(a = -1)))
  )
})

test_that("a plain SexpError keeps the default class, with e$param and the new wording", {
  e <- tryCatch(either_int_or_str(3.5), error = function(e) e)
  expect_equal(class(e), c("rust_error", "simpleError", "error", "condition"))
  expect_equal(e$kind, "conversion")
  expect_equal(e$param, "value")
  expect_match(
    conditionMessage(e),
    "failed to convert parameter 'value' to Either<i32, String>: failed to convert to Either: ",
    fixed = TRUE
  )
  expect_no_match(conditionMessage(e), "wrong type, length, or contains NA", fixed = TRUE)

  e <- tryCatch(miniextendr:::test_fromstr_vec_ints(c(1, 2)), error = function(e) e)
  expect_equal(class(e), c("rust_error", "simpleError", "error", "condition"))
  expect_equal(e$kind, "conversion")
  expect_equal(e$param, "nums")
  expect_match(
    conditionMessage(e),
    "failed to convert parameter 'nums' to AsFromStrVec<i32>: ",
    fixed = TRUE
  )
})
