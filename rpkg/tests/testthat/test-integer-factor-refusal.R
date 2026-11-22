# #1660: with the R-side check off, the Rust conversion itself refuses a
# factor, so its level codes never reach the function as integers.

factor_refusal_entries <- function() {
  list(
    slice = "factor_refusal_slice",
    vec = "factor_refusal_vec",
    option_vec = "factor_refusal_option_vec",
    i64_vec = "factor_refusal_i64_vec"
  )
}

test_that("integer vector conversions refuse a factor through the wrapper and .Call", {
  ns <- asNamespace("miniextendr")
  for (name in factor_refusal_entries()) {
    wrapper <- get(name, envir = ns)
    native <- get(paste0("C_miniextendr_", name), envir = ns)
    for (f in list(factor(c("b", "a")), ordered(c("b", "a")))) {
      err <- tryCatch(wrapper(f), error = identity)
      expect_s3_class(err, "rust_error")
      expect_match(conditionMessage(err), "got a factor", fixed = TRUE, info = name)
      expect_identical(err$param, "x")
      direct <- .Call(native, NULL, f)
      expect_identical(direct$kind, "conversion", info = name)
      expect_match(direct$error, "got a factor", fixed = TRUE, info = name)
    }
  }
})

test_that("the integers of a factor still convert once extracted, and other classes pass", {
  f <- factor(c("b", "a"))
  expect_identical(miniextendr:::factor_refusal_vec(as.integer(f)), c(2L, 1L))
  expect_identical(miniextendr:::factor_refusal_slice(as.integer(f)), 2L)
  ids <- structure(c(3L, NA), class = "my_ids")
  expect_identical(miniextendr:::factor_refusal_option_vec(ids), c(3L, NA))
  expect_equal(as.double(miniextendr:::factor_refusal_i64_vec(structure(4L, class = "my_id"))), 4)
})
