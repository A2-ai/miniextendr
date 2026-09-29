# Opt-in switch for the gctorture-heavy test blocks.
#
# The gctorture blocks (mostly in test-gc-stress-fixtures.R,
# test-externalptr-self-root.R, test-iter-to-dataframe.R and
# test-dataframe-deserialize.R) account for ~94% of the suite's runtime
# (~32 of ~34 min), so they run only on request: set MINIEXTENDR_STRESS to a
# true value. `just devtools-test-stress [FILTER]` does that locally; plain
# `just devtools-test` and every CI job that runs the suite (R CMD check legs,
# CRAN-like check, r-tests and its heap-check rounds, feature legs, the webR
# smoke) leave it unset and skip them. In CI only the sharded `r-stress-tests`
# job (push-to-main / cron / dispatch, or a PR carrying the `gc-stress` label)
# and the nightly gctorture2(step=100) full-suite sweep set it.
# Run the stress blocks when a change adds a path holding SEXPs across
# allocations. See docs/GCTORTURE_TESTING.md.

# Skip the calling test unless MINIEXTENDR_STRESS is true. The value is parsed
# with as.logical(), so "true", "TRUE", "True" and "T" enable the blocks;
# anything else (unset, "false", and also "1") leaves them off. Also skips on
# CRAN: a half-hour gctorture pass is far beyond CRAN's check time budget.
skip_gc_stress_if_disabled <- function() {
  skip_on_cran()
  if (!isTRUE(as.logical(Sys.getenv("MINIEXTENDR_STRESS", "false")))) {
    skip("GC-stress block is opt-in: set MINIEXTENDR_STRESS=true or run `just devtools-test-stress`")
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
