# GC stress fixture tests
#
# Exercises no-arg gc_stress_* fixtures under gctorture(TRUE) to verify
# PROTECT discipline for SEXP-storage-across-allocations paths.
#
# See docs/GCTORTURE_TESTING.md for background on the harness pattern.

# region: NamedDataFrameListBuilder -------------------------------------------

test_that("gc_stress_named_df_list_builder returns a valid named list under gctorture", {
  skip_gc_stress_if_disabled()
  # Load the package first, then enable gctorture — see docs/GCTORTURE_TESTING.md.
  # Flip gctorture on for the loop, off when done regardless of failure.
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)

  ok <- 0L
  fail <- character(0L)
  for (i in seq_len(20L)) {
    res <- tryCatch(
      { miniextendr:::gc_stress_named_df_list_builder(); "ok" },
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
  result <- miniextendr:::gc_stress_named_df_list_builder()
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
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)

  ok <- 0L
  fail <- character(0L)
  for (i in seq_len(20L)) {
    res <- tryCatch(
      { miniextendr:::gc_stress_str_borrow(); "ok" },
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

# region: legacy-prefix gctorture fixtures (gc_protect_tests.rs, #307) --------

# These two predate the gc_stress_ naming convention (#430), so the dynamic
# sweep below does not enumerate them — exercise them explicitly, with the
# structure assertions the sweep can't make.
test_that("List::from_values / from_pairs string fixtures survive gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)

  for (i in seq_len(5L)) {
    v <- miniextendr:::test_list_from_values_strings_gctorture()
    expect_length(v, 16L)
    expect_equal(v[[1]], "element-0")

    p <- miniextendr:::test_list_from_pairs_strings_gctorture()
    expect_length(p, 16L)
    expect_equal(names(p)[[1]], "k0")
    expect_equal(p[[1]], "v0")
  }
})

# endregion

# region: dynamic sweep — every no-arg gc_stress_* fixture --------------------

# Self-registering harness (#1026): enumerate every exported no-arg
# gc_stress_* fixture so future fixtures are exercised without editing this
# file. The explicit tests above assert result *structure*; this sweep only
# asserts survival under gctorture. Feature-gated fixtures self-solve: if not
# compiled, they are not in the namespace. Iterations stay low (5) because
# this runs in PR CI — nightly's gctorture2 sweep amplifies it.
test_that("every no-arg gc_stress_* fixture survives gctorture", {
  skip_gc_stress_if_disabled()
  ns <- getNamespace("miniextendr")
  fixtures <- ls(ns, pattern = "^gc_stress_")
  fixtures <- Filter(function(f) length(formals(get(f, ns))) == 0L, fixtures)
  # gc_stress_with_r_thread_stop raises by design (its point is the raw
  # Rf_error longjmp path; test-worker-longjmp.R expect_error()s it). Every
  # other fixture's contract is an error-free return.
  fixtures <- setdiff(fixtures, "gc_stress_with_r_thread_stop")
  # CI's r-stress-tests job splits this sweep across parallel shards via
  # MINIEXTENDR_STRESS_SHARD=k/n (helper-gc-stress.R); locally the env var
  # is unset and the full fixture list runs.
  fixtures <- gc_stress_shard_subset(fixtures)
  expect_gt(length(fixtures), 0L)

  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)

  ok <- 0L
  fail <- character(0L)
  for (f in fixtures) {
    res <- "ok"
    for (i in seq_len(5L)) {
      res <- tryCatch(
        { get(f, ns)(); "ok" },
        error = function(e) conditionMessage(e)
      )
      if (!identical(res, "ok")) {
        fail <- c(fail, sprintf("%s iteration %d: %s", f, i, res))
        break
      }
    }
    if (identical(res, "ok")) ok <- ok + 1L
  }

  expect_equal(
    ok, length(fixtures),
    info = sprintf(
      "%d of %d fixtures survived; failures: %s",
      ok, length(fixtures), paste(fail, collapse = "; ")
    )
  )
})

# endregion

# region: expression RCall builder (#430) -------------------------------------

test_that("gc_stress_expression_call survives gctorture and returns the right value", {
  skip_gc_stress_if_disabled()
  # Load the package first, then enable gctorture — see docs/GCTORTURE_TESTING.md.
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)

  ok <- 0L
  fail <- character(0L)
  for (i in seq_len(20L)) {
    res <- tryCatch(
      {
        stopifnot(identical(miniextendr:::gc_stress_expression_call(), "alpha-beta"))
        "ok"
      },
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
