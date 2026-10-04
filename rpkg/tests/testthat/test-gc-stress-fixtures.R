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

# region: enum DataFrameRow split partitions (#1748) --------------------------

# Each fixture checks its partitions in Rust and panics on a mismatch; these
# blocks pin the shapes from R as well.
test_that("gc_stress_dataframe_split_multi_variant returns one data.frame per variant", {
  res <- miniextendr:::gc_stress_dataframe_split_multi_variant()
  expect_type(res, "list")
  expect_false(is.data.frame(res))
  expect_named(res, c("click", "impression", "error"))
  for (part in res) expect_s3_class(part, "data.frame")
  expect_identical(colnames(res$click), c("id", "x", "y"))
  expect_equal(res$click$x, c(0, 3, 6, 9))
  expect_identical(res$impression$slot, c("slot_1", "slot_4", "slot_7", "slot_10"))
  expect_identical(res$error$code, c(402L, 405L, 408L, 411L))
})

test_that("gc_stress_dataframe_split_nested_flatten returns flattened partitions", {
  res <- miniextendr:::gc_stress_dataframe_split_nested_flatten()
  expect_named(res, c("tracked", "other"))
  expect_s3_class(res$tracked, "data.frame")
  expect_s3_class(res$other, "data.frame")
  expect_identical(colnames(res$tracked), c("id", "status_variant", "status_code"))
  expect_equal(nrow(res$tracked), 2L)
  expect_identical(res$tracked$status_variant, c("Ok", "Err"))
  expect_identical(res$tracked$status_code, c(NA, 401L))
  expect_identical(res$other$id, 2:7)
})

test_that("gc_stress_dataframe_split_as_list returns list-column cells", {
  res <- miniextendr:::gc_stress_dataframe_split_as_list()
  expect_named(res, c("located", "other"))
  expect_s3_class(res$located, "data.frame")
  expect_equal(nrow(res$located), 6L)
  expect_type(res$located$origin, "list")
  expect_equal(res$located$origin[[6]], list(x = 6, y = 6.5))
  expect_equal(nrow(res$other), 2L)
})

test_that("enum split fixtures keep every partition intact under gctorture", {
  skip_gc_stress_if_disabled()
  fixtures <- c(
    "gc_stress_dataframe_split_multi_variant",
    "gc_stress_dataframe_split_nested_flatten",
    "gc_stress_dataframe_split_as_list"
  )
  ns <- getNamespace("miniextendr")
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)

  fail <- character(0L)
  for (f in fixtures) {
    for (i in seq_len(10L)) {
      res <- tryCatch(
        {
          out <- get(f, ns)()
          if (!all(vapply(out, is.data.frame, logical(1L)))) {
            stop("a partition is not a data.frame")
          }
          "ok"
        },
        error = function(e) conditionMessage(e)
      )
      if (!identical(res, "ok")) {
        fail <- c(fail, sprintf("%s iteration %d: %s", f, i, res))
        break
      }
    }
  }

  expect_identical(fail, character(0L))
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

# region: trait View argument rooting -----------------------------------------

# The View behind a `#[miniextendr]` trait converts each argument to a SEXP
# before the vtable call; every converted argument has to stay rooted while
# the later ones allocate and while the shim converts them back.
trait_view_args_expected <- 'host|first|second|[1.5, 2.5]|["x", "y"]|last'

test_that("gc_stress_trait_view_args passes every argument through the View", {
  expect_identical(miniextendr:::gc_stress_trait_view_args(), trait_view_args_expected)
})

test_that("gc_stress_trait_view_args keeps View arguments rooted under gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)

  ok <- 0L
  fail <- character(0L)
  for (i in seq_len(20L)) {
    res <- tryCatch(
      {
        out <- miniextendr:::gc_stress_trait_view_args()
        if (identical(out, trait_view_args_expected)) "ok" else paste("got", out)
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
