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

test_that("gc_stress_dataframe_struct_as_list returns list-column cells", {
  df <- miniextendr:::gc_stress_dataframe_struct_as_list()
  expect_s3_class(df, "data.frame")
  expect_identical(colnames(df), c("id", "origin"))
  expect_equal(nrow(df), 8L)
  expect_type(df$origin, "list")
  expect_equal(df$origin[[8]], list(x = 7, y = 7.5))
})

test_that("DataFrameRow split and as_list fixtures stay intact under gctorture", {
  skip_gc_stress_if_disabled()
  fixtures <- c(
    "gc_stress_dataframe_split_multi_variant",
    "gc_stress_dataframe_split_nested_flatten",
    "gc_stress_dataframe_split_as_list",
    "gc_stress_dataframe_struct_as_list"
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
          parts <- if (is.data.frame(out)) list(out) else out
          if (!all(vapply(parts, is.data.frame, logical(1L)))) {
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

# region: enum DataFrameRow split held across the caller's allocations (#1750)

test_that("gc_stress_dataframe_split_held returns the held splits and the other frame", {
  res <- miniextendr:::gc_stress_dataframe_split_held()
  expect_named(res, c("split", "single", "other"))
  expect_false(is.data.frame(res$split))
  expect_named(res$split, c("click", "impression", "error"))
  for (part in res$split) expect_s3_class(part, "data.frame")
  expect_equal(res$split$click$x, c(0, 3, 6, 9))
  expect_identical(res$split$impression$slot, c("slot_1", "slot_4", "slot_7", "slot_10"))
  expect_identical(res$split$error$code, c(402L, 405L, 408L, 411L))
  expect_s3_class(res$single, "data.frame")
  expect_identical(colnames(res$single), c("x", "y"))
  expect_equal(res$single$y, c(0.5, 1.5, 2.5, 3.5))
  expect_s3_class(res$other, "data.frame")
  expect_identical(res$other$name, c("p0", "p1", "p2", "p3"))
})

test_that("a held into_dataframe_split result survives the caller's allocations under gctorture", {
  skip_gc_stress_if_disabled()
  # Load the package first, then enable gctorture — see docs/GCTORTURE_TESTING.md.
  f <- miniextendr:::gc_stress_dataframe_split_held
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)

  fail <- character(0L)
  for (i in seq_len(10L)) {
    res <- tryCatch(
      {
        out <- f()
        if (!all(vapply(c(out$split, list(out$single, out$other)), is.data.frame, logical(1L)))) {
          stop("a partition is not a data.frame")
        }
        "ok"
      },
      error = function(e) conditionMessage(e)
    )
    if (!identical(res, "ok")) {
      fail <- c(fail, sprintf("iteration %d: %s", i, res))
      break
    }
  }

  expect_identical(fail, character(0L))
})

# endregion

# region: vctrs / serde_json / raw-tagged rooting (#1759, #1760, #1761) -------

# These fixtures are feature-gated (vctrs, jiff + vctrs, serde_json,
# raw_conversions), so a build without the feature does not define them.
skip_without_fixture <- function(f) {
  if (!exists(f, envir = asNamespace("miniextendr"), inherits = FALSE)) {
    skip(paste(f, "is not compiled into this build"))
  }
}

# Each fixture checks every value in Rust and panics on a mismatch; these
# blocks pin the last round's shapes from R as well.
test_that("gc_stress_vctrs_constructors returns classed vctr, rcrd and list_of", {
  skip_without_fixture("gc_stress_vctrs_constructors")
  res <- miniextendr:::gc_stress_vctrs_constructors()
  expect_named(res, c("percent", "rational", "int_lists"))
  expect_identical(class(res$percent), c("derived_percent", "vctrs_vctr"))
  expect_identical(unclass(res$percent), c(3.25, 3.5))
  expect_identical(class(res$rational), c("derived_rational", "vctrs_rcrd", "vctrs_vctr"))
  expect_identical(unclass(res$rational), list(n = 4:9, d = c(8L, 12L, 16L, 20L, 24L, 28L)))
  expect_identical(
    class(res$int_lists),
    c("derived_int_lists", "vctrs_list_of", "vctrs_vctr", "list")
  )
  expect_identical(attr(res$int_lists, "size"), 4L)
  expect_identical(
    unclass(res$int_lists)[1:4],
    list(3L, c(3L, 4L), integer(0), 5L)
  )
})

test_that("gc_stress_jiff_rcrd returns all four jiff records", {
  skip_without_fixture("gc_stress_jiff_rcrd")
  res <- miniextendr:::gc_stress_jiff_rcrd()
  expect_named(res, c("span", "zoned", "datetime", "time"))
  for (nm in names(res)) {
    expect_identical(class(res[[nm]]), c(paste0("jiff_", nm), "vctrs_rcrd", "vctrs_vctr"))
  }
  expect_identical(unclass(res$span)$years, 4:6)
  expect_identical(unclass(res$span)$days, c(2L, 4L, 6L))
  expect_identical(unclass(res$zoned)$tz, c("UTC", "Europe/Paris", "America/New_York"))
  expect_equal(unclass(res$zoned)$timestamp, rep(1704078000, 3))
  expect_identical(unclass(res$datetime)$day, 1:3)
  expect_identical(unclass(res$time)$minute, 0:2)
})

test_that("gc_stress_json_scalar_strings returns per-call strings", {
  skip_without_fixture("gc_stress_json_scalar_strings")
  res <- miniextendr:::gc_stress_json_scalar_strings()
  expect_named(res, c("object", "array", "scalar"))
  obj <- res$object
  expect_named(obj, c("k1", "k2", "k3", "k4", "k5"))
  # Last round: slots 18-23, tags "s" to "x", then the call's 5-digit hex id.
  call_id <- substring(res$scalar, 2L)
  expect_match(call_id, "^[0-9a-f]{5}$")
  expect_identical(obj$k1, paste0("s", call_id))
  expect_identical(obj$k2, 7L)
  expect_identical(obj$k3, paste0("t", call_id))
  expect_true(obj$k4)
  expect_identical(obj$k5, list(paste0("u", call_id), 1.5, NULL))
  expect_identical(res$array, list(paste0("v", call_id), 2L, FALSE, paste0("w", call_id)))
  expect_identical(res$scalar, paste0("x", call_id))
})

test_that("gc_stress_raw_tagged returns a tagged raw vector", {
  skip_without_fixture("gc_stress_raw_tagged")
  res <- miniextendr:::gc_stress_raw_tagged()
  expect_type(res, "raw")
  expect_length(res, 48L)
  expect_identical(rawToChar(res[1:4]), "MXRB")
  expect_match(attr(res, "mx_raw_type"), "GcStressRawWords<2>$")
})

test_that("vctrs, serde_json and raw-tagged fixtures stay intact under gctorture", {
  skip_gc_stress_if_disabled()
  ns <- getNamespace("miniextendr")
  fixtures <- c(
    "gc_stress_vctrs_constructors",
    "gc_stress_jiff_rcrd",
    "gc_stress_json_scalar_strings",
    "gc_stress_raw_tagged"
  )
  fixtures <- Filter(function(f) exists(f, envir = ns, inherits = FALSE), fixtures)
  if (length(fixtures) == 0L) skip("no fixture is compiled into this build")
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)

  fail <- character(0L)
  for (f in fixtures) {
    for (i in seq_len(10L)) {
      res <- tryCatch(
        {
          get(f, ns)()
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

test_that("gc_stress_eval_error keeps the caught condition rooted under gctorture (#1861)", {
  expected <- c("held", "gc_held_error", "error", "condition", "thrower", "held")
  expect_identical(miniextendr:::gc_stress_eval_error(), expected)

  skip_gc_stress_if_disabled()
  # Load the package first, then enable gctorture — see docs/GCTORTURE_TESTING.md.
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)

  ok <- 0L
  fail <- character(0L)
  for (i in seq_len(10L)) {
    res <- tryCatch(
      {
        stopifnot(identical(miniextendr:::gc_stress_eval_error(), expected))
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
  gctorture(FALSE)

  expect_equal(ok, 10L, info = paste("failures:", paste(fail, collapse = "; ")))
})

test_that("gc_stress_try_eval_with_handlers keeps the caught condition rooted under gctorture (#1893)", {
  expected <- c("alpha-beta", "held", "gc_held_error", "error", "condition", "thrower")
  expect_identical(miniextendr:::gc_stress_try_eval_with_handlers(), expected)

  skip_gc_stress_if_disabled()
  # Load the package first, then enable gctorture — see docs/GCTORTURE_TESTING.md.
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)

  ok <- 0L
  fail <- character(0L)
  for (i in seq_len(10L)) {
    res <- tryCatch(
      {
        stopifnot(identical(miniextendr:::gc_stress_try_eval_with_handlers(), expected))
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
  gctorture(FALSE)

  expect_equal(ok, 10L, info = paste("failures:", paste(fail, collapse = "; ")))
})

# endregion

# region: unforced dots (LazyDots, #1892) -------------------------------------

test_that("gc_stress_lazy_dots keeps the dots' expressions rooted under gctorture (#1892)", {
  expected <- c(
    "5", "first,a,,,bad", "false,false,true,false,false",
    "y,call:+,,call:paste0,call:stop", "10", "3", "b2", "held",
    'argument "..3" is missing, with no default'
  )
  expect_identical(miniextendr:::gc_stress_lazy_dots(), expected)

  skip_gc_stress_if_disabled()
  # Load the package first, then enable gctorture — see docs/GCTORTURE_TESTING.md.
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)

  ok <- 0L
  fail <- character(0L)
  for (i in seq_len(10L)) {
    res <- tryCatch(
      {
        stopifnot(identical(miniextendr:::gc_stress_lazy_dots(), expected))
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
  gctorture(FALSE)

  expect_equal(ok, 10L, info = paste("failures:", paste(fail, collapse = "; ")))
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
