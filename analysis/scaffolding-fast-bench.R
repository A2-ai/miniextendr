# Bench the new fast-path knobs end-to-end on installed package wrappers.
#
# Compares:
#   fast_i32_default       (full wrapper)
#   fast_i32_no_preconditions
#   fast_i32_no_call_attribution
#   fast_i32_fast          (bundle: both)
# and similarly the 3-arg shape (fast_sum3_default vs fast_sum3_fast).
#
# Run:
#   Rscript analysis/scaffolding-fast-bench.R \
#     2>&1 | tee analysis/scaffolding-fast-output.txt

suppressPackageStartupMessages({
  library(bench)
  library(miniextendr)
})

cat("# Fast-path knob bench\n\n")
cat("Generated:", format(Sys.time(), "%Y-%m-%d %H:%M:%S %Z"), "\n")
cat("R:", R.version.string, "\n")
cat("miniextendr:", as.character(packageVersion("miniextendr")), "\n")
cat("Platform:", R.version$platform, "\n")
cat("CPU:", system("sysctl -n machdep.cpu.brand_string", intern = TRUE), "\n\n")

bench_quiet <- function(...) {
  bench::mark(..., min_iterations = 20000L, check = FALSE,
              filter_gc = FALSE, time_unit = "ns")
}
to_ns <- function(x) as.numeric(x)

# ---------------------------------------------------------------------------
# 1-arg sweep — show the cumulative effect of dropping each layer.
# ---------------------------------------------------------------------------

cat("## 1-arg identity (i32)\n\n")
reps <- 5L
results <- matrix(NA_real_, nrow = 4L, ncol = reps,
                  dimnames = list(c("default", "no_preconditions",
                                    "no_call_attribution", "fast"),
                                  paste0("rep", 1:reps)))
for (i in seq_len(reps)) {
  b <- bench_quiet(
    default              = miniextendr:::fast_i32_default(42L),
    no_preconditions     = miniextendr:::fast_i32_no_preconditions(42L),
    no_call_attribution  = miniextendr:::fast_i32_no_call_attribution(42L),
    fast                 = miniextendr:::fast_i32_fast(42L)
  )
  results[, i] <- to_ns(b$min)
}

summary_tbl <- data.frame(
  variant = rownames(results),
  min  = round(apply(results, 1, min), 1),
  mean = round(apply(results, 1, mean), 1),
  median = round(apply(results, 1, median), 1),
  max  = round(apply(results, 1, max), 1)
)
summary_tbl$savings_vs_default_ns <-
  summary_tbl$min[1] - summary_tbl$min
summary_tbl$speedup_x <-
  round(summary_tbl$min[1] / summary_tbl$min, 2)
print(summary_tbl, row.names = FALSE)

# ---------------------------------------------------------------------------
# 3-arg sweep — show how stopifnot scales.
# ---------------------------------------------------------------------------

cat("\n## 3-arg sum (a+b+c)\n\n")
results3 <- matrix(NA_real_, nrow = 2L, ncol = reps,
                   dimnames = list(c("default", "fast"),
                                   paste0("rep", 1:reps)))
for (i in seq_len(reps)) {
  b <- bench_quiet(
    default = miniextendr:::fast_sum3_default(1L, 2L, 3L),
    fast    = miniextendr:::fast_sum3_fast(1L, 2L, 3L)
  )
  results3[, i] <- to_ns(b$min)
}

summary3 <- data.frame(
  variant = rownames(results3),
  min  = round(apply(results3, 1, min), 1),
  mean = round(apply(results3, 1, mean), 1),
  median = round(apply(results3, 1, median), 1),
  max  = round(apply(results3, 1, max), 1)
)
summary3$savings_vs_default_ns <- summary3$min[1] - summary3$min
summary3$speedup_x <- round(summary3$min[1] / summary3$min, 2)
print(summary3, row.names = FALSE)

# ---------------------------------------------------------------------------
# Sanity: error UX comparison
# ---------------------------------------------------------------------------

cat("\n## Error UX comparison\n\n")
cap_err <- function(expr) tryCatch(expr, error = function(e) e)

cat("default (stopifnot + match.call):\n")
e1 <- cap_err(miniextendr:::fast_i32_default("not an int"))
cat("  message:", conditionMessage(e1), "\n")
cat("  call:   ", deparse(conditionCall(e1)), "\n")
cat("  classes:", paste(head(class(e1), 5), collapse = ", "), "\n\n")

cat("fast (no stopifnot, no match.call):\n")
e2 <- cap_err(miniextendr:::fast_i32_fast("not an int"))
cat("  message:", conditionMessage(e2), "\n")
cat("  call:   ", deparse(conditionCall(e2)), "\n")
cat("  classes:", paste(head(class(e2), 5), collapse = ", "), "\n")

# ---------------------------------------------------------------------------
# R6 class-method dispatch — `fast` on the impl block.
# ---------------------------------------------------------------------------

cat("\n## R6 class-method dispatch\n\n")
ns <- getNamespace("miniextendr")
default_obj <- ns$FastCounter$new(0L)
fast_obj    <- ns$FastCounterFast$new(0L)

results_r6 <- matrix(NA_real_, nrow = 4L, ncol = reps,
                     dimnames = list(c("default_value", "fast_value",
                                       "default_add", "fast_add"),
                                     paste0("rep", 1:reps)))
for (i in seq_len(reps)) {
  b <- bench_quiet(
    default_value = default_obj$value(),
    fast_value    = fast_obj$value(),
    default_add   = default_obj$add(1L),
    fast_add      = fast_obj$add(1L)
  )
  results_r6[, i] <- to_ns(b$min)
}

summary_r6 <- data.frame(
  variant = rownames(results_r6),
  min  = round(apply(results_r6, 1, min), 1),
  mean = round(apply(results_r6, 1, mean), 1),
  median = round(apply(results_r6, 1, median), 1),
  max  = round(apply(results_r6, 1, max), 1)
)
# Pair-wise speedups
default_value_min <- summary_r6$min[summary_r6$variant == "default_value"]
fast_value_min    <- summary_r6$min[summary_r6$variant == "fast_value"]
default_add_min   <- summary_r6$min[summary_r6$variant == "default_add"]
fast_add_min      <- summary_r6$min[summary_r6$variant == "fast_add"]
summary_r6$savings_ns <- c(0,
                           default_value_min - fast_value_min,
                           0,
                           default_add_min - fast_add_min)
summary_r6$speedup_x <- c(NA,
                          round(default_value_min / fast_value_min, 2),
                          NA,
                          round(default_add_min / fast_add_min, 2))
print(summary_r6, row.names = FALSE)

cat("\n# Done.\n")
