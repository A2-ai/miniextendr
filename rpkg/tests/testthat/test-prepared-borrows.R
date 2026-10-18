test_that("prepared branches retain fresh R storage through GC torture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  for (i in seq_len(3L)) {
    expect_null(miniextendr:::gc_stress_prepared_either())
  }
})
