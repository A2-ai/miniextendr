#!/usr/bin/env Rscript
# ALTREP vs Eager SEXP Benchmark (D2)
#
# Compares into_sexp() (eager materialization) vs into_altrep() (lazy) for
# integer and real vectors at various sizes. Measures creation time, element
# access, full iteration (sum), and length query.

library(miniextendr)

cat("\n=== ALTREP vs Eager SEXP Benchmark ===\n\n")

ITERS <- 100L
sizes <- c(1000L, 100000L, 1000000L)

bench_one <- function(label, expr, iters = ITERS) {
  # Warm up
  for (i in 1:5) expr
  elapsed <- system.time(for (i in seq_len(iters)) expr)[["elapsed"]]
  us_per_call <- (elapsed / iters) * 1e6
  cat(sprintf("  %-50s %10.1f us/call\n", label, us_per_call))
  invisible(us_per_call)
}

# ---------------------------------------------------------------------------
# 1. Integer vectors: bench_vec_copy vs bench_vec_altrep
# ---------------------------------------------------------------------------
cat("Integer vectors (copy vs ALTREP):\n\n")

for (n in sizes) {
  cat(sprintf("  --- n = %s ---\n", format(n, big.mark = ",")))

  # Creation
  bench_one(sprintf("copy  create (n=%d)", n),   bench_vec_copy(n))
  bench_one(sprintf("altrep create (n=%d)", n),  bench_vec_altrep(n))

  # Sum (full iteration)
  x_copy   <- bench_vec_copy(n)
  x_altrep <- bench_vec_altrep(n)
  bench_one(sprintf("copy  sum    (n=%d)", n),   sum(x_copy))
  bench_one(sprintf("altrep sum   (n=%d)", n),   sum(x_altrep))

  # Single element access (first + last)
  bench_one(sprintf("copy  [1]    (n=%d)", n),   x_copy[1L])
  bench_one(sprintf("altrep [1]   (n=%d)", n),   x_altrep[1L])

  # Length
  bench_one(sprintf("copy  length (n=%d)", n),   length(x_copy))
  bench_one(sprintf("altrep length(n=%d)", n),   length(x_altrep))

  cat("\n")
}

# ---------------------------------------------------------------------------
# 2. Real vectors: boxed_reals (ALTREP) vs into_sexp (copy)
# ---------------------------------------------------------------------------
cat("Real vectors (boxed ALTREP vs eager):\n\n")

for (n in sizes) {
  cat(sprintf("  --- n = %s ---\n", format(n, big.mark = ",")))

  # Creation
  bench_one(sprintf("copy   create (n=%d)", n),  altrep_from_doubles(as.double(seq_len(n))))
  bench_one(sprintf("altrep create (n=%d)", n),  boxed_reals(n))

  # Sum
  x_copy   <- altrep_from_doubles(as.double(seq_len(n)))
  x_altrep <- boxed_reals(n)
  bench_one(sprintf("copy   sum    (n=%d)", n),  sum(x_copy))
  bench_one(sprintf("altrep sum    (n=%d)", n),  sum(x_altrep))

  # Single element
  bench_one(sprintf("copy   [1]    (n=%d)", n),  x_copy[1L])
  bench_one(sprintf("altrep [1]    (n=%d)", n),  x_altrep[1L])

  cat("\n")
}

# ---------------------------------------------------------------------------
# 3. Arithmetic sequences (compute-on-access ALTREP) vs materialized
# ---------------------------------------------------------------------------
cat("Arithmetic sequence ALTREP vs materialized:\n\n")

for (n in sizes) {
  cat(sprintf("  --- n = %s ---\n", format(n, big.mark = ",")))

  # Creation
  bench_one(sprintf("seq()        create (n=%d)", n),  seq(1.0, by = 0.5, length.out = n))
  bench_one(sprintf("arith_seq()  create (n=%d)", n),  arith_seq(1.0, 0.5, n))

  # Sum
  x_r     <- seq(1.0, by = 0.5, length.out = n)
  x_altrep <- arith_seq(1.0, 0.5, n)
  bench_one(sprintf("seq()        sum    (n=%d)", n),  sum(x_r))
  bench_one(sprintf("arith_seq()  sum    (n=%d)", n),  sum(x_altrep))

  # Element access
  bench_one(sprintf("seq()        [1]    (n=%d)", n),  x_r[1L])
  bench_one(sprintf("arith_seq()  [1]    (n=%d)", n),  x_altrep[1L])

  cat("\n")
}

cat("=== Done ===\n")
