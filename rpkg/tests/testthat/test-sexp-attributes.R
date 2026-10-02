# SexpExt::has_attributes() and SexpExt::is_identical(), checked against R's
# own attributes() and identical().

attribute_cases <- list(
  null = NULL,
  bare_false = FALSE,
  integer = 1:3,
  plain_list = list(1, "a"),
  names_only = c(a = 1, b = 2),
  matrix = matrix(1:4, 2L),
  factor = factor(c("x", "y")),
  classed = structure(list(), class = "mx_thing"),
  data_frame = data.frame(a = 1),
  false_with_attribute = structure(FALSE, foo = 1),
  pairlist_untagged = pairlist(1),
  pairlist_tagged = pairlist(a = 1),
  call = quote(undefined_fn(undefined_arg)),
  call_with_attribute = structure(quote(undefined_fn(undefined_arg)), foo = 1),
  symbol = quote(undefined_symbol),
  environment = new.env()
)

test_that("has_attributes() agrees with attributes() for every case", {
  for (name in names(attribute_cases)) {
    x <- attribute_cases[[name]]
    expect_identical(sexp_has_attributes(x), !is.null(attributes(x)), label = name)
  }
})

test_that("has_attributes() distinguishes plain, named and classed objects", {
  expect_false(sexp_has_attributes(FALSE))
  expect_false(sexp_has_attributes(1:3))
  expect_true(sexp_has_attributes(c(a = 1)))
  expect_true(sexp_has_attributes(structure(list(), class = "mx_thing")))
  expect_true(sexp_has_attributes(structure(FALSE, foo = 1)))
  # A pairlist's tag names count, as attributes() reports them.
  expect_false(sexp_has_attributes(pairlist(1)))
  expect_true(sexp_has_attributes(pairlist(a = 1)))
})

identical_false_cases <- list(
  FALSE, !TRUE, as.logical(0L), TRUE, NA, 0L, 0, "FALSE", logical(0),
  c(FALSE, FALSE), c(a = FALSE), structure(FALSE, foo = 1), list(FALSE),
  factor("FALSE"), NULL
)

test_that("is_identical(x, FALSE) agrees with identical(x, FALSE)", {
  for (i in seq_along(identical_false_cases)) {
    x <- identical_false_cases[[i]]
    expect_identical(
      sexp_is_identical_false(x),
      identical(x, FALSE),
      label = paste(deparse(x), collapse = " ")
    )
  }
  expect_true(sexp_is_identical_false(FALSE))
  expect_false(sexp_is_identical_false(structure(FALSE, foo = 1)))
})

test_that("is_identical() agrees with identical() on pairs", {
  f <- function(x) x
  pairs <- list(
    list(1:3, c(1L, 2L, 3L)),
    list(1:3, c(1, 2, 3)),
    list(c(a = 1), c(a = 1)),
    list(c(a = 1), c(b = 1)),
    # attrib.as.set = TRUE: attribute order does not matter.
    list(structure(1, a = 1, b = 2), structure(1, b = 2, a = 1)),
    list(NA_real_, NaN),
    list(f, function(x) x),
    # ignore.environment = FALSE: closures in different environments differ.
    list(f, local(function(x) x))
  )
  for (pair in pairs) {
    expect_identical(
      sexp_is_identical(pair[[1]], pair[[2]]),
      identical(pair[[1]], pair[[2]]),
      label = paste(deparse(pair), collapse = " ")
    )
  }
})
