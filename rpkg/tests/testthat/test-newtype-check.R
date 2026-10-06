# `#[try_from_sexp(validate = ...)]` (#1815): a `#[derive(TryFromSexp)]`
# newtype's check runs on the R value in every shape, before the inner type
# reads it, and its refusal is the argument error: the check's classes and
# fields, `kind = "conversion"`, `e$param`, `e$rust_type` and the call as
# written. Fixtures: rpkg/src/rust/newtype_check_tests.rs.

refused <- function(expr) tryCatch(expr, error = identity)

unit_classes <- c(
  "mx_unit_refused", "mx_newtype_check",
  "rust_error", "simpleError", "error", "condition"
)
reason <- function(class) {
  sprintf("got a %s; give a plain number in the data's time unit", class)
}

mins <- as.difftime(5, units = "mins")
day <- as.Date("2024-01-01")
dates <- as.Date(c("2024-01-01", "2024-02-01"))
times <- as.POSIXct(c("2024-01-01", NA), tz = "UTC")

test_that("a scalar newtype runs its check", {
  expect_identical(miniextendr:::newtype_check_scalar(5), 5)

  e <- refused(miniextendr:::newtype_check_scalar(mins))
  expect_identical(class(e), unit_classes)
  expect_identical(
    conditionMessage(e),
    paste0("'x' must be a single double: ", reason("difftime"))
  )
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "x")
  expect_identical(e$rust_type, "Elapsed")
  expect_identical(e$unit_class, "difftime")
  expect_identical(conditionCall(e), quote(miniextendr:::newtype_check_scalar(mins)))
})

test_that("Option<T> runs the check on a value, never on NULL", {
  expect_identical(miniextendr:::newtype_check_option(), "NULL")
  expect_identical(miniextendr:::newtype_check_option(NULL), "NULL")
  expect_identical(miniextendr:::newtype_check_option(2.5), "2.5")

  e <- refused(miniextendr:::newtype_check_option(day))
  expect_identical(class(e), unit_classes)
  expect_identical(
    conditionMessage(e),
    paste0("'x' must be NULL or a single double: ", reason("Date"))
  )
  expect_identical(e$param, "x")
  expect_identical(e$rust_type, "Option<Elapsed>")
  expect_identical(e$unit_class, "Date")
  expect_identical(conditionCall(e), quote(miniextendr:::newtype_check_option(day)))
})

test_that("Vec<T> runs the check on the whole vector", {
  expect_identical(miniextendr:::newtype_check_vec(c(1, 2)), 3)

  e <- refused(miniextendr:::newtype_check_vec(dates))
  expect_identical(class(e), unit_classes)
  expect_identical(
    conditionMessage(e),
    paste0("invalid 'x' argument: ", reason("Date"))
  )
  expect_identical(e$param, "x")
  expect_identical(e$rust_type, "Vec<Elapsed>")
  expect_identical(e$unit_class, "Date")
  expect_identical(conditionCall(e), quote(miniextendr:::newtype_check_vec(dates)))
})

test_that("Vec<Option<T>> runs the check on the whole vector", {
  expect_identical(miniextendr:::newtype_check_vec_option(c(1, NA)), 1L)

  e <- refused(miniextendr:::newtype_check_vec_option(times))
  expect_identical(class(e), unit_classes)
  expect_identical(
    conditionMessage(e),
    paste0("invalid 'x' argument: ", reason("POSIXt"))
  )
  expect_identical(e$param, "x")
  expect_identical(e$rust_type, "Vec<Option<Elapsed>>")
  expect_identical(e$unit_class, "POSIXt")
  expect_identical(conditionCall(e), quote(miniextendr:::newtype_check_vec_option(times)))
})

test_that("an Either arm reports its check's refusal, classes included", {
  expect_identical(miniextendr:::newtype_check_either(NULL), "NULL")
  expect_identical(miniextendr:::newtype_check_either(" 3 "), "Some(3.0)")
  expect_identical(miniextendr:::newtype_check_either(data.frame(a = 1)), "data frame")

  # The data frame arm refused the kind of value, so the reason is the
  # check's, with its classes and fields.
  e <- refused(miniextendr:::newtype_check_either(mins))
  expect_identical(class(e), unit_classes)
  expect_identical(
    conditionMessage(e),
    paste0("'tau' must be NULL or a single number or a data frame: ", reason("difftime"))
  )
  expect_identical(e$param, "tau")
  expect_identical(e$unit_class, "difftime")
  expect_identical(conditionCall(e), quote(miniextendr:::newtype_check_either(mins)))

  # Both arms refused the kind of value: no refusal to report.
  e <- refused(miniextendr:::newtype_check_either(list(1)))
  expect_identical(class(e), c("rust_error", "simpleError", "error", "condition"))
  expect_identical(
    conditionMessage(e),
    "'tau' must be NULL or a single number or a data frame: got list"
  )
})

test_that("raising from inside try_from_sexp keeps the class and call, not the argument context", {
  expect_identical(miniextendr:::newtype_check_raised(2), 2)

  # `rust_error!` in a conversion is the function's error, not an argument
  # error: the class and the call as written, but `kind = "error"`, no
  # `e$param`, no `e$rust_type` and no crate `conversion_error_class`.
  e <- refused(miniextendr:::newtype_check_raised(mins))
  expect_identical(
    class(e),
    c("mx_unit_refused", "rust_error", "simpleError", "error", "condition")
  )
  expect_identical(conditionMessage(e), "got a difftime")
  expect_identical(e$kind, "error")
  expect_null(e$param)
  expect_null(e$rust_type)
  expect_identical(conditionCall(e), quote(miniextendr:::newtype_check_raised(mins)))
})
