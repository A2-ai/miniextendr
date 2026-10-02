# Cross-Package Trait Dispatch Benchmarks
#
# Times key cross-package operations between producer.pkg and consumer.pkg
# with bench::mark(). bench re-evaluates each case on every iteration, picks
# the iteration count itself, and leaves out iterations that ran a garbage
# collection. Each line reports the median and minimum time per call and how
# many iterations they summarise.
#
# Requires both packages to be installed (just cross-install), which puts them
# in this checkout's tests/cross-package/.r-lib library, and the bench package
# (listed in rproject.toml, so rv's library has it).
#
# Usage (the recipe puts that library first on .libPaths()):
#   cd tests/cross-package && just bench-interop

# region: Helpers

# Print one line per case of a bench::mark() result. The case labels are the
# names of the expressions passed to bench::mark().
report <- function(marks) {
  labels <- as.character(marks$expression)
  us <- function(t) as.numeric(t) * 1e6
  for (i in seq_len(nrow(marks))) {
    cat(sprintf(
      "  %-48s  median: %7.2f us  min: %7.2f us  (%d iterations)\n",
      labels[[i]], us(marks$median[[i]]), us(marks$min[[i]]), marks$n_itr[[i]]
    ))
  }
  cat("\n")
}

# endregion

# region: Check packages

for (pkg in c("bench", "producer.pkg", "consumer.pkg")) {
  if (!requireNamespace(pkg, quietly = TRUE)) {
    stop("Package '", pkg, "' is not installed in this R library.\n",
         "producer.pkg and consumer.pkg: run `just cross-install`.\n",
         "bench: listed in rproject.toml; `rv sync` installs it.")
  }
}

library(producer.pkg)
library(consumer.pkg)

cat("Cross-Package Interop Benchmarks\n")
cat(sprintf("  bench %s: per-call time; bench picks the iteration count\n\n",
            packageVersion("bench")))

# endregion

# The cases within a section return different values, and several mutate the
# counter they are given, so bench::mark() must not compare results
# (check = FALSE).

# region: A. Object creation

cat("Object Creation\n")

report(bench::mark(
  "new_counter(0L)" = new_counter(0L),
  "new_double_counter(0L)" = new_double_counter(0L),
  check = FALSE
))

# endregion

# region: B. Trait dispatch operations (consumer calls on producer objects)

cat("Producer -> Consumer Trait Dispatch\n")

counter <- new_counter(0L)

report(bench::mark(
  "peek_value (read-only, &self)" = peek_value(counter),
  "increment_twice (&mut self x2)" = increment_twice(counter),
  "add_and_get(&mut self, i32)" = add_and_get(counter, 1L),
  "is_counter (tag query)" = is_counter(counter),
  check = FALSE
))

# endregion

# region: C. Consumer -> Producer dispatch

# DoubleCounter is created by consumer and read by producer's counter_get_value.

cat("Consumer -> Producer Trait Dispatch\n")

double_counter <- new_double_counter(0L)

report(bench::mark(
  "peek_value on DoubleCounter" = peek_value(double_counter),
  "increment_twice on DoubleCounter" = increment_twice(double_counter),
  "counter_get_value (producer reads consumer obj)" =
    counter_get_value(double_counter),
  check = FALSE
))

# endregion

# region: D. ExternalPtr pass-through (opaque cross-package pointer relay)

cat("ExternalPtr Pass-Through\n")

data <- SharedData$create(1.0, 2.0, "bench")

report(bench::mark(
  "passthrough_ptr (opaque relay)" = passthrough_ptr(data),
  "is_external_ptr check" = is_external_ptr(data),
  check = FALSE
))

# endregion

# region: E. Dispatch symmetry comparison

cat("Dispatch Symmetry (SimpleCounter vs DoubleCounter)\n")

simple <- new_counter(0L)
double <- new_double_counter(0L)

report(bench::mark(
  "increment_twice(SimpleCounter)" = increment_twice(simple),
  "increment_twice(DoubleCounter)" = increment_twice(double),
  "peek_value(SimpleCounter)" = peek_value(simple),
  "peek_value(DoubleCounter)" = peek_value(double),
  check = FALSE
))

# endregion

cat("Done.\n")
