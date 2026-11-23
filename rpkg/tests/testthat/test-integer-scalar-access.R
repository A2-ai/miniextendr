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
    list(ordered("a"), "got a factor"),
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

test_that("factor codes require explicit extraction on scalar paths", {
  for (x in list(factor("b"), ordered("b"))) {
    expect_error(miniextendr:::element_only_scalar_alias(x))
    expect_error(ReceiverCounter$new(0L)$add(x))
    expect_error(miniextendr:::coerce_input_scalar_i32(x))
    for (native in list(miniextendr:::C_miniextendr_coerce_input_scalar_i32,
                        miniextendr:::C_miniextendr_plain_na_i32)) {
      direct <- .Call(native, NULL, x)
      expect_identical(direct$kind, "conversion")
      expect_match(direct$error, "got a factor", fixed = TRUE)
    }
  }
})

test_that("an ALTREP exit drops a previously converted argument and preserves the condition", {
  inputs <- list(
    scalar_length_error = miniextendr:::element_only_ints(1L, fail_length = TRUE),
    scalar_element_error = altrep_classed_error_on_elt(1L, "scalar_element_error",
                                                       "element callback failed")
  )
  for (kind in names(inputs)) {
    before <- miniextendr:::dropped_scalar_args()
    error <- tryCatch(miniextendr:::scalar_after_drop_guard(NULL, inputs[[kind]]),
                      error = identity)
    expect_s3_class(error, kind)
    expect_identical(miniextendr:::dropped_scalar_args(), before + 1L)
  }
})
