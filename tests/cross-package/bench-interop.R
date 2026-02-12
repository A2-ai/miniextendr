# Cross-Package Trait Dispatch Benchmarks
#
# Times key cross-package operations between producer.pkg and consumer.pkg.
# Requires both packages to be installed (just cross-install).
#
# Usage:
#   Rscript tests/cross-package/bench-interop.R

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

WARMUP   <- 50L
ITERS    <- 500L

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

bench_median <- function(label, expr, warmup = WARMUP, iters = ITERS) {
  # Warmup
  for (i in seq_len(warmup)) expr

  # Timed iterations
  times <- numeric(iters)
  for (i in seq_len(iters)) {
    t0 <- proc.time()[[3L]]
    expr
    times[i] <- proc.time()[[3L]] - t0
  }
  median_us <- median(times) * 1e6
  cat(sprintf("  %-45s  median: %8.1f us\n", label, median_us))
  invisible(median_us)
}

# ---------------------------------------------------------------------------
# Check packages
# ---------------------------------------------------------------------------

if (!requireNamespace("producer.pkg", quietly = TRUE) ||
    !requireNamespace("consumer.pkg", quietly = TRUE)) {
  stop("Both producer.pkg and consumer.pkg must be installed.\n",
       "Run: just cross-install")
}

library(producer.pkg)
library(consumer.pkg)

cat("Cross-Package Interop Benchmarks\n")
cat(sprintf("  warmup=%d  iterations=%d\n\n", WARMUP, ITERS))

# ---------------------------------------------------------------------------
# A. Object creation
# ---------------------------------------------------------------------------

cat("Object Creation\n")

bench_median("new_counter(0L)", {
  new_counter(0L)
})

bench_median("new_double_counter(0L)", {
  new_double_counter(0L)
})

cat("\n")

# ---------------------------------------------------------------------------
# B. Trait dispatch operations (consumer calls on producer objects)
# ---------------------------------------------------------------------------

cat("Producer -> Consumer Trait Dispatch\n")

counter <- new_counter(0L)

bench_median("peek_value (read-only, &self)", {
  peek_value(counter)
})

bench_median("increment_twice (&mut self x2)", {
  increment_twice(counter)
})

bench_median("add_and_get(&mut self, i32)", {
  add_and_get(counter, 1L)
})

bench_median("is_counter (tag query)", {
  is_counter(counter)
})

cat("\n")

# ---------------------------------------------------------------------------
# C. Consumer -> Producer dispatch (DoubleCounter created by consumer,
#    read by producer's counter_get_value)
# ---------------------------------------------------------------------------

cat("Consumer -> Producer Trait Dispatch\n")

double_counter <- new_double_counter(0L)

bench_median("peek_value on DoubleCounter", {
  peek_value(double_counter)
})

bench_median("increment_twice on DoubleCounter", {
  increment_twice(double_counter)
})

bench_median("counter_get_value (producer reads consumer obj)", {
  counter_get_value(double_counter)
})

cat("\n")

# ---------------------------------------------------------------------------
# D. ExternalPtr pass-through (opaque cross-package pointer relay)
# ---------------------------------------------------------------------------

cat("ExternalPtr Pass-Through\n")

data <- SharedData$create(1.0, 2.0, "bench")

bench_median("passthrough_ptr (opaque relay)", {
  passthrough_ptr(data)
})

bench_median("is_external_ptr check", {
  is_external_ptr(data)
})

cat("\n")

# ---------------------------------------------------------------------------
# E. Dispatch symmetry comparison
# ---------------------------------------------------------------------------

cat("Dispatch Symmetry (SimpleCounter vs DoubleCounter)\n")

simple  <- new_counter(0L)
double  <- new_double_counter(0L)

bench_median("increment_twice(SimpleCounter)", {
  increment_twice(simple)
})

bench_median("increment_twice(DoubleCounter)", {
  increment_twice(double)
})

bench_median("peek_value(SimpleCounter)", {
  peek_value(simple)
})

bench_median("peek_value(DoubleCounter)", {
  peek_value(double)
})

cat("\nDone.\n")
