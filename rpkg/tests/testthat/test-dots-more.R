test_that("greetings variants with dots run without error", {
  # All named/unnamed dots variants now work correctly after macro fix
  expect_null(greetings_with_named_dots(a = 1, b = 2))
  expect_null(greetings_with_named_and_unused_dots(a = 1))
  expect_null(greetings_with_nameless_dots(1, 2, 3))
  expect_null(greetings_last_as_named_dots(1L, x = 2))
  expect_null(greetings_last_as_named_and_unused_dots(1L, x = 2))
  expect_null(greetings_last_as_nameless_dots(1L, 2, 3))
})

# =============================================================================
# typed_list! macro tests
# =============================================================================

test_that("validate_numeric_args validates type and length", {
  # Valid input: numeric(4) + list
  expect_equal(
    validate_numeric_args(alpha = c(1.0, 2.0, 3.0, 4.0), beta = list(1, 2)),
    4L
  )

  # Wrong length for alpha
  expect_error(
    validate_numeric_args(alpha = c(1.0, 2.0), beta = list(1, 2)),
    "wrong length"
  )

  # Missing required field
  expect_error(
    validate_numeric_args(beta = list(1, 2)),
    "missing"
  )

  # Wrong type for alpha (integer instead of numeric)
  expect_error(
    validate_numeric_args(alpha = 1:4, beta = list(1, 2)),
    "wrong type"
  )

  # Optional field gamma can be omitted
  expect_equal(
    validate_numeric_args(alpha = c(1.0, 2.0, 3.0, 4.0), beta = list(1)),
    4L
  )

  # Optional field gamma can be provided
  expect_equal(
    validate_numeric_args(alpha = c(1.0, 2.0, 3.0, 4.0), beta = list(), gamma = "hello"),
    4L
  )
})

test_that("validate_strict_args rejects extra fields", {
  # Valid input
  expect_equal(validate_strict_args(x = 1.0, y = 2.0), "x=1, y=2")

  # Extra field should error
  expect_error(
    validate_strict_args(x = 1.0, y = 2.0, z = 3.0),
    "extra"
  )

  # Missing field
  expect_error(
    validate_strict_args(x = 1.0),
    "missing"
  )
})

test_that("validate_class_args checks R class", {
  # Valid data.frame - returns ncol (data.frame is a list of columns)
  expect_equal(validate_class_args(data = data.frame(a = 1:3)), 1L)
  expect_equal(validate_class_args(data = data.frame(a = 1:3, b = 4:6)), 2L)

  # tibble should work (inherits from data.frame)
  skip_if_not_installed("tibble")
  expect_equal(validate_class_args(data = tibble::tibble(a = 1:5)), 1L)
  expect_equal(validate_class_args(data = tibble::tibble(a = 1:5, b = 6:10)), 2L)

  # Plain list should fail
  expect_error(
    validate_class_args(data = list(a = 1:3)),
    "wrong type"
  )

  # Matrix should fail
  expect_error(
    validate_class_args(data = matrix(1:6, nrow = 2)),
    "wrong type"
  )
})

# =============================================================================
# Attribute sugar: #[miniextendr(dots = typed_list!(...))]
# =============================================================================

test_that("validate_with_attribute works with valid input", {
  result <- validate_with_attribute(x = 1.0, y = 2.0)
  expect_equal(result, "x=1, y=2")
})

test_that("validate_with_attribute fails on missing required field", {
  # Same wording as the direct typed_list! path (Display, not Debug) — see
  # "validate_numeric_args validates type and length" above.
  expect_error(
    validate_with_attribute(x = 1.0),
    "missing"
  )
})

test_that("validate_with_attribute fails on wrong type", {
  # Same wording as the direct typed_list! path (Display, not Debug) — see
  # "validate_numeric_args validates type and length" above.
  expect_error(
    validate_with_attribute(x = "not a number", y = 2.0),
    "wrong type"
  )
})

test_that("validate_attr_optional works without optional field", {
  result <- validate_attr_optional(name = "Alice")
  expect_equal(result, "Hello, Alice!")
})

test_that("validate_attr_optional works with optional field", {
  result <- validate_attr_optional(name = "Bob", greeting = "Hi")
  expect_equal(result, "Hi, Bob!")
})

test_that("direct and attribute-sugar typed_list paths share one message style", {
  # Audit A8: the attribute-sugar path used to Debug-format TypedListError
  # (PascalCase variant names like `Missing { name: "y" }`), so a user regex
  # matching one entry point missed the other. Both now go through the
  # Display impl — assert the SAME regex matches both paths.
  missing_re <- "missing required field"
  expect_error(validate_numeric_args(beta = list(1, 2)), missing_re) # direct
  expect_error(validate_with_attribute(x = 1.0), missing_re) # attribute sugar

  wrong_type_re <- "has wrong type: expected"
  expect_error(
    validate_numeric_args(alpha = 1:4, beta = list(1, 2)), # direct
    wrong_type_re
  )
  expect_error(
    validate_with_attribute(x = "not a number", y = 2.0), # attribute sugar
    wrong_type_re
  )
})

test_that("TypedList::get / get_opt retrieve VECTOR fields (golife hiccup #5)", {
  # Before hiccup #5, get/get_opt required T: TryFromSexp<Error = SexpError>,
  # so only scalar target types compiled -- .get::<Vec<i32>>() failed to build.
  # This fixture retrieves numeric()/integer()/character() fields as Vec<f64> /
  # Vec<i32> / Option<Vec<String>>; if it links and returns, the vector-target
  # bound works.
  result <- typed_list_get_vectors()
  expect_equal(
    result,
    "weights_sum=7.5, survive_sum=100, labels=Some([\"a\", \"b\"]), absent_is_none=true"
  )
})
