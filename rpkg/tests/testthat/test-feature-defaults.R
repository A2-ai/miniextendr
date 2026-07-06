# Runtime assertions for the feature-controlled #[miniextendr] option defaults
# (docs/FEATURE_DEFAULTS.md) and their no_* opt-outs, driven by the fixtures in
# src/rust/feature_default_fixtures.rs. On a default build every pair behaves
# identically; the scheduled feature-legs CI job (.github/workflows/ci.yml)
# rebuilds with worker-default / strict-default / coerce-default / r6-default /
# s7-default flipped on and re-runs this file, which is the only runtime
# coverage those features have (audit A5/A10).

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
  coerce_on <- miniextendr_has_feature("coerce-default")
  # INTSXP is accepted in every mode.
  expect_equal(as.numeric(fdefault_strict_i64(1L)), 1)
  expect_equal(as.numeric(fdefault_no_strict_i64(1L)), 1)
  if (strict_on) {
    expect_error(fdefault_strict_i64(TRUE))
    expect_error(fdefault_strict_i64(as.raw(1)))
  }
  # Under coerce-default an i64 param converts via R's *integer* native type
  # (#1112), which also rejects logical/raw — so no_strict leniency is only
  # observable when coerce-default is off. The CI feature legs keep
  # strict-default and coerce-default on separate matrix rows for this reason.
  if (strict_on && !coerce_on) {
    # no_strict opts back out to the lenient multi-source conversion.
    expect_equal(as.numeric(fdefault_no_strict_i64(TRUE)), 1)
    expect_equal(as.numeric(fdefault_no_strict_i64(as.raw(1))), 1)
  }
  if (!strict_on && !coerce_on) {
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
    # Under coerce the parameter converts via R's *integer* native type, so a
    # logical input is rejected — coerce narrows the accepted SEXP types
    # rather than widening them (#1112 tracks whether that stays the design).
    expect_error(fdefault_coerce_flag(TRUE))
    # no_coerce opts back out: logical works, integer does not.
    expect_true(fdefault_no_coerce_flag(TRUE))
    expect_error(fdefault_no_coerce_flag(1L))
  } else {
    expect_true(fdefault_coerce_flag(TRUE))
    expect_true(fdefault_no_coerce_flag(TRUE))
    expect_error(fdefault_coerce_flag(1L))
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
