# M13: Rprof over a representative testthat suite.
#
# Goal: does the wrapper scaffolding actually show up in a real-world
# workload, or are we optimising a microbenchmark that never accumulates?
#
# We profile test-conversions.R (768 lines, the most wrapper-heavy file
# in the suite — it exercises every TryFromSexp / IntoR path).
#
# Run:
#   Rscript analysis/scaffolding-bench-rprof.R \
#     2>&1 | tee analysis/scaffolding-bench-rprof-output.txt

suppressPackageStartupMessages({
  library(testthat)
  library(miniextendr)
})

cat("# M13: Rprof over test-conversions.R\n\n")
cat("R:", R.version.string, "\n")
cat("miniextendr:", as.character(packageVersion("miniextendr")), "\n\n")

profile_file <- tempfile(fileext = ".prof")

# Profile at 10 ms intervals so we sample enough to see the wrapper layer.
# memory.profiling so we also catch allocations.
# Function-name profiling (not line.profiling) so we see what's actually
# being called rather than test file line numbers. Faster sampling rate to
# catch the wrapper layer which executes in single-digit microseconds.
Rprof(profile_file, interval = 0.001, line.profiling = FALSE,
      memory.profiling = FALSE)

# Run the file. capture.output to suppress per-test chatter.
test_file <- "rpkg/tests/testthat/test-conversions.R"
invisible(capture.output(
  testthat::test_file(test_file, reporter = "silent")
))

Rprof(NULL)

# ---------------------------------------------------------------------------
# Report
# ---------------------------------------------------------------------------

cat("## Top by self.time\n\n")
summary_obj <- summaryRprof(profile_file)
self_tab <- summary_obj$by.self
self_tab <- head(self_tab[order(-self_tab$self.time), ], 25)
print(self_tab)

cat("\n## Top by total.time\n\n")
total_tab <- summary_obj$by.total
total_tab <- head(total_tab[order(-total_tab$total.time), ], 20)
print(total_tab)

# ---------------------------------------------------------------------------
# Look specifically for wrapper-layer functions.
# ---------------------------------------------------------------------------

cat("\n## Wrapper-layer hot spots\n\n")
wrapper_patterns <- c(
  "stopifnot",
  "match.call",
  "inherits",
  "isTRUE",
  ".miniextendr_raise_condition",
  "structure"
)

# Search by.self for wrapper-layer fns.
self_all <- summary_obj$by.self
self_idx <- vapply(rownames(self_all), function(nm)
  any(vapply(wrapper_patterns, function(p) grepl(p, nm, fixed = TRUE),
             logical(1))),
  logical(1))
wrapper_self <- self_all[self_idx, , drop = FALSE]
wrapper_self <- wrapper_self[order(-wrapper_self$self.time), ]
print(wrapper_self)

cat("\n## Total sample time vs wrapper-layer time\n")
total_time <- summary_obj$sampling.time
wrapper_time <- sum(wrapper_self$self.time)
cat(sprintf("  total elapsed:        %8.3f s\n", total_time))
cat(sprintf("  wrapper layer total:  %8.3f s (%.1f%%)\n",
            wrapper_time, 100 * wrapper_time / total_time))

cat("\n## Done.\n")
