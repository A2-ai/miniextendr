# A list of checked objects as an argument (#1837): `Vec<Model>` and
# `Vec<Option<Model>>` of a `#[derive(TryFromSexp)]` newtype over `List` run
# the type's check on each element and report every failing element, with its
# position, in one argument error. Fixtures in
# `src/rust/list_newtype_vec_tests.rs`.

mx_model <- function(...) structure(list(...), class = "mx_model")
catch_error <- function(expr) tryCatch(expr, error = function(e) e)

test_that("a list of models converts, each element checked as a model", {
  expect_identical(list_newtype_count(list(mx_model(a = 1), mx_model(b = 2))), 2L)
  expect_identical(list_newtype_count(list()), 0L)
  # The outer list is not a model; only its elements are checked.
  expect_identical(list_newtype_count(list(mx_model())), 1L)
})

test_that("one bad element names its position and the parameter", {
  e <- catch_error(list_newtype_count(list(mx_model(), data.frame(a = 1))))
  expect_s3_class(e, "mx_model_refused")
  expect_s3_class(e, "rust_error")
  expect_identical(
    conditionMessage(e),
    "invalid 'fits' argument: use model_from_df() for a data frame (element 2)"
  )
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "fits")
  expect_identical(e$rust_type, "Vec<Model>")
  expect_identical(e$advice, "model_from_df()")
  expect_identical(conditionCall(e)[[1L]], quote(list_newtype_count))
})

test_that("every bad element is reported in one error", {
  e <- catch_error(list_newtype_count(list(1, mx_model(), data.frame(a = 1), "x")))
  expect_s3_class(e, "mx_model_refused")
  expect_identical(
    conditionMessage(e),
    paste0(
      "invalid 'fits' argument: got no model object (elements 1, 4); ",
      "use model_from_df() for a data frame (element 3)"
    )
  )
  expect_identical(e$param, "fits")
})

test_that("the check's class catches one model and a list of them alike", {
  for (call in list(
    quote(list_newtype_one(1)),
    quote(list_newtype_count(list(1))),
    quote(list_newtype_present(list(NULL, 1)))
  )) {
    e <- tryCatch(eval(call), mx_model_refused = function(e) e)
    expect_s3_class(e, "mx_model_refused")
    expect_identical(e$kind, "conversion")
  }
})

test_that("an argument that is not a list is a type error", {
  e <- catch_error(list_newtype_count(1:3))
  expect_s3_class(e, "rust_error")
  expect_false(inherits(e, "mx_model_refused"))
  expect_identical(conditionMessage(e), "invalid 'fits' argument: expected list, got integer")
  expect_identical(e$param, "fits")
  e <- catch_error(list_newtype_count(NULL))
  expect_identical(conditionMessage(e), "invalid 'fits' argument: expected list, got NULL")
})

test_that("one model given where a list of them is expected reports its fields", {
  e <- catch_error(list_newtype_count(mx_model(a = 1, b = 2)))
  expect_s3_class(e, "mx_model_refused")
  expect_identical(
    conditionMessage(e),
    "invalid 'fits' argument: got no model object (elements 1, 2)"
  )
})

test_that("Vec<Option<Model>> reads NULL elements as None, without the check", {
  expect_identical(
    list_newtype_present(list(mx_model(), NULL, mx_model())),
    c(TRUE, FALSE, TRUE)
  )
  expect_identical(list_newtype_present(list(NULL, NULL)), c(FALSE, FALSE))
  e <- catch_error(list_newtype_present(list(NULL, data.frame(a = 1))))
  expect_s3_class(e, "mx_model_refused")
  expect_identical(
    conditionMessage(e),
    "invalid 'fits' argument: use model_from_df() for a data frame (element 2)"
  )
})

test_that("a list of models goes back to R as a list", {
  models <- list(mx_model(a = 1), mx_model(b = 2))
  expect_identical(list_newtype_round_trip(models), models)
  with_null <- list(mx_model(a = 1), NULL, mx_model())
  expect_identical(list_newtype_round_trip_option(with_null), with_null)
})

test_that("Vec<List> reads a list of lists and reports every non-list", {
  expect_identical(
    list_newtype_lengths(list(list(1, 2), list(), data.frame(a = 1:3))),
    c(2L, 0L, 1L)
  )
  e <- catch_error(list_newtype_lengths(list(list(), 2, "a", pairlist(a = 1))))
  expect_s3_class(e, "rust_error")
  expect_identical(
    conditionMessage(e),
    paste0(
      "invalid 'x' argument: expected list, got numeric (element 2); ",
      "expected list, got character (element 3); ",
      "expected list, got pairlist (element 4)"
    )
  )
  expect_identical(e$param, "x")
})

test_that("the models read under GC pressure come back whole", {
  models <- gc_stress_list_newtype_vec()
  expect_length(models, 24L)
  expect_s3_class(models[[1L]], "mx_model")
  expect_identical(models[[24L]][[1L]], 23L)
})
