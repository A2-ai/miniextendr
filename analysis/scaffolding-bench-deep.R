# Deep bench: extend M6 with bytecompile sensitivity (M9), run-to-run
# variance (M10), and arg-count / precondition-shape sweep (M12).
#
# Run:
#   Rscript analysis/scaffolding-bench-deep.R \
#     2>&1 | tee analysis/scaffolding-bench-deep-output.txt
#
# M11 (hand-strip an installed wrapper) needs an R CMD INSTALL cycle and
# lives in scaffolding-bench-installed.R. M13 (Rprof on testthat) lives
# in scaffolding-bench-rprof.R.

suppressPackageStartupMessages({
  library(bench)
  library(compiler)
  library(miniextendr)
})

ns <- getNamespace("miniextendr")
C_sym  <- ns$C_conv_sexp_arg
C_i32  <- ns$C_conv_i32_arg

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

bench_quiet <- function(...) {
  bench::mark(..., min_iterations = 20000L, check = FALSE,
              filter_gc = FALSE, time_unit = "ns")
}

to_ns <- function(x) as.numeric(x)

format_results <- function(b, extra = NULL) {
  res <- data.frame(
    variant   = as.character(b$expression),
    min_ns    = round(to_ns(b$min), 1),
    median_ns = round(to_ns(b$median), 1),
    iter_per_sec = round(b$`itr/sec`, 0)
  )
  if (!is.null(extra)) {
    res <- cbind(res, extra)
  }
  res
}

# ---------------------------------------------------------------------------
# Session info
# ---------------------------------------------------------------------------

cat("# Deep scaffolding bench\n\n")
cat("Generated:", format(Sys.time(), "%Y-%m-%d %H:%M:%S %Z"), "\n")
cat("R:", R.version.string, "\n")
cat("miniextendr:", as.character(packageVersion("miniextendr")), "\n")
cat("Platform:", R.version$platform, "\n")
cat("CPU:", system("sysctl -n machdep.cpu.brand_string", intern = TRUE), "\n")
cat("compiler::enableJIT:", compiler::enableJIT(NA_integer_), "\n")
cat("\n")

# ---------------------------------------------------------------------------
# Variants — define both compiled and non-compiled versions.
# ---------------------------------------------------------------------------

wrap_full_src <- function(x) {
  stopifnot(
    "'x' must be numeric, logical, or raw" = is.numeric(x) || is.logical(x) || is.raw(x),
    "'x' must have length 1" = length(x) == 1L
  )
  .val <- .Call(C_sym, .call = match.call(), x)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__")))
    return(.miniextendr_raise_condition(.val, sys.call()))
  .val
}

wrap_no_stopifnot_src <- function(x) {
  .val <- .Call(C_sym, .call = match.call(), x)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__")))
    return(.miniextendr_raise_condition(.val, sys.call()))
  .val
}

wrap_no_matchcall_src <- function(x) {
  stopifnot(
    "'x' must be numeric, logical, or raw" = is.numeric(x) || is.logical(x) || is.raw(x),
    "'x' must have length 1" = length(x) == 1L
  )
  .val <- .Call(C_sym, .call = NULL, x)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__")))
    return(.miniextendr_raise_condition(.val, sys.call()))
  .val
}

wrap_no_stop_no_call_src <- function(x) {
  .val <- .Call(C_sym, .call = NULL, x)
  if (inherits(.val, "rust_condition_value") && isTRUE(attr(.val, "__rust_condition__")))
    return(.miniextendr_raise_condition(.val, sys.call()))
  .val
}

wrap_bare_src <- function(x) .Call(C_sym, .call = NULL, x)

# Byte-compiled variants.
wrap_full_cmp           <- compiler::cmpfun(wrap_full_src,           list(optimize = 3L))
wrap_no_stopifnot_cmp   <- compiler::cmpfun(wrap_no_stopifnot_src,   list(optimize = 3L))
wrap_no_matchcall_cmp   <- compiler::cmpfun(wrap_no_matchcall_src,   list(optimize = 3L))
wrap_no_stop_no_call_cmp <- compiler::cmpfun(wrap_no_stop_no_call_src, list(optimize = 3L))
wrap_bare_cmp           <- compiler::cmpfun(wrap_bare_src,           list(optimize = 3L))

