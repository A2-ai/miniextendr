# GC stress fixture tests
#
# Exercises no-arg gc_stress_* fixtures under gctorture(TRUE) to verify
# PROTECT discipline for SEXP-storage-across-allocations paths.
#
# See docs/GCTORTURE_TESTING.md for background on the harness pattern.

# region: NamedDataFrameListBuilder -------------------------------------------

test_that("gc_stress_named_df_list_builder returns a valid named list under gctorture", {
  # Load the package first, then enable gctorture — see docs/GCTORTURE_TESTING.md.
  # Flip gctorture on for the loop, off when done regardless of failure.
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)

  ok <- 0L
  fail <- character(0L)
  for (i in seq_len(20L)) {
    res <- tryCatch(
      { gc_stress_named_df_list_builder(); "ok" },
      error = function(e) conditionMessage(e)
    )
    if (identical(res, "ok")) {
      ok <- ok + 1L
    } else {
      fail <- c(fail, sprintf("iteration %d: %s", i, res))
    }
  }

  expect_equal(ok, 20L, info = paste("failures:", paste(fail, collapse = "; ")))
})

test_that("gc_stress_named_df_list_builder returns correct structure", {
  result <- gc_stress_named_df_list_builder()
  expect_type(result, "list")
  expect_named(result, c("results", "error"))
  expect_s3_class(result[["results"]], "data.frame")
  expect_s3_class(result[["error"]], "data.frame")
  expect_equal(nrow(result[["results"]]), 50L)
  expect_equal(nrow(result[["error"]]), 20L)
  expect_identical(colnames(result[["results"]]), c("id", "value"))
  expect_identical(colnames(result[["error"]]), c("id", "msg"))
})

# endregion

# region: zero-copy &str argument borrow (#664) -------------------------------

test_that("str_borrow_len round-trips a zero-copy &str argument", {
  # `#[miniextendr] fn(s: &str)` now borrows R's CHARSXP pool directly (no
  # owning-String copy) on the main-thread path. The char count must be exact —
  # a corrupted/truncated borrow would return the wrong length.
  expect_equal(str_borrow_len(""), 0L)
  expect_equal(str_borrow_len("ascii"), 5L)
  expect_equal(str_borrow_len("héllo"), 5L) # multibyte: chars, not bytes
  expect_equal(str_borrow_len("a longer string with spaces"), 27L)
})

test_that("gc_stress_str_borrow keeps zero-copy &str views intact under gctorture", {
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)

  ok <- 0L
  fail <- character(0L)
  for (i in seq_len(20L)) {
    res <- tryCatch(
      { gc_stress_str_borrow(); "ok" },
      error = function(e) conditionMessage(e)
    )
    if (identical(res, "ok")) {
      ok <- ok + 1L
    } else {
      fail <- c(fail, sprintf("iteration %d: %s", i, res))
    }
  }

  expect_equal(ok, 20L, info = paste("failures:", paste(fail, collapse = "; ")))
})

# endregion
