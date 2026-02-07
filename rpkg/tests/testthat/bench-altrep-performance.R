# Benchmark: ALTREP vs Regular Copy Performance
# Run with: Rscript tests/testthat/bench-altrep-performance.R

library(miniextendr)

# Check if bench package is available, fallback to system.time
has_bench <- requireNamespace("bench", quietly = TRUE)

cat("\n=== ALTREP Performance Benchmarks ===\n\n")

# Test different sizes
sizes <- c(100, 1000, 10000, 100000, 1000000)

results <- data.frame(
  size = integer(),
  copy_time_ms = numeric(),
  altrep_time_ms = numeric(),
  speedup = numeric()
)

for (n in sizes) {
  cat(sprintf("Testing size: %d\n", n))

  if (has_bench) {
    # Use bench package for more accurate timing
    copy_bench <- bench::mark(
      {
        x <- rep(0L, n)
        sum(x)
      },
      iterations = 100,
      check = FALSE
    )

    altrep_bench <- bench::mark(
      {
        x <- large_vec_altrep()
        if (length(x) != 100000) stop("Wrong size")
        NULL  # Just creation, no access
      },
      iterations = 100,
      check = FALSE
    )

    copy_time <- as.numeric(copy_bench$median) * 1000  # Convert to ms
    altrep_time <- as.numeric(altrep_bench$median) * 1000

  } else {
    # Fallback to system.time
    copy_time <- system.time({
      for (i in 1:100) {
        x <- rep(0L, n)
        sum(x)
      }
    })["elapsed"] * 10  # ms per iteration

    altrep_time <- system.time({
      for (i in 1:100) {
        x <- large_vec_altrep()
        if (length(x) != 100000) stop("Wrong size")
        NULL
      }
    })["elapsed"] * 10
  }

  speedup <- copy_time / altrep_time

  results <- rbind(results, data.frame(
    size = n,
    copy_time_ms = copy_time,
    altrep_time_ms = altrep_time,
    speedup = speedup
  ))

  cat(sprintf("  Copy: %.3f ms, ALTREP: %.3f ms, Speedup: %.1fx\n",
              copy_time, altrep_time, speedup))
}

cat("\n=== Summary Table ===\n")
print(results)

# Test element access overhead
cat("\n=== Element Access Overhead ===\n\n")

if (has_bench) {
  # Regular vector access
  regular <- rep(0L, 10000)
  regular_access <- bench::mark(
    regular[5000],
    iterations = 10000,
    check = FALSE
  )

  # ALTREP vector access
  altrep <- large_vec_altrep()
  altrep_access <- bench::mark(
    altrep[5000],
    iterations = 10000,
    check = FALSE
  )

  cat(sprintf("Regular vector access: %.2f ns\n", as.numeric(regular_access$median) * 1e9))
  cat(sprintf("ALTREP vector access:  %.2f ns\n", as.numeric(altrep_access$median) * 1e9))
  cat(sprintf("Overhead: %.2f ns\n",
              (as.numeric(altrep_access$median) - as.numeric(regular_access$median)) * 1e9))
} else {
  cat("Install 'bench' package for detailed access timing\n")
}

cat("\n=== Memory Usage ===\n\n")

# Test memory usage
regular <- rep(0L, 1000000)
cat(sprintf("Regular vector (1M elements): %d bytes\n",
            as.numeric(object.size(regular))))

altrep <- large_vec_altrep()
cat(sprintf("ALTREP vector (100K elements): %d bytes\n",
            as.numeric(object.size(altrep))))

cat("\nNote: ALTREP memory is in Rust heap, not R heap\n")
cat("Use pryr::object_size() for more accurate measurements\n")