# Sanity check.
x <- 42L
stopifnot(
  wrap_full_src(x) == x, wrap_full_cmp(x) == x,
  wrap_no_stopifnot_src(x) == x, wrap_no_stopifnot_cmp(x) == x,
  wrap_no_matchcall_src(x) == x, wrap_no_matchcall_cmp(x) == x,
  wrap_no_stop_no_call_src(x) == x, wrap_no_stop_no_call_cmp(x) == x,
  wrap_bare_src(x) == x, wrap_bare_cmp(x) == x
)

# ---------------------------------------------------------------------------
# M9: bytecompile sensitivity (compare src vs cmp side-by-side)
# ---------------------------------------------------------------------------

cat("\n# M9: bytecompile sensitivity\n")
cat("Side-by-side: same wrapper, with and without compiler::cmpfun().\n")
cat("Reference: miniextendr::conv_sexp_arg is the actual package wrapper,\n")
cat("auto-compiled at install time via R_COMPILE_PKGS / lazy-load JIT.\n\n")

m9 <- bench_quiet(
  full_src                = wrap_full_src(x),
  full_cmp                = wrap_full_cmp(x),
  no_stopifnot_src        = wrap_no_stopifnot_src(x),
  no_stopifnot_cmp        = wrap_no_stopifnot_cmp(x),
  no_matchcall_src        = wrap_no_matchcall_src(x),
  no_matchcall_cmp        = wrap_no_matchcall_cmp(x),
  no_stop_no_call_src     = wrap_no_stop_no_call_src(x),
  no_stop_no_call_cmp     = wrap_no_stop_no_call_cmp(x),
  bare_src                = wrap_bare_src(x),
  bare_cmp                = wrap_bare_cmp(x),
  package_conv_sexp_arg   = miniextendr::conv_sexp_arg(x)
)
print(format_results(m9), row.names = FALSE)

# Side-by-side deltas: src vs cmp for each variant.
cat("\n## Bytecompile delta (src - cmp, ns)\n")
deltas <- data.frame(
  variant = c("full", "no_stopifnot", "no_matchcall", "no_stop_no_call", "bare"),
  src_ns = c(
    to_ns(m9$min[as.character(m9$expression) == "full_src"]),
    to_ns(m9$min[as.character(m9$expression) == "no_stopifnot_src"]),
    to_ns(m9$min[as.character(m9$expression) == "no_matchcall_src"]),
    to_ns(m9$min[as.character(m9$expression) == "no_stop_no_call_src"]),
    to_ns(m9$min[as.character(m9$expression) == "bare_src"])
  ),
  cmp_ns = c(
    to_ns(m9$min[as.character(m9$expression) == "full_cmp"]),
    to_ns(m9$min[as.character(m9$expression) == "no_stopifnot_cmp"]),
    to_ns(m9$min[as.character(m9$expression) == "no_matchcall_cmp"]),
    to_ns(m9$min[as.character(m9$expression) == "no_stop_no_call_cmp"]),
    to_ns(m9$min[as.character(m9$expression) == "bare_cmp"])
  )
)
deltas$delta_ns <- deltas$src_ns - deltas$cmp_ns
deltas$cmp_pct  <- round(100 * deltas$cmp_ns / deltas$src_ns, 1)
print(deltas, row.names = FALSE)

# ---------------------------------------------------------------------------
# M10: run-to-run variance (5 reps of the key strip variants).
# ---------------------------------------------------------------------------

cat("\n# M10: run-to-run variance (5 reps)\n\n")

reps <- 5L
variants <- list(
  full_cmp        = wrap_full_cmp,
  no_stopifnot_cmp = wrap_no_stopifnot_cmp,
  no_matchcall_cmp = wrap_no_matchcall_cmp,
  no_stop_no_call_cmp = wrap_no_stop_no_call_cmp,
  bare_cmp        = wrap_bare_cmp,
  package_conv_sexp_arg = function(x) miniextendr::conv_sexp_arg(x),
  package_conv_i32_arg  = function(x) miniextendr::conv_i32_arg(x)
)

variance_results <- matrix(NA_real_, nrow = length(variants), ncol = reps,
                           dimnames = list(names(variants), paste0("rep", 1:reps)))
for (i in seq_len(reps)) {
  b <- do.call(bench_quiet, lapply(variants, function(f) bquote(.(f)(.(x)))))
  variance_results[, i] <- to_ns(b$min)
}

variance_summary <- data.frame(
  variant = rownames(variance_results),
  min  = round(apply(variance_results, 1, min), 1),
  mean = round(apply(variance_results, 1, mean), 1),
  median = round(apply(variance_results, 1, median), 1),
  max  = round(apply(variance_results, 1, max), 1),
  sd   = round(apply(variance_results, 1, sd), 1)
)
variance_summary$cv_pct <- round(100 * variance_summary$sd / variance_summary$mean, 1)
print(variance_summary, row.names = FALSE)

