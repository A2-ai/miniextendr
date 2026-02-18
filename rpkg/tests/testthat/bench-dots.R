#!/usr/bin/env Rscript
# Dots Collection and typed_list Validation Benchmark (D4)
#
# Measures end-to-end overhead of ... argument collection + validation
# visible from R. Compares untyped dots (no validation) vs typed_list
# validated dots at various argument counts.

library(miniextendr)

cat("\n=== Dots Collection and Validation Benchmark ===\n\n")

ITERS <- 5000L

bench_one <- function(label, expr, iters = ITERS) {
  for (i in 1:10) expr
  elapsed <- system.time(for (i in seq_len(iters)) expr)[["elapsed"]]
  us_per_call <- (elapsed / iters) * 1e6
  cat(sprintf("  %-55s %8.2f us/call\n", label, us_per_call))
  invisible(us_per_call)
}

# ---------------------------------------------------------------------------
# 1. Untyped dots (no validation, just collection)
# ---------------------------------------------------------------------------
cat("Untyped dots (greetings_with_named_dots):\n")

bench_one("0 args",   greetings_with_named_dots())
bench_one("5 args",   greetings_with_named_dots(a=1, b=2, c=3, d=4, e=5))
bench_one("20 args",  greetings_with_named_dots(
  a1=1, a2=2, a3=3, a4=4, a5=5, a6=6, a7=7, a8=8, a9=9, a10=10,
  b1=1, b2=2, b3=3, b4=4, b5=5, b6=6, b7=7, b8=8, b9=9, b10=10
))
cat("\n")

# ---------------------------------------------------------------------------
# 2. Typed dots with typed_list validation (strict mode)
# ---------------------------------------------------------------------------
cat("Typed dots — strict (validate_strict_args):\n")
cat("  Spec: @exact; x => numeric(), y => numeric()\n")

bench_one("2 args (pass)",  validate_strict_args(x = 1.0, y = 2.0))
cat("\n")

# ---------------------------------------------------------------------------
# 3. Typed dots with multi-field spec
# ---------------------------------------------------------------------------
cat("Typed dots — 3 fields (validate_numeric_args):\n")
cat("  Spec: alpha => numeric(4), beta => list(), gamma? => character()\n")

bench_one("3 args (all present)",
  validate_numeric_args(alpha = c(1,2,3,4), beta = list(1), gamma = "hi"))
bench_one("2 args (optional absent)",
  validate_numeric_args(alpha = c(1,2,3,4), beta = list(1)))
cat("\n")

# ---------------------------------------------------------------------------
# 4. Attribute-sugar typed dots
# ---------------------------------------------------------------------------
cat("Attribute-sugar dots (validate_with_attribute):\n")
cat("  Spec: x => numeric(), y => numeric()\n")

bench_one("2 args (pass)",  validate_with_attribute(x = 1.0, y = 2.0))
cat("\n")

cat("Attribute-sugar with optional (validate_attr_optional):\n")
cat("  Spec: name => character(), greeting? => character()\n")

bench_one("2 args (all present)",
  validate_attr_optional(name = "World", greeting = "Hi"))
bench_one("1 arg (optional absent)",
  validate_attr_optional(name = "World"))
cat("\n")

# ---------------------------------------------------------------------------
# 5. Validation failure cost
# ---------------------------------------------------------------------------
cat("Validation failure cost:\n")

bench_one("strict wrong type (expect error)",
  tryCatch(validate_strict_args(x = "not_numeric", y = 2.0),
           error = function(e) NULL))
bench_one("strict extra field (expect error)",
  tryCatch(validate_strict_args(x = 1.0, y = 2.0, z = 3.0),
           error = function(e) NULL))
cat("\n")

# ---------------------------------------------------------------------------
# 6. Baseline: plain function call with no dots
# ---------------------------------------------------------------------------
cat("Baseline — no-dots function call:\n")

# A simple function that takes 2 args, no dots overhead.
bench_one("add(1L, 2L)",              add(1L, 2L))
bench_one("bench_vec_copy(100L)",     bench_vec_copy(100L))
cat("\n")

cat("=== Done ===\n")
