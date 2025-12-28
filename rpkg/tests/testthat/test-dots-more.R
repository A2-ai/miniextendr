test_that("greetings variants handle dots (known wrappers gaps marked)", {
  expect_error(
    greetings_with_named_dots(a = 1, b = 2),
    "unused argument",
    info = "Wrapper currently generated as function(dots=...), so named dots fail; fix in macro then relax test."
  )
  expect_error(
    greetings_with_named_and_unused_dots(a = 1),
    "unused argument",
    info = "Wrapper currently generated as function(unused_dots=...), so named dots fail."
  )
  expect_null(greetings_with_nameless_dots(1, 2, 3))
  expect_error(
    greetings_last_as_named_dots(1L, x = 2),
    "unused argument",
    info = "Generated wrapper uses dots=... formal; adjust when macro fixed."
  )
  expect_error(
    greetings_last_as_named_and_unused_dots(1L, x = 2),
    "unused argument",
    info = "Generated wrapper uses dots=... formal; adjust when macro fixed."
  )
  expect_null(greetings_last_as_nameless_dots(1L, 2, 3))
})
