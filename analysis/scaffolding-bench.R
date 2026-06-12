# Scaffolding-cost benchmark for `#[miniextendr]`-generated R wrappers.
#
# Goal: pinpoint how much time the generated wrapper layers add on top of a
# baseline R/C call, broken down per layer (R wrapper, .Call boundary,
# with_r_unwind_protect, TryFromSexp decode, IntoR encode, error path,
# class-system dispatch).
#
# Layers per call (from `cargo expand` of conversions::C_conv_*):
#   R-side
#     1. function prologue: stopifnot(...) precondition checks (scalars only)
#     2. .Call(C_..., .call = match.call(), <args>) — match.call() allocates
#     3. post-call: inherits(.val, "rust_condition_value") + attr fast check
#   C-side
#     4. extern "C-unwind" entry
#     5. with_r_unwind_protect:
#          - Box<CallData{f, result, panic_payload}> heap alloc
#          - get_continuation_token()
#          - outer catch_unwind (Rust landing pad)
#          - R_UnwindProtect_C_unwind trampoline + cleanup_handler install
#          - inner catch_unwind around the body closure
#          - Box::from_raw reclaim
#          - drain_log_queue_if_available (no-op without `log` feature)
#     6. per arg: TryFromSexp::try_from_sexp(x)?  on error → make_rust_condition_value
#     7. user body
#     8. return: IntoR::into_sexp(__result)   (skipped for fn -> SEXP)
#
# Run:
#   Rscript analysis/scaffolding-bench.R 2>&1 | tee analysis/scaffolding-bench-output.txt

suppressPackageStartupMessages({
  library(bench)
  library(miniextendr)
})

ns <- getNamespace("miniextendr")

# -----------------------------------------------------------------------------
# Helpers
# -----------------------------------------------------------------------------

# Tight wrapper around bench::mark() — drop check, focus on min/median/iter count.
mk <- function(...) {
  bench::mark(..., min_iterations = 5000L, check = FALSE, filter_gc = FALSE,
              time_unit = "ns")
}

press <- function(grid, expr_fn, label) {
  bench::press(
    {{ grid }},
    .grid = grid,
    expr_fn = expr_fn,
    label = label
  )
}

print_section <- function(title, result) {
  cat("\n##", title, "\n")
  res <- as.data.frame(result)
  # Convert bench durations to plain numeric ns for stable formatting.
  to_ns <- function(x) if (inherits(x, "bench_time")) as.numeric(x) * 1e9 else as.numeric(x)
  keep <- intersect(c("expression", "min", "median", "itr/sec", "n_itr", "n_gc",
                      "size"), names(res))
  res <- res[, keep, drop = FALSE]
  if ("min"    %in% names(res)) res$min_ns    <- round(to_ns(res$min), 1)
  if ("median" %in% names(res)) res$median_ns <- round(to_ns(res$median), 1)
  res$min <- NULL; res$median <- NULL
  res$expression <- vapply(res$expression, function(e) {
    if (is.call(e) || is.symbol(e)) paste(deparse(e), collapse = " ") else as.character(e)
  }, character(1))
  # Reorder
  ord <- c("expression", "size", "min_ns", "median_ns", "itr/sec", "n_itr", "n_gc")
  res <- res[, intersect(ord, names(res)), drop = FALSE]
  print(res, row.names = FALSE)
}

# -----------------------------------------------------------------------------
# Session info — always at top of output for reproducibility.
# -----------------------------------------------------------------------------

cat("# miniextendr scaffolding benchmark\n\n")
cat("Generated:", format(Sys.time(), "%Y-%m-%d %H:%M:%S %Z"), "\n")
cat("R:", R.version.string, "\n")
cat("miniextendr:", as.character(packageVersion("miniextendr")), "\n")
cat("Platform:", R.version$platform, "\n")
cat("CPU:", system("sysctl -n machdep.cpu.brand_string", intern = TRUE), "\n")
cat("\n")

# -----------------------------------------------------------------------------
# 1. Baseline: R-only wrappers vs `.Call` direct vs full miniextendr wrapper.
#
#    Measures the cost of the scaffolding stack on the most trivial body
#    possible — a SEXP passthrough.
# -----------------------------------------------------------------------------

