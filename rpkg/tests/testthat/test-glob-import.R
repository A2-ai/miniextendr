test_that("DataFrameRow readers compile and round-trip under a glob import (#2020)", {
  counts <- get("glob_import_roundtrip", envir = asNamespace("miniextendr"))()
  expect_equal(counts[c(1, 3, 5)], c(4L, 3L, 2L))
  par <- counts[c(2, 4, 6)]
  skip_if(all(par == -1L), "built without rayon")
  expect_equal(par, c(4L, 3L, 2L))
})

test_that("a worker wrapper compiles under a glob import", {
  worker <- get0("glob_import_worker", envir = asNamespace("miniextendr"))
  skip_if(is.null(worker), "built without worker-thread")
  expect_equal(worker("ab", c(1, 2)), 5)
  expect_error(worker("ab", numeric()), "empty")
})

test_that("a serde-classed error compiles under a glob import", {
  serde <- get0("glob_import_serde", envir = asNamespace("miniextendr"))
  skip_if(is.null(serde), "built without serde")
  e <- tryCatch(serde(3L), error = function(e) e)
  expect_s3_class(e, "glob_refused")
  expect_equal(conditionMessage(e), "refused with 3")
})
