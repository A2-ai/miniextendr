# A pending `Sidecar` value is rooted from construction until it sits in a
# slot (#2216). Fixture: `gc_stress_sidecar_pending()` in
# src/rust/gc_stress_fixtures.rs, which builds `Sidecar<SEXP>` values from
# fresh lists, in a constructor and in a rebuild hook (`Sidecar::replace`),
# churns the GC and reads them back.

test_that("gc_stress_sidecar_pending runs", {
  expect_null(miniextendr:::gc_stress_sidecar_pending())
})

test_that("pending Sidecar values survive allocation-triggered collection", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  miniextendr:::gc_stress_sidecar_pending()
  gctorture(FALSE)
  succeed()
})
