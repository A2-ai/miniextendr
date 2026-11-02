# A hand-written TryFromSexpElement: Celsius implements the element trait
# itself, so Vec<Celsius>, Option<Celsius> and Vec<Option<Celsius>> convert
# through f64's containers, and its check refuses a "fahrenheit" vector in
# every shape (element_trait_tests.rs).

fahrenheit <- function(x) structure(x, class = "fahrenheit")

test_that("Vec<T> of a hand-written element reads f64's vector", {
  expect_identical(celsius_mean(c(10, 20, 30)), 20)
  expect_identical(celsius_mean(-5), -5)
  expect_identical(celsius_mean(numeric()), NA_real_)
  expect_true(is.na(celsius_mean(c(1, NA))))
})

test_that("Option<T> of a hand-written element is None for NULL and NA", {
  expect_identical(celsius_or_room(37), 37)
  expect_identical(celsius_or_room(NULL), 20)
  expect_identical(celsius_or_room(NA_real_), 20)
})

test_that("Vec<Option<T>> of a hand-written element keeps the NAs apart", {
  expect_identical(celsius_known(c(1, NA, 3)), 2L)
  expect_identical(celsius_known(c(NA_real_, NA_real_)), 0L)
  expect_identical(celsius_known(numeric()), 0L)
})

test_that("the scalar forwards to f64's conversion", {
  expect_identical(celsius_scalar(-40), -40)
  expect_error(celsius_scalar(c(1, 2)))
  expect_error(celsius_scalar(numeric()))
})

test_that("the element check refuses in every shape, with its own classes", {
  for (f in list(celsius_mean, celsius_or_room, celsius_known, celsius_scalar)) {
    e <- tryCatch(f(fahrenheit(50)), error = function(e) e)
    expect_s3_class(e, c("temperature_unit_error", "temperature_error"))
    expect_match(
      conditionMessage(e),
      "temperatures in Fahrenheit: convert them to Celsius first",
      fixed = TRUE
    )
    expect_identical(e$unit, "fahrenheit")
  }
  # The check runs before the vector is read: an empty refused vector errors.
  expect_error(celsius_mean(fahrenheit(numeric())), class = "temperature_unit_error")
})
