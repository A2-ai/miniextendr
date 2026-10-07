# A type that words its whole argument error once (#1833): `ModelArg`'s
# check, in `#[try_from_sexp(validate = ...)]`, gives `RError::argument_message`,
# which is the condition's message as given. Fixtures in
# `src/rust/argument_message_tests.rs`.

mx_model <- function() structure(list(a = 1, b = 2), class = "mx_model")
catch_error <- function(expr) tryCatch(expr, error = function(e) e)

test_that("the type's argument message matches the inherits(..., when(...)) guard", {
  inputs <- list(data.frame(a = 1), 1, "x", list(1))
  for (input in inputs) {
    via_type <- catch_error(argument_message_model(input))
    via_guard <- catch_error(argument_message_guard(input))
    expect_s3_class(via_type, "rust_error")
    expect_identical(conditionMessage(via_type), conditionMessage(via_guard))
    expect_identical(class(via_type), class(via_guard))
    expect_identical(via_type$kind, "conversion")
    expect_identical(via_guard$kind, "conversion")
    expect_identical(via_type$param, "fit")
    expect_identical(via_guard$param, "fit")
    # The same call shape, each naming its own function.
    expect_identical(conditionCall(via_type)[[1L]], quote(argument_message_model))
    expect_identical(conditionCall(via_guard)[[1L]], quote(argument_message_guard))
    expect_identical(as.list(conditionCall(via_type))[-1L], as.list(conditionCall(via_guard))[-1L])
    # Only the conversion names the Rust type, as for any conversion error.
    expect_identical(via_type$rust_type, "ModelArg")
    expect_null(via_guard$rust_type)
  }
  expect_identical(
    conditionMessage(catch_error(argument_message_model(data.frame(a = 1)))),
    "use model_from_df() for a data frame"
  )
  expect_identical(conditionMessage(catch_error(argument_message_model(1))), "expected a model object")
  expect_identical(argument_message_model(mx_model()), 2L)
  expect_identical(argument_message_guard(mx_model()), 2L)
})

test_that("every function taking the type gets the check, with its own param", {
  e <- catch_error(argument_message_model_other(1, data.frame(a = 1)))
  expect_identical(conditionMessage(e), "use model_from_df() for a data frame")
  expect_identical(e$param, "fit")
  expect_identical(argument_message_model_other(1, mx_model()), 3)
})

test_that("Option<T> of the type checks any value but NULL", {
  expect_false(argument_message_model_option())
  expect_false(argument_message_model_option(NULL))
  expect_true(argument_message_model_option(mx_model()))
  e <- catch_error(argument_message_model_option(data.frame(a = 1)))
  expect_s3_class(e, "rust_error")
  expect_identical(conditionMessage(e), "use model_from_df() for a data frame")
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "fit")
  expect_identical(conditionMessage(catch_error(argument_message_model_option(1))), "expected a model object")
})

test_that("a hand-written conversion's RError gives its argument message too", {
  e <- catch_error(argument_message_hand(data.frame(a = 1)))
  expect_s3_class(e, "rust_error")
  expect_identical(conditionMessage(e), "use model_from_df() for a data frame")
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "fit")
  expect_identical(argument_message_hand(mx_model()), 2L)
})

test_that("a derived RConditionError gives the variant's argument message", {
  e <- catch_error(argument_message_derived(data.frame(a = 1)))
  expect_s3_class(e, "mx_model_error_data_frame")
  expect_s3_class(e, "mx_model_error")
  expect_identical(conditionMessage(e), "use model_from_df() for a data frame")
  expect_identical(e$param, "fit")
  # A variant without one keeps the prefixed message.
  e <- catch_error(argument_message_derived(1))
  expect_s3_class(e, "mx_model_error_not_a_model")
  expect_identical(conditionMessage(e), "invalid 'fit' argument: got no model object")
  expect_identical(argument_message_derived(mx_model()), 2L)
})

test_that("a scalar-backed type's argument message holds in Vec and Vec<Option>", {
  hours <- as.difftime(c(1, 2), units = "hours")
  advice <- "give the spacing as a plain number in hours"
  for (call in list(
    quote(argument_message_spacing(hours[1L])),
    quote(argument_message_spacing_vec(hours)),
    quote(argument_message_spacing_vec_option(hours))
  )) {
    e <- catch_error(eval(call))
    expect_s3_class(e, "mx_spacing_refused")
    expect_s3_class(e, "rust_error")
    expect_identical(conditionMessage(e), advice)
    expect_identical(e$param, "x")
    expect_identical(e$kind, "conversion")
  }
  expect_identical(argument_message_spacing(1.5), 1.5)
  expect_identical(argument_message_spacing_vec(c(1, 2)), 3)
  expect_identical(argument_message_spacing_vec_option(c(1, NA)), 1L)
})
