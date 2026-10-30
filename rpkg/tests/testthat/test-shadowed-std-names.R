test_that("a serde-classed error compiles with the standard names shadowed", {
  serde <- get0("shadowed_std_names_serde", envir = asNamespace("miniextendr"))
  skip_if(is.null(serde), "built without serde")
  e <- tryCatch(serde(3L), error = function(e) e)
  expect_s3_class(e, "shadowed_refused")
  expect_equal(conditionMessage(e), "refused with 3")
  expect_equal(e$code, 3L)
})

test_that("serde_condition! compiles with the standard names shadowed (#2217)", {
  serde <- get0("shadowed_std_names_serde_condition", envir = asNamespace("miniextendr"))
  skip_if(is.null(serde), "built without serde")
  e <- tryCatch(serde(3L), error = function(e) e)
  expect_s3_class(e, c("miniextendr_error_refused", "miniextendr_error", "rust_error"))
  expect_equal(conditionMessage(e), "refused with 3")
  expect_equal(e$code, 3L)
})
