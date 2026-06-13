# M6 + M1: Hand-strip the R wrapper progressively to find the theoretical
# floor of scaffolding cost, and attribute the 1.4 μs R-wrapper layer to
# specific lines.
#
# We benchmark 8 variants of a wrapper around .Call(C_conv_sexp_arg, ...).
# Each removes one piece. The deltas attribute cost to that piece.
#
# Run:
#   Rscript analysis/scaffolding-strip-bench.R 2>&1 \
#     | tee analysis/scaffolding-strip-output.txt

suppressPackageStartupMessages({
  library(bench)
  library(miniextendr)
})

ns <- getNamespace("miniextendr")
C_sym  <- ns$C_conv_sexp_arg     # SEXP -> SEXP identity (TryFromSexp is no-op)
C_i32  <- ns$C_conv_i32_arg      # SEXP -> SEXP via TryFromSexp<i32>/IntoR

# ---------------------------------------------------------------------------
# Variants — sexp path (no TryFromSexp work). Lets us isolate R-side cost.
# Each variant strips ONE more piece than the previous.
# ---------------------------------------------------------------------------

# A. Full wrapper — what the macro emits today.
wrap_full <- function(x) {
  stopifnot(
    "'x' must be numeric, logical, or raw" = is.numeric(x) || is.logical(x) || is.raw(x),
    "'x' must have length 1" = length(x) == 1L
  )
  .val <- .Call(C_sym, .call = match.call(), x)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__")))
    return(.miniextendr_raise_condition(.val, sys.call()))
  .val
}

# B. Drop stopifnot only (keep match.call + post-check).
wrap_no_stopifnot <- function(x) {
  .val <- .Call(C_sym, .call = match.call(), x)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__")))
    return(.miniextendr_raise_condition(.val, sys.call()))
  .val
}

# C. Drop match.call() only (keep stopifnot + post-check). .call = NULL.
wrap_no_matchcall <- function(x) {
  stopifnot(
    "'x' must be numeric, logical, or raw" = is.numeric(x) || is.logical(x) || is.raw(x),
    "'x' must have length 1" = length(x) == 1L
  )
  .val <- .Call(C_sym, .call = NULL, x)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__")))
    return(.miniextendr_raise_condition(.val, sys.call()))
  .val
}

# D. Drop post-check only (keep stopifnot + match.call).
wrap_no_postcheck <- function(x) {
  stopifnot(
    "'x' must be numeric, logical, or raw" = is.numeric(x) || is.logical(x) || is.raw(x),
    "'x' must have length 1" = length(x) == 1L
  )
  .Call(C_sym, .call = match.call(), x)
}

# E. Drop stopifnot + match.call (keep post-check).
wrap_no_stop_no_call <- function(x) {
  .val <- .Call(C_sym, .call = NULL, x)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__")))
    return(.miniextendr_raise_condition(.val, sys.call()))
  .val
}

# F. Drop stopifnot + post-check (keep match.call). Tests how cheap a
#    minimal wrapper can be while preserving call attribution.
wrap_only_matchcall <- function(x) {
  .Call(C_sym, .call = match.call(), x)
}

# G. Drop everything except the .Call. The R floor.
wrap_bare <- function(x) {
  .Call(C_sym, .call = NULL, x)
}

# H. Below the R floor: no closure at all, .Call inline.
#    (Identical to G in practice — included for sanity.)
call_inline <- function(x) .Call(C_sym, NULL, x)

# ---------------------------------------------------------------------------
# Variants — i32 path (with TryFromSexp + IntoR work). Reveals whether the
# C-side cost is invariant of the R-side variants.
# ---------------------------------------------------------------------------

i32_full <- function(x) {
  stopifnot(
    "'x' must be numeric, logical, or raw" = is.numeric(x) || is.logical(x) || is.raw(x),
    "'x' must have length 1" = length(x) == 1L
  )
  .val <- .Call(C_i32, .call = match.call(), x)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__")))
    return(.miniextendr_raise_condition(.val, sys.call()))
  .val
}

i32_bare <- function(x) .Call(C_i32, .call = NULL, x)

# ---------------------------------------------------------------------------
# Sanity check — all variants return 42L for input 42L.
# ---------------------------------------------------------------------------
stopifnot(
  wrap_full(42L)        == 42L,
  wrap_no_stopifnot(42L)  == 42L,
  wrap_no_matchcall(42L)  == 42L,
  wrap_no_postcheck(42L)  == 42L,
  wrap_no_stop_no_call(42L) == 42L,
  wrap_only_matchcall(42L)  == 42L,
  wrap_bare(42L)         == 42L,
  call_inline(42L)       == 42L,
  i32_full(42L)          == 42L,
  i32_bare(42L)          == 42L
)

# ---------------------------------------------------------------------------
# Bench all variants together — bench::mark's `relative` will give us
# ratios automatically.
# ---------------------------------------------------------------------------

