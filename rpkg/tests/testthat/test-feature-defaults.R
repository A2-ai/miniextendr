# Runtime assertions for the feature-controlled #[miniextendr] option defaults
# (docs/FEATURE_DEFAULTS.md) and their opt-outs, driven by the fixtures in
# src/rust/feature_default_fixtures.rs. On a default build every pair behaves
# identically; the scheduled feature-legs CI job (.github/workflows/ci.yml)
# rebuilds with worker-default / strict-default / coerce-default /
# no-preconditions-default / r6-default / s7-default flipped on and re-runs this file, which is the only
# runtime coverage those features have (audit A5/A10).

test_that("worker-default routes bare functions to the worker thread", {
  bare <- fdefault_worker_thread_name()
  pinned <- fdefault_no_worker_thread_name()
  if (miniextendr_has_feature("worker-default")) {
    expect_equal(bare, "miniextendr-worker")
    expect_false(identical(bare, pinned))
  } else {
    expect_identical(bare, pinned)
  }
})

test_that("strict-default rejects logical/raw inputs to i64 params", {
  strict_on <- miniextendr_has_feature("strict-default")
  # INTSXP is accepted in every mode.
  expect_equal(as.numeric(fdefault_strict_i64(1L)), 1)
  expect_equal(as.numeric(fdefault_no_strict_i64(1L)), 1)
  if (strict_on) {
    expect_error(fdefault_strict_i64(TRUE))
    expect_error(fdefault_strict_i64(as.raw(1)))
  }
  # no_strict restores multi-source conversion even with coerce-default enabled.
  expect_equal(as.numeric(fdefault_no_strict_i64(TRUE)), 1)
  expect_equal(as.numeric(fdefault_no_strict_i64(as.raw(1))), 1)
  if (!strict_on) {
    expect_equal(as.numeric(fdefault_strict_i64(TRUE)), 1)
    expect_equal(as.numeric(fdefault_strict_i64(as.raw(1))), 1)
  }
})

test_that("coerce-default converts bool params from R integers", {
  if (miniextendr_has_feature("coerce-default")) {
    expect_true(fdefault_coerce_flag(1L))
    expect_false(fdefault_coerce_flag(0L))
    # Only 0/1 coerce to bool.
    expect_error(fdefault_coerce_flag(2L))
    # Logical inputs remain accepted when integer coercion is enabled.
    expect_true(fdefault_coerce_flag(TRUE))
    # no_coerce opts back out: logical works, integer does not.
    expect_true(fdefault_no_coerce_flag(TRUE))
    expect_error(fdefault_no_coerce_flag(1L))
  } else {
    expect_true(fdefault_coerce_flag(TRUE))
    expect_true(fdefault_no_coerce_flag(TRUE))
    expect_error(fdefault_coerce_flag(1L))
  }
})

test_that("no-preconditions-default drops preconditions for bare fns, preconditions restores them", {
  unchecked <- miniextendr_has_feature("no-preconditions-default")
  # coerce-default widens a bare `i32` to whole-number doubles (and logical /
  # raw), so both the R-side check and the conversion name the wider input.
  coerced <- miniextendr_has_feature("coerce-default")
  check_message <- if (coerced) {
    "'x' must be integer or whole-number numeric"
  } else {
    "'x' must be integer"
  }
  conversion_message <- if (coerced) {
    "'x' must be a single whole number: got character"
  } else {
    "'x' must be a single integer: got character"
  }
  if (coerced) {
    expect_identical(fdefault_bare_i32(2), 2L)
    expect_identical(fdefault_checks_restored_i32(2), 2L)
  }
  # Both paths raise the same argument-error condition (#1591); they differ
  # in which side words it.
  if (unchecked) {
    # The R-side check is gone under no-preconditions-default; TryFromSexp still rejects
    # bad input, worded by the conversion.
    e <- tryCatch(fdefault_bare_i32("nope"), error = function(e) e)
    expect_s3_class(e, "rust_error")
    expect_identical(conditionMessage(e), conversion_message)
    expect_identical(e$rust_type, "i32")
    # `preconditions` opts back out: the checks are restored, so the R-side check
    # words the error, and no Rust type is involved.
    e2 <- tryCatch(fdefault_checks_restored_i32("nope"), error = function(e) e)
    expect_s3_class(e2, "rust_error")
    expect_identical(conditionMessage(e2), check_message)
    expect_null(e2$rust_type)
  } else {
    # preconditions are on (every build without no-preconditions-default):
    # the bare fn's R-side check words the error.
    e <- tryCatch(fdefault_bare_i32("nope"), error = function(e) e)
    expect_s3_class(e, "rust_error")
    expect_identical(e$kind, "conversion")
    expect_identical(e$param, "x")
    expect_identical(conditionMessage(e), check_message)
    expect_null(e$rust_type)
  }
})

test_that("class-system default selects Env/R6/S7 for the bare probe impl", {
  if (miniextendr_has_feature("r6-default")) {
    expect_true(inherits(FdefaultProbe, "R6ClassGenerator"))
    obj <- FdefaultProbe$new(7L)
    expect_equal(obj$probe_value(), 7L)
  } else if (miniextendr_has_feature("s7-default")) {
    expect_true(inherits(FdefaultProbe, "S7_class"))
    obj <- FdefaultProbe(7L)
    # ::: — the S7 shortcut only exists in s7-default builds, so it is never in
    # the committed (default-build) NAMESPACE exports.
    expect_equal(miniextendr:::FdefaultProbe_probe_value(obj), 7L)
  } else {
    expect_false(inherits(FdefaultProbe, "R6ClassGenerator"))
    obj <- FdefaultProbe$new(7L)
    expect_equal(obj$probe_value(), 7L)
  }
})

test_that("growth-debug records growth events", {
  skip_if_missing_feature("growth-debug")
  expect_equal(growth_debug_test(), 3L)
})
