test_that("a generated scalar i32 entry reads an element-only ALTREP", {
  scalar <- miniextendr:::coerce_input_native_default
  native <- miniextendr:::C_miniextendr_coerce_input_native_default
  only <- miniextendr:::element_only_ints
  x <- only(37L)

  expect_identical(scalar(x), 37L)
  expect_identical(.Call(native, NULL, x), 37L)
  expect_identical(miniextendr:::element_only_scalar_alias(x), 37L)
  counter <- ReceiverCounter$new(5L)
  expect_identical(counter$add(x), 42L)
  # Control: explicitly asking this provider for contiguous storage must fail.
  expect_error(altrep_sexp_materialize_int(x), "data pointer")
  # The integer branch of an explicitly coerced entry can read it too.
  expect_identical(miniextendr:::coerce_input_scalar_i32(x), 37L)
})

test_that("integer admission also holds at the registered .Call entry", {
  scalar <- miniextendr:::coerce_input_native_default
  native <- miniextendr:::C_miniextendr_coerce_input_native_default
  only <- miniextendr:::element_only_ints
  f <- factor("b", levels = c("a", "b"))
  expect_identical(scalar(as.integer(f)), 2L)
  expect_identical(miniextendr:::coerce_input_scalar_i32(TRUE), 1L)

  cases <- list(
    list(f, "got a factor"),
    list(TRUE, "got logical"),
    list(37, "got numeric"),
    list(only(NA_integer_), "NA is not allowed"),
    list(only(integer()), "got length 0"),
    list(only(c(1L, 2L)), "got length 2")
  )
  for (case in cases) {
    expect_error(scalar(case[[1L]]))
    direct <- .Call(native, NULL, case[[1L]])
    expect_identical(direct$kind, "conversion")
    expect_match(direct$error, case[[2L]], fixed = TRUE)
    expect_identical(direct$data$param, "x")
  }
})
