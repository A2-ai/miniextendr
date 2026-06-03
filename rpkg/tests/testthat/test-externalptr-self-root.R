# Regression tests for ExternalPtr self-rooting (issue #836).
#
# `ExternalPtr::new` roots its EXTPTRSXP with `R_PreserveObject` for the Rust
# handle's whole lifetime, so building a `Vec<ExternalPtr<T>>` the naive way
# (`.map(ExternalPtr::new).collect()`) is GC-safe — the per-element allocation
# can no longer collect the earlier handles still held in the Vec.
#
# The Rust fixture `gc_stress_externalptr_vec()` does the naive build, churns
# the GC with a throwaway batch (exercising the Drop -> R_ReleaseObject path),
# then re-reads every handle through `R_ExternalPtrAddr`. Its internal
# assertions panic (→ R error) on any collected handle.

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
