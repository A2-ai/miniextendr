# Regression tests for ExternalPtr self-rooting (issue #836).
#
# `ExternalPtr::new` roots its EXTPTRSXP in a process-wide `ProtectPool` (a
# GC-traced VECSXP with O(1) any-order release) for the Rust handle's whole
# lifetime, so building a `Vec<ExternalPtr<T>>` the naive way
# (`.map(ExternalPtr::new).collect()`) is GC-safe — the per-element allocation
# can no longer collect the earlier handles still held in the Vec.
#
# The Rust fixture `gc_stress_externalptr_vec()` does the naive build, churns
# the GC with a throwaway batch (exercising the Drop -> pool-release path),
# then re-reads every handle through `R_ExternalPtrAddr`. Its internal
# assertions panic (→ R error) on any collected handle.
#
# `gc_stress_externalptr_collect_list()` covers the *destination-rooted* bulk
# builder `ExternalPtr::collect_into_r_list`, which roots each element via the
# protected result list instead of the pool (no per-element pool traffic).

test_that("naive Vec<ExternalPtr> construction runs without corruption", {
  expect_null(gc_stress_externalptr_vec())
})

test_that("naive Vec<ExternalPtr> construction survives gctorture", {
  # The real coverage is the gctorture sweep over no-arg exports (see
  # docs/GCTORTURE_TESTING.md), but a few torture iterations here give CI a
  # direct regression guard. The fixture is small, so this stays cheap.
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  for (i in seq_len(3)) {
    expect_null(gc_stress_externalptr_vec())
  }
})

test_that("collect_into_r_list bulk build runs without corruption", {
  expect_null(gc_stress_externalptr_collect_list())
})

test_that("collect_into_r_list bulk build survives gctorture", {
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  for (i in seq_len(3)) {
    expect_null(gc_stress_externalptr_collect_list())
  }
})
