#!/usr/bin/env Rscript
# Vctrs Protocol Overhead Benchmark (D3)
#
# Measures vctrs-aware operations on miniextendr-defined vctrs types:
# vec_c, vec_slice, format, arithmetic. Compares vctrs type performance
# against raw numeric vectors.
#
# Requires: vctrs package installed.

library(miniextendr)

if (!requireNamespace("vctrs", quietly = TRUE)) {
  cat("Skipping vctrs benchmark: vctrs package not installed.\n")
  quit(save = "no", status = 0)
}

library(vctrs)

cat("\n=== Vctrs Protocol Overhead Benchmark ===\n\n")

ITERS <- 2000L

bench_one <- function(label, expr, iters = ITERS) {
  for (i in 1:5) expr
  elapsed <- system.time(for (i in seq_len(iters)) expr)[["elapsed"]]
  us_per_call <- (elapsed / iters) * 1e6
  cat(sprintf("  %-50s %8.2f us/call\n", label, us_per_call))
  invisible(us_per_call)
}

# ---------------------------------------------------------------------------
# 1. DerivedPercent (double-based vctrs type)
# ---------------------------------------------------------------------------
cat("DerivedPercent (double-based):\n")

pct_small  <- DerivedPercent$new(c(0.1, 0.5, 0.9))
pct_medium <- DerivedPercent$new(seq(0, 1, length.out = 100))
raw_small  <- c(0.1, 0.5, 0.9)
raw_medium <- seq(0, 1, length.out = 100)

# Construction
bench_one("DerivedPercent$new(3 elem)",      DerivedPercent$new(c(0.1, 0.5, 0.9)))
bench_one("raw numeric(3)",                  c(0.1, 0.5, 0.9))

# vec_c (concatenation)
bench_one("vec_c(pct, pct) small",           vec_c(pct_small, pct_small))
bench_one("c(raw, raw) small",               c(raw_small, raw_small))
bench_one("vec_c(pct, pct) medium",          vec_c(pct_medium, pct_medium))
bench_one("c(raw, raw) medium",              c(raw_medium, raw_medium))

# vec_slice (subsetting)
bench_one("vec_slice(pct, 1:2)",             vec_slice(pct_small, 1:2))
bench_one("raw[1:2]",                        raw_small[1:2])
bench_one("vec_slice(pct_medium, 1:50)",     vec_slice(pct_medium, 1:50))
bench_one("raw_medium[1:50]",               raw_medium[1:50])

# Format
bench_one("format(pct_small)",               format(pct_small))
bench_one("format(raw_small)",               format(raw_small))
bench_one("format(pct_medium)",              format(pct_medium))

cat("\n")

# ---------------------------------------------------------------------------
# 2. DerivedPoint (record-style vctrs type)
# ---------------------------------------------------------------------------
cat("DerivedPoint (record-based, x/y fields):\n")

pt_small  <- DerivedPoint$new(c(1, 2, 3), c(4, 5, 6))
pt_medium <- DerivedPoint$new(seq_len(100), seq_len(100) * 2)

# Construction
bench_one("DerivedPoint$new(3 elem)",        DerivedPoint$new(c(1,2,3), c(4,5,6)))

# vec_c
bench_one("vec_c(pt, pt) small",             vec_c(pt_small, pt_small))
bench_one("vec_c(pt, pt) medium",            vec_c(pt_medium, pt_medium))

# vec_slice
bench_one("vec_slice(pt_small, 1:2)",        vec_slice(pt_small, 1:2))
bench_one("vec_slice(pt_medium, 1:50)",      vec_slice(pt_medium, 1:50))

# Format
bench_one("format(pt_small)",                format(pt_small))
bench_one("format(pt_medium)",               format(pt_medium))

cat("\n")

# ---------------------------------------------------------------------------
# 3. DerivedTemp (arithmetic-enabled vctrs type)
# ---------------------------------------------------------------------------
cat("DerivedTemp (double-based with arithmetic):\n")

temp_small  <- DerivedTemp$new(c(20, 25, 30))
temp_medium <- DerivedTemp$new(seq(0, 100, length.out = 100))

# Construction
bench_one("DerivedTemp$new(3 elem)",         DerivedTemp$new(c(20, 25, 30)))

# Arithmetic (if supported)
tryCatch({
  bench_one("temp + temp small",             temp_small + temp_small)
  bench_one("temp + temp medium",            temp_medium + temp_medium)
  bench_one("temp * 2 small",               temp_small * 2)
  bench_one("temp * 2 medium",              temp_medium * 2)
}, error = function(e) {
  cat(sprintf("  (arithmetic not supported: %s)\n", conditionMessage(e)))
})

# Sum
tryCatch({
  bench_one("sum(temp_medium)",              sum(temp_medium))
  bench_one("sum(raw_medium)",               sum(raw_medium))
  bench_one("mean(temp_medium)",             mean(temp_medium))
  bench_one("mean(raw_medium)",              mean(raw_medium))
}, error = function(e) {
  cat(sprintf("  (math not supported: %s)\n", conditionMessage(e)))
})

cat("\n")

cat("=== Done ===\n")
