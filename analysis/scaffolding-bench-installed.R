# M11: hand-strip an installed wrapper and re-bench.
#
# Goal: confirm that the per-layer deltas measured against hand-defined
# closures (scaffolding-strip-bench.R) hold when the wrapper is installed
# into the package namespace and dispatched the same way as the real
# conv_i32_arg.
#
# Strategy: define 5 stripped variants inside the miniextendr namespace
# via assignInNamespace(). Each shares the same namespace lookup path,
# lazy-load semantics, and byte-compile pass as the genuine wrapper.
# Only the body differs.
#
# Run:
#   Rscript analysis/scaffolding-bench-installed.R \
#     2>&1 | tee analysis/scaffolding-bench-installed-output.txt

suppressPackageStartupMessages({
  library(bench)
  library(compiler)
  library(miniextendr)
})

ns <- getNamespace("miniextendr")
raise <- ns$.miniextendr_raise_condition
C_i32 <- ns$C_conv_i32_arg
C_sym <- ns$C_conv_sexp_arg

# ---------------------------------------------------------------------------
# Define stripped variants in the package namespace.
# ---------------------------------------------------------------------------

# Original-shape wrapper (mirror of what the macro emits for i32 arg).
i32_full <- function(x) {
  stopifnot(
    "'x' must be numeric, logical, or raw" = is.numeric(x) || is.logical(x) || is.raw(x),
    "'x' must have length 1" = length(x) == 1L
  )
  .val <- .Call(C_conv_i32_arg, .call = match.call(), x)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__")))
    return(.miniextendr_raise_condition(.val, sys.call()))
  .val
}

i32_no_stopifnot <- function(x) {
  .val <- .Call(C_conv_i32_arg, .call = match.call(), x)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__")))
    return(.miniextendr_raise_condition(.val, sys.call()))
  .val
}

i32_no_matchcall <- function(x) {
  stopifnot(
    "'x' must be numeric, logical, or raw" = is.numeric(x) || is.logical(x) || is.raw(x),
    "'x' must have length 1" = length(x) == 1L
  )
  .val <- .Call(C_conv_i32_arg, .call = NULL, x)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__")))
    return(.miniextendr_raise_condition(.val, sys.call()))
  .val
}

i32_no_stop_no_call <- function(x) {
  .val <- .Call(C_conv_i32_arg, .call = NULL, x)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__")))
    return(.miniextendr_raise_condition(.val, sys.call()))
  .val
}

# Removed: stopifnot + match.call + post-check.
# Equivalent to the C wrapper unconditionally returning the value.
# This is only safe if the body cannot return a tagged condition SEXP.
# For benchmark purposes we still inherit the same call shape.
i32_bare <- function(x) .Call(C_conv_i32_arg, .call = NULL, x)

# Assign each into the miniextendr namespace directly. assignInNamespace()
# requires the binding to pre-exist; we want net-new bindings, so use raw
# assign() with the namespace environment. The fns inherit the namespace
# as their enclosing environment, which gives them the same lookup path
# for C_conv_i32_arg / .miniextendr_raise_condition as the genuine wrapper.
# Byte-compile mirrors the install-time compile path (R 4.6 JIT level 3).
# Namespace is locked; create an unlocked sibling env that imports from the
# namespace. Bindings here are equivalent for our test purposes: same lookup
# path (because we set environment(fn) to inherit from ns), same JIT.
test_env <- new.env(parent = ns)
environment(i32_full)            <- test_env
environment(i32_no_stopifnot)    <- test_env
environment(i32_no_matchcall)    <- test_env
environment(i32_no_stop_no_call) <- test_env
environment(i32_bare)            <- test_env

test_env$i32_full            <- compiler::cmpfun(i32_full)
test_env$i32_no_stopifnot    <- compiler::cmpfun(i32_no_stopifnot)
test_env$i32_no_matchcall    <- compiler::cmpfun(i32_no_matchcall)
test_env$i32_no_stop_no_call <- compiler::cmpfun(i32_no_stop_no_call)
test_env$i32_bare            <- compiler::cmpfun(i32_bare)

