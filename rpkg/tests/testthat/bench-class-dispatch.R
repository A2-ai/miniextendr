#!/usr/bin/env Rscript
# Class System Method Dispatch Benchmark (D1)
#
# Compares method call overhead across 5 class systems:
# R6, S3, S4, S7, Env.
# Uses the Counter types defined in each class system's test module.
# Measures: constructor, getter, mutator, and static method.

library(miniextendr)

cat("\n=== Class System Method Dispatch Benchmark ===\n\n")

ITERS <- 10000L

bench_one <- function(label, expr) {
  # Warm up
  for (i in 1:10) expr
  # Measure
  elapsed <- system.time(for (i in seq_len(ITERS)) expr)[["elapsed"]]
  us_per_call <- (elapsed / ITERS) * 1e6
  cat(sprintf("  %-50s %8.2f us/call\n", label, us_per_call))
  invisible(us_per_call)
}

results <- list()

# ---------------------------------------------------------------------------
# 1. R6 class (R6Counter)
# ---------------------------------------------------------------------------
cat("R6 (R6Counter) — R6::R6Class dispatch:\n")
r6 <- R6Counter$new(0L)
results$r6_new      <- bench_one("R6Counter$new(0L)",          R6Counter$new(0L))
results$r6_value    <- bench_one("r6$value()",                 r6$value())
results$r6_inc      <- bench_one("r6$inc()",                   r6$inc())
results$r6_add      <- bench_one("r6$add(5L)",                 r6$add(5L))
results$r6_static   <- bench_one("R6Counter$default_counter()",R6Counter$default_counter())
cat("\n")

# ---------------------------------------------------------------------------
# 2. S3 class (S3Counter)
# ---------------------------------------------------------------------------
cat("S3 (S3Counter) — UseMethod() dispatch:\n")
s3 <- S3Counter$new(0L)
results$s3_new      <- bench_one("S3Counter$new(0L)",          S3Counter$new(0L))
results$s3_value    <- bench_one("s3_value(s3)",               s3_value(s3))
results$s3_inc      <- bench_one("s3_inc(s3)",                 s3_inc(s3))
results$s3_add      <- bench_one("s3_add(s3, 5L)",             s3_add(s3, 5L))
cat("\n")

# ---------------------------------------------------------------------------
# 3. S4 class (S4Counter)
# ---------------------------------------------------------------------------
cat("S4 (S4Counter) — standardGeneric() dispatch:\n")
s4 <- S4Counter(0L)
results$s4_new      <- bench_one("S4Counter(0L)",              S4Counter(0L))
results$s4_value    <- bench_one("s4_value(s4)",               s4_value(s4))
results$s4_inc      <- bench_one("s4_inc(s4)",                 s4_inc(s4))
results$s4_add      <- bench_one("s4_add(s4, 5L)",             s4_add(s4, 5L))
results$s4_static   <- bench_one("S4Counter_default_counter()",S4Counter_default_counter())
cat("\n")

# ---------------------------------------------------------------------------
# 4. S7 class (S7Counter)
# ---------------------------------------------------------------------------
cat("S7 (S7Counter) — S7::S7_dispatch:\n")
s7 <- S7Counter(0L)
results$s7_new      <- bench_one("S7Counter(0L)",              S7Counter(0L))
results$s7_value    <- bench_one("s7_value(s7)",               s7_value(s7))
results$s7_inc      <- bench_one("s7_inc(s7)",                 s7_inc(s7))
results$s7_add      <- bench_one("s7_add(s7, 5L)",             s7_add(s7, 5L))
cat("\n")

# ---------------------------------------------------------------------------
# 5. Env class (CounterTraitEnv)
# ---------------------------------------------------------------------------
cat("Env (CounterTraitEnv) — environment $ dispatch:\n")
env_obj <- CounterTraitEnv$new(0L)
results$env_new     <- bench_one("CounterTraitEnv$new(0L)",    CounterTraitEnv$new(0L))
results$env_value   <- bench_one("env$get_value()",            env_obj$get_value())

# Trait dispatch: env$MatrixCounter$custom_get()
results$env_trait   <- bench_one("env$MatrixCounter$custom_get()", env_obj$MatrixCounter$custom_get())
results$env_tradd   <- bench_one("env$MatrixCounter$custom_add(5)", env_obj$MatrixCounter$custom_add(5L))

# Static trait method (no dispatch needed)
results$env_static  <- bench_one("CounterTraitEnv$MatrixCounter$default_value()",
                                 CounterTraitEnv$MatrixCounter$default_value())
cat("\n")

# ---------------------------------------------------------------------------
# 6. Baseline: plain .Call (no class dispatch)
# ---------------------------------------------------------------------------
cat("Baseline — plain function (no dispatch):\n")
results$baseline    <- bench_one("add(1L, 2L)",                add(1L, 2L))
cat("\n")

# ---------------------------------------------------------------------------
# Summary table
# ---------------------------------------------------------------------------
cat("=== Summary (us/call, lower is better) ===\n\n")
cat(sprintf("  %-12s  %8s  %8s  %8s  %8s\n", "System", "new()", "get()", "mutate()", "static()"))
cat(sprintf("  %-12s  %8.2f  %8.2f  %8.2f  %8.2f\n", "R6",
  results$r6_new, results$r6_value, results$r6_add, results$r6_static))
cat(sprintf("  %-12s  %8.2f  %8.2f  %8.2f  %8s\n", "S3",
  results$s3_new, results$s3_value, results$s3_add, "N/A"))
cat(sprintf("  %-12s  %8.2f  %8.2f  %8.2f  %8.2f\n", "S4",
  results$s4_new, results$s4_value, results$s4_add, results$s4_static))
cat(sprintf("  %-12s  %8.2f  %8.2f  %8.2f  %8s\n", "S7",
  results$s7_new, results$s7_value, results$s7_add, "N/A"))
cat(sprintf("  %-12s  %8.2f  %8.2f  %8.2f  %8.2f\n", "Env",
  results$env_new, results$env_value, results$env_tradd, results$env_static))
cat(sprintf("  %-12s  %8s  %8s  %8.2f  %8s\n", "Baseline",
  "N/A", "N/A", results$baseline, "N/A"))
cat("\n")