cat("# M6: hand-stripped R wrapper variants\n")
cat("R:", R.version.string, "/ miniextendr:",
    as.character(packageVersion("miniextendr")), "\n")
cat("Tool: bench::mark, min_iterations = 50000, time_unit = ns\n\n")

x <- 42L
b <- bench::mark(
  A_full              = wrap_full(x),
  B_no_stopifnot      = wrap_no_stopifnot(x),
  C_no_matchcall      = wrap_no_matchcall(x),
  D_no_postcheck      = wrap_no_postcheck(x),
  E_no_stop_no_call   = wrap_no_stop_no_call(x),
  F_only_matchcall    = wrap_only_matchcall(x),
  G_bare              = wrap_bare(x),
  H_inline_call       = call_inline(x),
  i32_full            = i32_full(x),
  i32_bare            = i32_bare(x),
  min_iterations = 50000L,
  check = FALSE,
  filter_gc = FALSE,
  time_unit = "ns"
)

# With time_unit = "ns" bench returns ns-scaled numerics already; no extra
# conversion needed.
to_ns <- function(x) as.numeric(x)
res <- data.frame(
  variant   = as.character(b$expression),
  min_ns    = round(to_ns(b$min), 1),
  median_ns = round(to_ns(b$median), 1),
  iter_per_sec = round(b$`itr/sec`, 0)
)
print(res, row.names = FALSE)

# ---------------------------------------------------------------------------
# M1: Per-line cost attribution.
#
# Reasoning: each variant strips ONE thing relative to A_full.
#   A_full          = baseline                          (full)
#   B_no_stopifnot  = full − stopifnot                  → cost(stopifnot)   = A − B
#   C_no_matchcall  = full − match.call (success path)  → cost(match.call)  = A − C
#   D_no_postcheck  = full − post-check                 → cost(post-check)  = A − D
#   G_bare          = none                              → floor (just .Call)
#   sum_pieces      = A − B + A − C + A − D
#   reconstructed   = A − sum_pieces
#   reconstructed should ≈ G_bare if pieces are roughly additive.
# ---------------------------------------------------------------------------

expr_names <- as.character(b$expression)
ns_for <- function(name) round(to_ns(b$min[expr_names == name]), 1)
a  <- ns_for("A_full")
b1 <- ns_for("B_no_stopifnot")
c1 <- ns_for("C_no_matchcall")
d1 <- ns_for("D_no_postcheck")
e1 <- ns_for("E_no_stop_no_call")
f1 <- ns_for("F_only_matchcall")
g1 <- ns_for("G_bare")

cat("\n# M1: per-line attribution (min_ns)\n")
attribution <- data.frame(
  piece = c(
    "stopifnot()",
    "match.call() (success path)",
    "inherits()/attr() post-check",
    "stopifnot + match.call (sum)",
    "stopifnot + post-check (sum)",
    "match.call + post-check (sum)",
    "all three (A_full - G_bare)",
    "G_bare floor (.Call only, no wrapper)"
  ),
  delta_vs_full = c(
    a - b1,                # stopifnot
    a - c1,                # match.call
    a - d1,                # post-check
    a - e1,                # stopifnot + match.call
    a - f1,                # stopifnot + post-check (F is only match.call left)
    a - g1 - (a - b1) - (a - d1),  # not directly measurable; reconstructed
    a - g1,                # all three
    g1                     # absolute floor
  )
)
print(attribution, row.names = FALSE)

# ---------------------------------------------------------------------------
# Quick sanity probe — verify the error path still fires with .call = NULL.
# ---------------------------------------------------------------------------

cat("\n# Sanity: error condition with .call = NULL\n")
raise <- ns$.miniextendr_raise_condition

demo_with_match_call <- function(msg) {
  .val <- .Call(ns$C_demo_error, .call = match.call(), msg)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__")))
    return(raise(.val, sys.call()))
  .val
}

demo_with_null_call <- function(msg) {
  .val <- .Call(ns$C_demo_error, .call = NULL, msg)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__")))
    return(raise(.val, sys.call()))
  .val
}

cap_err <- function(expr) tryCatch(expr, error = function(e) e)

e1 <- cap_err(demo_with_match_call("oops via match.call"))
e2 <- cap_err(demo_with_null_call("oops via sys.call fallback"))

cat("---- match.call() path ----\n")
cat("  message:", conditionMessage(e1), "\n")
cat("  call   :", deparse(conditionCall(e1)), "\n")
cat("  classes:", paste(head(class(e1), 6), collapse = ", "), "\n")
cat("---- .call = NULL path (sys.call fallback) ----\n")
cat("  message:", conditionMessage(e2), "\n")
cat("  call   :", deparse(conditionCall(e2)), "\n")
cat("  classes:", paste(head(class(e2), 6), collapse = ", "), "\n")

cat("\n# Done.\n")