# Baseline: pure-R identity. Tells us R call-and-return cost on this machine.
r_identity      <- function(x) x

# Bypass the generated R wrapper but keep `.Call` shape (no match.call, no
# post-call inherits/attr check). Pure C-side scaffolding cost.
call_direct_sexp <- function(x)
  .Call(ns$C_conv_sexp_arg, NULL, x)

# Bypass the generated R wrapper and `.Call` adornments entirely.
# Equivalent to what an extendr-style wrapper would do.
call_no_match_call <- function(x)
  .Call(ns$C_conv_sexp_arg, NULL, x)

x_scalar <- 42L
x_vec64  <- as.numeric(1:64)

cat("\n# 1. Baseline (SEXP passthrough) ------------------------------------\n")
b1 <- mk(
  r_identity_scalar       = r_identity(x_scalar),
  base_identity_scalar    = identity(x_scalar),
  call_direct_scalar      = .Call(ns$C_conv_sexp_arg, NULL, x_scalar),
  wrapper_full_scalar     = miniextendr::conv_sexp_arg(x_scalar),

  r_identity_vec64        = r_identity(x_vec64),
  call_direct_vec64       = .Call(ns$C_conv_sexp_arg, NULL, x_vec64),
  wrapper_full_vec64      = miniextendr::conv_sexp_arg(x_vec64)
)
print_section("Baseline — SEXP passthrough (scaffolding only)", b1)

# -----------------------------------------------------------------------------
# 2. TryFromSexp argument-decode cost.
#
#    Same wrapper shape, different TryFromSexp impls. The delta vs
#    conv_sexp_arg isolates per-arg decode cost.
# -----------------------------------------------------------------------------

cat("\n# 2. TryFromSexp arg decode -----------------------------------------\n")
b2 <- mk(
  arg_sexp_scalar         = miniextendr::conv_sexp_arg(42L),
  arg_i32_scalar          = miniextendr::conv_i32_arg(42L),
  arg_f64_scalar          = miniextendr::conv_f64_arg(42.0),
  arg_rbool_scalar        = miniextendr::conv_rbool_arg(TRUE),
  arg_u8_scalar           = miniextendr::conv_u8_arg(as.raw(7L)),
  arg_string_scalar       = miniextendr::conv_string_arg("hello")
)
print_section("TryFromSexp scalar arg decode", b2)

# -----------------------------------------------------------------------------
# 3. Vec arg decode — size sweep via bench::press()
# -----------------------------------------------------------------------------

cat("\n# 3. Vec<T> arg decode (size sweep) ----------------------------------\n")
b3 <- bench::press(
  size = c(1L, 16L, 256L, 4096L, 65536L),
  {
    xi <- seq_len(size)
    xd <- as.numeric(seq_len(size))
    bench::mark(
      vec_i32_arg = miniextendr::conv_vec_i32_len(xi),
      vec_f64_arg = miniextendr::conv_vec_f64_len(xd),
      min_iterations = 500L, check = FALSE, filter_gc = FALSE, time_unit = "ns"
    )
  }
)
print_section("Vec<T> arg decode by size", b3)

# -----------------------------------------------------------------------------
# 4. IntoR return-encode cost (no args, vary return type).
# -----------------------------------------------------------------------------

cat("\n# 4. IntoR return encode ---------------------------------------------\n")
b4 <- mk(
  ret_i32_scalar    = miniextendr::conv_i32_ret(),
  ret_f64_scalar    = miniextendr::conv_f64_ret(),
  ret_rbool_scalar  = miniextendr::conv_rbool_ret(),
  ret_u8_scalar     = miniextendr::conv_u8_ret(),
  ret_string_scalar = miniextendr::conv_string_ret(),
  ret_sexp          = miniextendr::conv_sexp_ret()
)
print_section("IntoR scalar return encode", b4)

# -----------------------------------------------------------------------------
# 5. Vec<T> return encode — size sweep.
# -----------------------------------------------------------------------------

