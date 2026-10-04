test_that("greetings_with_nameless_dots() runs without error", {
  expect_null(greetings_with_nameless_dots())
  expect_null(greetings_with_nameless_dots(1, 2, 3))
})

test_that("greetings_last_as_nameless_dots() runs without error", {
  expect_null(greetings_last_as_nameless_dots(1L))
  expect_null(greetings_last_as_nameless_dots(1L, 2, 3))
})

test_that("`_: ...` dots are R's plain `...` (#1743)", {
  expect_identical(names(formals(greetings_with_nameless_dots)), "...")
  expect_identical(
    names(formals(greetings_last_as_nameless_dots)),
    c("exclamations", "...")
  )
  expect_null(greetings_with_nameless_dots(a = 1, 2, b = "x"))
  expect_null(greetings_last_as_nameless_dots(1L, extra = TRUE))
  # typed_list! sugar reads `_: ...` through its synthetic binding.
  expect_identical(names(formals(validate_with_attribute)), "...")
  expect_equal(validate_with_attribute(x = 1, y = 2), "x=1, y=2")
})