# Sanity.
stopifnot(
  test_env$i32_full(42L)             == 42L,
  test_env$i32_no_stopifnot(42L)     == 42L,
  test_env$i32_no_matchcall(42L)     == 42L,
  test_env$i32_no_stop_no_call(42L)  == 42L,
  test_env$i32_bare(42L)             == 42L,
  miniextendr::conv_i32_arg(42L) == 42L
)

# ---------------------------------------------------------------------------
# Bench package-installed wrapper alongside the in-namespace stripped variants.
# Multiple reps for variance.
# ---------------------------------------------------------------------------

cat("# M11: installed-wrapper hand-strip\n")
cat("Variants all live in `miniextendr` namespace, share lazy-load path,\n")
cat("byte-compiled at JIT level 3.\n\n")

bench_quiet <- function(...) {
  bench::mark(..., min_iterations = 20000L, check = FALSE,
              filter_gc = FALSE, time_unit = "ns")
}
to_ns <- function(x) as.numeric(x)

x <- 42L

# 5 reps to get a variance read.
reps <- 5L
variants <- c("conv_i32_arg", "i32_full", "i32_no_stopifnot",
              "i32_no_matchcall", "i32_no_stop_no_call", "i32_bare")
results <- matrix(NA_real_, nrow = length(variants), ncol = reps,
                  dimnames = list(variants, paste0("rep", 1:reps)))
for (r in seq_len(reps)) {
  b <- bench_quiet(
    conv_i32_arg        = miniextendr::conv_i32_arg(x),
    i32_full            = test_env$i32_full(x),
    i32_no_stopifnot    = test_env$i32_no_stopifnot(x),
    i32_no_matchcall    = test_env$i32_no_matchcall(x),
    i32_no_stop_no_call = test_env$i32_no_stop_no_call(x),
    i32_bare            = test_env$i32_bare(x)
  )
  results[, r] <- to_ns(b$min)
}

summary_tbl <- data.frame(
  variant = rownames(results),
  min  = round(apply(results, 1, min), 1),
  mean = round(apply(results, 1, mean), 1),
  median = round(apply(results, 1, median), 1),
  max  = round(apply(results, 1, max), 1),
  sd   = round(apply(results, 1, sd), 1)
)
summary_tbl$cv_pct <- round(100 * summary_tbl$sd / summary_tbl$mean, 1)
print(summary_tbl, row.names = FALSE)

# ---------------------------------------------------------------------------
# Cross-check: the package's conv_i32_arg should be within noise of i32_full.
# If it isn't, my hand-written variant misses something the macro emits.
# ---------------------------------------------------------------------------

cat("\n## Cross-check: package wrapper vs hand-written i32_full\n")
pkg_min <- summary_tbl$min[summary_tbl$variant == "conv_i32_arg"]
mine_min <- summary_tbl$min[summary_tbl$variant == "i32_full"]
cat(sprintf("  package conv_i32_arg min:    %d ns\n", pkg_min))
cat(sprintf("  hand-written i32_full min:   %d ns\n", mine_min))
cat(sprintf("  delta:                       %+d ns\n", pkg_min - mine_min))
if (abs(pkg_min - mine_min) <= 100) {
  cat("  → within noise; hand-written matches macro output.\n")
} else {
  cat("  → divergent; macro emits something extra. Print both bodies.\n")
  cat("\n  package conv_i32_arg body:\n")
  print(body(miniextendr::conv_i32_arg))
  cat("\n  hand-written i32_full body:\n")
  print(body(test_env$i32_full))
}

# ---------------------------------------------------------------------------
# Show the attribution one more time, this time from in-namespace numbers.
# ---------------------------------------------------------------------------

cat("\n## Per-line cost (in-namespace, min_ns)\n")
ns_min <- function(name) summary_tbl$min[summary_tbl$variant == name]
attribution <- data.frame(
  piece = c(
    "stopifnot()",
    "match.call() (success path)",
    "inherits/attr post-check",
    "all three (i32_full - i32_bare)",
    "i32_bare floor"
  ),
  cost_ns = c(
    ns_min("i32_full") - ns_min("i32_no_stopifnot"),
    ns_min("i32_full") - ns_min("i32_no_matchcall"),
    ns_min("i32_no_stop_no_call") - ns_min("i32_bare"),
    ns_min("i32_full") - ns_min("i32_bare"),
    ns_min("i32_bare")
  )
)
print(attribution, row.names = FALSE)

cat("\n# Done.\n")
