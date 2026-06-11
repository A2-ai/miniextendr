#!/usr/bin/env Rscript
# Benchmark for issue #949 — S7 fast-dispatch shortcut.
#
# Every S7 impl method now emits both the normal S7 generic AND a plain
# `<ClassName>_<method_name>(self, ...)` shortcut that calls `.Call` directly,
# bypassing `S7::S7_dispatch()` (the class walk + method-table lookup).
#
# This script compares three ways of invoking the same Rust routine on a hot
# loop, using the exported `S7MatchArgHolder` fixture's `current()` method:
#
#   1. Generic dispatch          — `current(obj)` via S7_dispatch
#   2. Cached method closure      — `S7::method(current, S7MatchArgHolder)(obj)`
#   3. Fast-path shortcut (#949)  — `S7MatchArgHolder_current(obj)`
#
# Run from the repo root so the rv project library is on the libpath:
#   cd <repo> && Rscript analysis/s7-fast-shortcut-bench-949.R

suppressPackageStartupMessages({
  library(miniextendr)
  library(S7)
})

stopifnot(requireNamespace("bench", quietly = TRUE))

# The exported S7 fixture: ImplMode-backed holder with a `current()` getter.
obj <- S7MatchArgHolder("Fast")

# (1) S7 generic dispatch — resolves the method on every call.
gen_call <- function() current(obj)

# (2) Cached method closure — resolve the method object once, call it directly.
#     This is the manual optimization users reach for today; the shortcut should
#     match or beat it without the boilerplate.
cached <- S7::method(current, S7MatchArgHolder)
cached_call <- function() cached(obj)

# (3) Fast-path shortcut emitted by #949 — bypasses S7_dispatch entirely.
shortcut_call <- function() S7MatchArgHolder_current(obj)

# Correctness: all three must agree.
stopifnot(identical(gen_call(), cached_call()))
stopifnot(identical(gen_call(), shortcut_call()))

cat("Result of current(obj):", format(gen_call()), "\n\n")

res <- bench::mark(
  generic  = gen_call(),
  cached   = cached_call(),
  shortcut = shortcut_call(),
  iterations = 10000,
  check = TRUE
)

print(res[, c("expression", "min", "median", "itr/sec", "mem_alloc")])

med <- as.numeric(res$median)
names(med) <- as.character(res$expression)
cat("\nSpeedups (median time):\n")
cat(sprintf("  generic  / shortcut = %.1fx\n", med[["generic"]] / med[["shortcut"]]))
cat(sprintf("  cached   / shortcut = %.2fx\n", med[["cached"]] / med[["shortcut"]]))
