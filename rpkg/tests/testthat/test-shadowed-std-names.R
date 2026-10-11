test_that("a worker wrapper compiles with Some, Ok and friends shadowed (#1991)", {
  worker <- get0("shadowed_std_names_worker", envir = asNamespace("miniextendr"))
  skip_if(is.null(worker), "built without worker-thread")
  expect_equal(worker(21L), 42L)
  expect_error(worker(-1L), "negative: -1")
})
