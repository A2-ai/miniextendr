test_that("worker wrappers compile with the standard names shadowed (#1991)", {
  ns <- asNamespace("miniextendr")
  worker <- get0("shadowed_std_names_worker", envir = ns)
  skip_if(is.null(worker), "built without worker-thread")
  expect_equal(worker(21L), 42L)
  expect_error(worker(-1L), "negative: -1")
  expect_equal(get("shadowed_std_names_worker_str", envir = ns)("abc"), 3L)
  expect_equal(get("ShadowedWorkerCounter", envir = ns)$add_len(1L, "abcd"), 5L)
})

test_that("a serde-classed error compiles with the standard names shadowed", {
  serde <- get0("shadowed_std_names_serde", envir = asNamespace("miniextendr"))
  skip_if(is.null(serde), "built without serde")
  e <- tryCatch(serde(3L), error = function(e) e)
  expect_s3_class(e, "shadowed_refused")
  expect_equal(conditionMessage(e), "refused with 3")
  expect_equal(e$code, 3L)
})
