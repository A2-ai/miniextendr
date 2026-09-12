test_that("conversion markers and attributes register both S7 directions", {
  input <- ConvertInput(12L)
  for (target in list(TypedConversion, AttributeConversion)) {
    converted <- S7::convert(input, target)
    expect_identical(conversion_value(converted), 12L)
    restored <- S7::convert(converted, ConvertInput)
    expect_true(S7::S7_inherits(restored, ConvertInput))
    expect_identical(conversion_value(restored), 12L)
  }
})

test_that("ordinary conversion method calls agree with S7::convert", {
  input <- ConvertInput(13L)
  for (prefix in c("TypedConversion", "AttributeConversion")) {
    from <- base::get(paste0(prefix, "_from_source"), asNamespace("miniextendr"))
    to <- base::get(paste0(prefix, "_to_source"), asNamespace("miniextendr"))
    converted <- from(input)
    expect_identical(conversion_value(converted), 13L)
    restored <- to(converted)
    expect_true(S7::S7_inherits(restored, ConvertInput))
    expect_identical(conversion_value(restored), 13L)
    expect_false(withVisible(to(converted))$visible)
    expect_false(withVisible(S7::convert(converted, ConvertInput))$visible)
  }
})

test_that("S7 conversion checks Result errors before class construction", {
  input <- ConvertInput(-1L)
  for (target in list(TypedConversion, AttributeConversion)) {
    expect_error(S7::convert(input, target), "negative conversion input", class = "rust_error")
    expect_error(S7::convert(target(-1L), ConvertInput), "negative conversion output", class = "rust_error")
  }
})

test_that("S7 conversion markers remain safe under GC stress", {
  skip_on_cran()
  input <- ConvertInput(14L)
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  converted <- S7::convert(input, TypedConversion)
  expect_identical(conversion_value(S7::convert(converted, ConvertInput)), 14L)
})
