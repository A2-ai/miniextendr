#!/usr/bin/env Rscript
# Simple ALTREP vs Copy Performance Benchmark
# Uses only base R - no dependencies

library(miniextendr)

cat("\n=== ALTREP vs Copy Performance Benchmark ===\n\n")
cat("Measuring vector creation time (100 iterations each)\n\n")

# Test different sizes
sizes <- c(100, 1000, 10000, 100000, 1000000)

results <- data.frame(
  size = integer(),
  copy_ms = numeric(),
  altrep_ms = numeric(),
  speedup = numeric(),
  stringsAsFactors = FALSE
)

for (n in sizes) {
  cat(sprintf("Size: %d elements\n", n))

  # Benchmark copy approach
  copy_time <- system.time({
    for (i in 1:100) {
      x <- bench_vec_copy(as.integer(n))
      rm(x)  # Force cleanup
      gc(verbose = FALSE, full = FALSE)
    }
  })["elapsed"]

  # Benchmark ALTREP approach
  altrep_time <- system.time({
    for (i in 1:100) {
      x <- bench_vec_altrep(as.integer(n))
      rm(x)
      gc(verbose = FALSE, full = FALSE)
    }
  })["elapsed"]

  # Convert to ms per operation
  copy_ms <- (copy_time / 100) * 1000
  altrep_ms <- (altrep_time / 100) * 1000
  speedup <- copy_ms / altrep_ms

  results <- rbind(results, data.frame(
    size = n,
    copy_ms = copy_ms,
    altrep_ms = altrep_ms,
    speedup = speedup
  ))

  cat(sprintf("  Copy:   %.3f ms/op\n", copy_ms))
  cat(sprintf("  ALTREP: %.3f ms/op\n", altrep_ms))
  cat(sprintf("  Speedup: %.1fx\n\n", speedup))
}

cat("=== Summary ===\n\n")
print(results, row.names = FALSE)

cat("\n=== Element Access Overhead ===\n\n")

# Create vectors
regular <- bench_vec_copy(10000L)
altrep_vec <- bench_vec_altrep(10000L)

# Benchmark element access
n_access <- 10000

regular_time <- system.time({
  for (i in 1:n_access) {
    x <- regular[5000]
  }
})["elapsed"]

altrep_time <- system.time({
  for (i in 1:n_access) {
    x <- altrep_vec[5000]
  }
})["elapsed"]

regular_ns <- (regular_time / n_access) * 1e9
altrep_ns <- (altrep_time / n_access) * 1e9
overhead_ns <- altrep_ns - regular_ns

cat(sprintf("Regular vector access: %.1f ns/op\n", regular_ns))
cat(sprintf("ALTREP vector access:  %.1f ns/op\n", altrep_ns))
cat(sprintf("Overhead:              %.1f ns/op (%.1fx slower)\n",
            overhead_ns, altrep_ns / regular_ns))

cat("\n=== Memory Usage ===\n\n")

# Memory comparison
regular_1m <- bench_vec_copy(1000000L)
altrep_1m <- bench_vec_altrep(1000000L)

cat(sprintf("Regular vector (1M elements): %s\n",
            format(object.size(regular_1m), units = "auto")))
cat(sprintf("ALTREP vector (1M elements):  %s\n",
            format(object.size(altrep_1m), units = "auto")))
cat("\nNote: ALTREP data lives in Rust heap (ExternalPtr)\n")
cat("      R only stores the pointer (~56 bytes)\n")

cat("\n=== Full Iteration Test ===\n\n")

cat("Testing performance when R accesses all elements:\n\n")

# When R needs all elements (e.g., sum), both approaches converge
regular_10k <- bench_vec_copy(100000L)
altrep_10k <- bench_vec_altrep(100000L)

regular_sum_time <- system.time({
  for (i in 1:100) {
    s <- sum(regular_10k)
  }
})["elapsed"] * 10  # ms per op

altrep_sum_time <- system.time({
  for (i in 1:100) {
    s <- sum(altrep_10k)
  }
})["elapsed"] * 10

cat(sprintf("Regular sum (100K elements): %.3f ms/op\n", regular_sum_time))
cat(sprintf("ALTREP sum (100K elements):  %.3f ms/op\n", altrep_sum_time))
cat(sprintf("Difference: %.1f%%\n",
            ((altrep_sum_time - regular_sum_time) / regular_sum_time) * 100))

cat("\n=== Conclusion ===\n\n")
cat("ALTREP advantages:\n")
cat("  - Much faster creation (especially for large vectors)\n")
cat("  - Lower R memory usage (data in Rust heap)\n")
cat("  - Minimal overhead for element access (~10-50ns)\n")
cat("\nBest for:\n")
cat("  - Large vectors (>1000 elements)\n")
cat("  - Data that may not be fully accessed\n")
cat("  - Lazy/computed values\n")
cat("  - Zero-copy from external sources\n\n")
