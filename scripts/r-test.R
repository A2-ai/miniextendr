#!/usr/bin/env Rscript

# Run a package's testthat suite for the just test recipes (devtools-test,
# devtools-test-stress, minirextendr-test, minirextendr-roundtrip).
#
# Every test runs, and the exit status says whether any failed:
#   - set_max_fails(Inf): testthat never stops early, so one run reports every
#     failure.
#   - stop_on_failure = TRUE: checked once, after the whole run; if any test
#     failed or errored, devtools::test() signals "Test failures." and Rscript
#     exits 1. It is not fail-fast. devtools::test() defaults to FALSE, which
#     exits 0 with failures.
#
# Usage:
#   Rscript scripts/r-test.R <package_dir> [filter]
# An empty filter runs the whole suite.

args <- commandArgs(trailingOnly = TRUE)
if (length(args) < 1L || length(args) > 2L) {
  stop("usage: Rscript scripts/r-test.R <package_dir> [filter]", call. = FALSE)
}
pkg <- args[[1L]]
filter <- if (length(args) == 2L && nzchar(args[[2L]])) args[[2L]]

testthat::set_max_fails(Inf)
devtools::test(pkg, filter = filter, stop_on_failure = TRUE)
