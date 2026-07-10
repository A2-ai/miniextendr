# CI orchestration for the gctorture-heavy test blocks.
#
# The gctorture blocks in test-gc-stress-fixtures.R, test-externalptr-self-root.R,
# test-iter-to-dataframe.R and test-dataframe-deserialize.R account for ~94% of
# the suite's runtime (~32 of ~34 min). CI runs them exactly once per PR, in the
# dedicated sharded `r-stress-tests` job; every other job that runs the suite
# (R CMD check legs, CRAN-like check, r-tests, heap-check rounds) sets
# MINIEXTENDR_SKIP_STRESS=1 so it only pays for the fast tests. Local
# `just devtools-test` sets neither variable and runs everything, unsharded.
# The nightly gctorture2(step=100) full-suite sweep also runs everything.
# See docs/GCTORTURE_TESTING.md.

# Skip the calling test when GC-stress work is delegated to another CI job.
# Also skips on CRAN: a half-hour gctorture pass is far beyond CRAN's check
# time budget, and the nightly sweep is the real deep net.
skip_gc_stress_if_disabled <- function() {
  skip_on_cran()
  if (nzchar(Sys.getenv("MINIEXTENDR_SKIP_STRESS"))) {
    skip("GC-stress block skipped: MINIEXTENDR_SKIP_STRESS is set (runs in the r-stress-tests CI job)")
  }
}

# Parse MINIEXTENDR_STRESS_SHARD="k/n" into c(k, n), or NULL when unset.
# Used by the dynamic fixture sweep to split its fixture list across the
# parallel r-stress-tests shards. Malformed values error loudly rather than
# silently running everything (a typo'd shard spec must not double coverage
# in one shard and drop it in another).
gc_stress_shard <- function() {
  spec <- Sys.getenv("MINIEXTENDR_STRESS_SHARD", "")
  if (!nzchar(spec)) {
    return(NULL)
  }
  parts <- strsplit(spec, "/", fixed = TRUE)[[1]]
  k <- suppressWarnings(as.integer(parts[[1]]))
  n <- suppressWarnings(as.integer(parts[[length(parts)]]))
  if (length(parts) != 2L || is.na(k) || is.na(n) || n < 1L || k < 1L || k > n) {
    stop("MINIEXTENDR_STRESS_SHARD must be 'k/n' with 1 <= k <= n, got: ", spec)
  }
  c(k, n)
}

# Subset a vector to this process's shard (round-robin by index), or return
# it unchanged when sharding is inactive.
gc_stress_shard_subset <- function(x) {
  shard <- gc_stress_shard()
  if (is.null(shard)) {
    return(x)
  }
  x[seq_along(x) %% shard[[2]] == shard[[1]] %% shard[[2]]]
}