cat("\n# 5. Vec<T> return encode (size sweep) -------------------------------\n")
# Each conv_vec_<T>_ret currently builds a fixed-size Vec internally — useful
# as a fixed-cost baseline; we also press over deliberately-sized fns below.
b5 <- mk(
  vec_i32_ret    = miniextendr::conv_vec_i32_ret(),
  vec_f64_ret    = miniextendr::conv_vec_f64_ret(),
  vec_string_ret = miniextendr::conv_vec_string_ret()
)
print_section("Vec<T> return encode (built-in sizes)", b5)

# -----------------------------------------------------------------------------
# 6. Error path cost.
#
#    A panic / error!() routes through:
#      catch_unwind → RCondition / panic_payload_to_string
#      → make_rust_condition_value (4-element list + class + attrs)
#      → R wrapper inherits()+attr() check → .miniextendr_raise_condition
#      → stop(structure(...))
#    Compare vs direct R-side stop().
# -----------------------------------------------------------------------------

cat("\n# 6. Error path -------------------------------------------------------\n")
err_r       <- function() tryCatch(stop("oops"), error = function(e) NULL)
err_macro   <- function() tryCatch(miniextendr::demo_error("oops"),
                                    error = function(e) NULL)
err_warn    <- function() tryCatch(miniextendr::demo_warning("hmm"),
                                    warning = function(w) NULL)

b6 <- mk(
  r_stop_caught         = err_r(),
  rust_error_macro      = err_macro(),
  rust_warning_macro    = err_warn()
)
print_section("Error transport — Rust → tagged SEXP → R condition", b6)

# -----------------------------------------------------------------------------
# 7. Class-system method dispatch overhead.
#
#    `demo_error`/etc are bare fns; class methods add a `$` dispatch + closure
#    chain. Compare directly via `bench::press()` over class systems.
#
#    SimpleCounter is exported by rpkg in multiple class flavors (S3, S4, R6,
#    S7, Env). Use whichever match — fall through gracefully if absent.
# -----------------------------------------------------------------------------

cat("\n# 7. Class-system dispatch -------------------------------------------\n")

# Build one counter object per class system (constructor APIs are heterogenous
# because each class system has its own conventions).
env_counter <- ns$SimpleCounter$new_counter(0L)   # Env: closure-based methods
r6_counter  <- ns$R6Counter$new(0L)               # R6: $method() dispatch
s4_counter  <- ns$S4Counter(0L)                   # S4: setMethod()
s7_counter  <- ns$S7Counter(0L)                   # S7: S7_dispatch()

# Sanity-check (will error here if the API drifted).
stopifnot(
  env_counter$get_value()   == 0L,
  r6_counter$value()        == 0L,
  ns$s4_value(s4_counter)   == 0L,
  ns$s7_value(s7_counter)   == 0L
)

b7 <- mk(
  Env_get_value  = env_counter$get_value(),
  R6_value       = r6_counter$value(),
  S4_value       = ns$s4_value(s4_counter),
  S7_value       = ns$s7_value(s7_counter),
  fn_baseline    = miniextendr::conv_i32_ret()  # bare fn, same body
)
print_section("Class-system method dispatch (`&self -> i32`)", b7)

# -----------------------------------------------------------------------------
# 8. Multi-arg fan-in (does arg count linearly add to scaffolding overhead?)
#
#    Use sum_args-style fns if available; otherwise approximate via fns that
#    take multiple scalars. (We always have at least conv_*_arg fns; chain
#    them to simulate.)
# -----------------------------------------------------------------------------

cat("\n# 8. Per-arg overhead scan -------------------------------------------\n")

# Tight 2-arg call if exposed, else two separate single-arg calls.
b8 <- mk(
  one_arg_i32  = miniextendr::conv_i32_arg(42L),
  two_calls    = { miniextendr::conv_i32_arg(42L); miniextendr::conv_i32_arg(43L) },
  three_calls  = { miniextendr::conv_i32_arg(1L); miniextendr::conv_i32_arg(2L);
                   miniextendr::conv_i32_arg(3L) }
)
print_section("Per-arg overhead approximation", b8)

# -----------------------------------------------------------------------------
# Done.
# -----------------------------------------------------------------------------

cat("\n# Done.\n")