# ---------------------------------------------------------------------------
# M12: multi-arg + multi-precondition-shape sweep.
#
# Build synthetic wrappers with varying arg counts (0/1/2/3/5) and
# precondition shape (none, numeric_scalar, string_scalar, nullable_numeric,
# numeric_vector).
# ---------------------------------------------------------------------------

cat("\n# M12: multi-arg + precondition-shape sweep\n\n")

# All wrappers below ignore their inputs and call C_sym(NULL, 42L).
# Cost differences come from the *prologue* (preconditions + match.call),
# not from C-side work.

make_wrapper <- function(n_args, precondition_shape, with_matchcall = FALSE) {
  arg_names <- if (n_args == 0L) character(0L) else paste0("a", seq_len(n_args))
  formals_str <- if (n_args == 0L) "" else paste(arg_names, collapse = ", ")

  preconds <- character(0L)
  if (n_args > 0L) {
    preconds <- switch(precondition_shape,
      none = character(0L),
      numeric_scalar = unlist(lapply(arg_names, function(a) c(
        sprintf("\"'%s' must be numeric, logical, or raw\" = is.numeric(%s) || is.logical(%s) || is.raw(%s)", a, a, a, a),
        sprintf("\"'%s' must have length 1\" = length(%s) == 1L", a, a)
      ))),
      string_scalar = unlist(lapply(arg_names, function(a) c(
        sprintf("\"'%s' must be character\" = is.character(%s)", a, a),
        sprintf("\"'%s' must have length 1\" = length(%s) == 1L", a, a)
      ))),
      nullable_numeric = unlist(lapply(arg_names, function(a) c(
        sprintf("\"'%s' must be NULL or numeric\" = is.null(%s) || is.numeric(%s)", a, a, a),
        sprintf("\"'%s' must be NULL or have length 1\" = is.null(%s) || length(%s) == 1L", a, a, a)
      ))),
      numeric_vector = unlist(lapply(arg_names, function(a) c(
        sprintf("\"'%s' must be numeric\" = is.numeric(%s)", a, a)
      )))
    )
  }
  precond_block <- if (length(preconds) == 0L) "" else paste0(
    "stopifnot(\n    ", paste(preconds, collapse = ",\n    "), "\n  )\n  "
  )
  call_arg <- if (with_matchcall) "match.call()" else "NULL"
  body <- sprintf(
    "function(%s) {\n  %s.Call(C_sym, .call = %s, 42L)\n}",
    formals_str, precond_block, call_arg
  )
  fn <- eval(parse(text = body), envir = environment())
  compiler::cmpfun(fn, list(optimize = 3L))
}

# Generate the matrix.
arg_counts <- c(0L, 1L, 2L, 3L, 5L)
shapes <- c("none", "numeric_scalar", "string_scalar", "nullable_numeric",
            "numeric_vector")

call_args <- list(
  scalar_int    = lapply(1:5, function(i) 1L),
  scalar_str    = lapply(1:5, function(i) "x"),
  scalar_null   = lapply(1:5, function(i) NULL),
  vec_int       = lapply(1:5, function(i) c(1L, 2L, 3L))
)
shape_to_args <- list(
  none             = "scalar_int",
  numeric_scalar   = "scalar_int",
  string_scalar    = "scalar_str",
  nullable_numeric = "scalar_int",
  numeric_vector   = "vec_int"
)

m12_rows <- list()
for (n in arg_counts) {
  for (sh in shapes) {
    if (n == 0L && sh != "none") next
    for (mc in c(FALSE, TRUE)) {
      fn <- make_wrapper(n, sh, with_matchcall = mc)
      args <- call_args[[shape_to_args[[sh]]]][seq_len(n)]
      # bench expects an expression — build it dynamically
      label <- sprintf("args=%d shape=%s mc=%s", n, sh,
                       if (mc) "match.call" else "NULL")
      # Time the call
      e <- bench_quiet(do.call(fn, args))
      m12_rows[[label]] <- data.frame(
        n_args = n, shape = sh,
        match_call = mc,
        min_ns = round(to_ns(e$min), 1),
        median_ns = round(to_ns(e$median), 1)
      )
    }
  }
}
m12 <- do.call(rbind, m12_rows)
rownames(m12) <- NULL
print(m12)

cat("\n# Done.\n")
