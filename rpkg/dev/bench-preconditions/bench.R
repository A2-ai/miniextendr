# Success-path cost of the R-side preconditions vs the Rust conversion.
#
# Usage, from the repository root, after `just configure && just rcmdinstall`:
#   Rscript rpkg/dev/bench-preconditions/bench.R <out.csv> [groups...]
# The default groups are the main run: scalar vector nona altrep whole inherits
# df choices. The chunked-scan run is a separate call with the group `chunked`.
# PCB_REPS (default 3) and PCB_MIN_TIME (seconds, default 0.4) set the reps and
# bench::mark's min_time. run.R drives every script in this directory.
#
# The fixtures are the `pcb_*` / `C_pcb_*` functions of
# rpkg/src/rust/precondition_bench_fixtures.rs. Every function is bound to a
# local name first (a `:::` lookup inside the benchmarked expression would cost
# about as much as a guard). Each group runs REPS times; within a rep, the
# variants of one case share one bench::mark() call so they see the same
# machine state. filter_gc = FALSE: the GC an allocating guard triggers is part
# of its cost.

args <- commandArgs(trailingOnly = TRUE)
if (length(args) < 1L) {
  stop("usage: Rscript rpkg/dev/bench-preconditions/bench.R <out.csv> [groups...]", call. = FALSE)
}
out_csv <- args[[1]]
groups <- if (length(args) > 1) args[-1] else c("scalar", "vector", "nona", "altrep", "whole", "inherits", "df", "choices")
REPS <- as.integer(Sys.getenv("PCB_REPS", "3"))
MIN_TIME <- as.numeric(Sys.getenv("PCB_MIN_TIME", "0.4"))

suppressPackageStartupMessages(library(bench))
ns <- asNamespace("miniextendr")
for (nm in grep("^(pcb_|unsafe_C_pcb_|fast_)", ls(ns, all.names = TRUE), value = TRUE)) {
  assign(nm, get(nm, ns))
}

results <- list()
record <- function(group, case, n, input, bm, rep) {
  df <- data.frame(
    group = group, case = case, n = n, input = input, rep = rep,
    variant = as.character(bm$expression),
    median_ns = as.numeric(bm$median) * 1e9,
    # 5%-trimmed mean of the per-iteration times: the macOS timer ticks at
    # ~41.7 ns, so a median is quantised; the mean dithers across ticks.
    tmean_ns = vapply(bm$time, function(t) mean(as.numeric(t), trim = 0.05), numeric(1)) * 1e9,
    mem_bytes = as.numeric(bm$mem_alloc),
    n_itr = bm$n_itr, n_gc = bm$n_gc,
    stringsAsFactors = FALSE
  )
  results[[length(results) + 1L]] <<- df
  write.csv(do.call(rbind, results), out_csv, row.names = FALSE)
  df
}
# Each expression is evaluated once first (warm-up), so one-time costs (first
# promise forcing, lazy loads) do not land in the memory measurement, which
# bench::mark takes on the first evaluation.
mk <- function(...) {
  ex <- as.list(substitute(list(...)))[-1L]
  env <- parent.frame()
  for (e in ex) eval(e, env)
  bench::mark(exprs = ex, env = env, check = FALSE, min_time = MIN_TIME,
              max_iterations = 1e6, min_iterations = 5, filter_gc = FALSE)
}
log <- function(...) cat(format(Sys.time(), "%H:%M:%S"), ..., "\n")

NS_VEC <- c(1, 1e3, 1e5, 1e7)
NS_ALT <- c(1e3, 1e5, 1e7)

set.seed(1)
for (rep in seq_len(REPS)) {
  log("rep", rep)

  if ("scalar" %in% groups) {
    log("scalar")
    record("scalar", "i32", 1, "1L", mk(guarded = pcb_i32(1L), unguarded = pcb_i32_u(1L)), rep)
    record("scalar", "f64", 1, "1.5", mk(guarded = pcb_f64(1.5), unguarded = pcb_f64_u(1.5)), rep)
    record("scalar", "bool", 1, "TRUE", mk(guarded = pcb_bool(TRUE), unguarded = pcb_bool_u(TRUE)), rep)
    record("scalar", "&str", 1, "\"abc\"", mk(guarded = pcb_str("abc"), unguarded = pcb_str_u("abc")), rep)
    record("scalar", "String", 1, "\"abc\"", mk(guarded = pcb_string("abc"), unguarded = pcb_string_u("abc")), rep)
    record("scalar", "Option<i32>", 1, "1L", mk(guarded = pcb_opt_i32(1L), unguarded = pcb_opt_i32_u(1L)), rep)
    record("scalar", "Option<i32>", 1, "NULL", mk(guarded = pcb_opt_i32(NULL), unguarded = pcb_opt_i32_u(NULL)), rep)
    record("scalar", "(i32, f64)", 1, "1L, 2.5", mk(guarded = pcb_two(1L, 2.5), unguarded = pcb_two_u(1L, 2.5)), rep)
    record("scalar", "fast_i32 (existing)", 1, "1L",
           mk(guarded = fast_i32_default(1L), unguarded = fast_i32_no_preconditions(1L)), rep)
    record("scalar", "fast_sum3 (existing)", 1, "1L,2L,3L",
           mk(guarded = fast_sum3_default(1L, 2L, 3L), unguarded = fast_sum3_no_preconditions(1L, 2L, 3L)), rep)
    record("scalar", "floors", 1, "1.5",
           mk(raw_call = unsafe_C_pcb_raw_floor(1.5),
              raw_type_len = unsafe_C_pcb_type_len(1.5),
              r_guard_only = isTRUE(is.double(1.5)) && isTRUE(length(1.5) == 1L)), rep)
  }

  if ("vector" %in% groups) {
    for (n in NS_VEC) {
      log("vector n =", n)
      xd <- runif(n)
      xi <- as.integer(runif(n) * 1000)
      record("vector", "&[f64]", n, "runif(n)", mk(guarded = pcb_slice_f64(xd), unguarded = pcb_slice_f64_u(xd)), rep)
      record("vector", "Vec<f64>", n, "runif(n)", mk(guarded = pcb_vec_f64(xd), unguarded = pcb_vec_f64_u(xd)), rep)
      record("vector", "&[i32]", n, "int(n)", mk(guarded = pcb_slice_i32(xi), unguarded = pcb_slice_i32_u(xi)), rep)
      record("vector", "Vec<i32>", n, "int(n)", mk(guarded = pcb_vec_i32(xi), unguarded = pcb_vec_i32_u(xi)), rep)
    }
  }

  if ("nona" %in% groups) {
    for (n in NS_VEC) {
      log("nona n =", n)
      xd <- runif(n)
      xi <- as.integer(runif(n) * 1000)
      xc <- structure(runif(n), class = "mx_num")
      record("nona", "no_na &[f64]", n, "runif(n)",
             mk(guard_type_na = pcb_nona_slice_f64(xd), guard_na = pcb_nona_slice_f64_u(xd),
                no_check = pcb_slice_f64_u(xd),
                rust_any_na = pcb_either_f64_nona(xd), rust_either_base = pcb_either_f64(xd)), rep)
      record("nona", "no_na &[i32]", n, "int(n)",
             mk(guard_type_na = pcb_nona_slice_i32(xi), guard_na = pcb_nona_slice_i32_u(xi),
                no_check = pcb_slice_i32_u(xi),
                rust_any_na = pcb_either_i32_nona(xi), rust_either_base = pcb_either_i32(xi)), rep)
      record("nona", "scan dbl", n, "runif(n)",
             mk(sexp_floor = pcb_sexp_floor(xd), r_anyNA_guard = pcb_sexp_rguard_nona(xd),
                raw_floor = unsafe_C_pcb_raw_floor(xd), raw_slice = unsafe_C_pcb_na_slice(xd),
                raw_elt = unsafe_C_pcb_na_framework(xd), raw_capi = unsafe_C_pcb_na_capi(xd)), rep)
      record("nona", "scan int", n, "int(n)",
             mk(sexp_floor = pcb_sexp_floor(xi), r_anyNA_guard = pcb_sexp_rguard_nona(xi),
                raw_floor = unsafe_C_pcb_raw_floor(xi), raw_slice = unsafe_C_pcb_na_slice(xi),
                raw_elt = unsafe_C_pcb_na_framework(xi), raw_capi = unsafe_C_pcb_na_capi(xi)), rep)
      record("nona", "scan classed dbl", n, "structure(runif(n), class)",
             mk(sexp_floor = pcb_sexp_floor(xc), r_anyNA_guard = pcb_sexp_rguard_nona(xc),
                raw_floor = unsafe_C_pcb_raw_floor(xc), raw_capi = unsafe_C_pcb_na_capi(xc)), rep)
    }
  }

  if ("altrep" %in% groups) {
    for (n in NS_ALT) {
      log("altrep n =", n)
      # Each iteration builds a fresh compact sequence: expansion is sticky.
      record("altrep", "scan compact int", n, "1:n",
             mk(ctor_only = length(1:n), altrep_floor = pcb_altrep_floor(1:n),
                r_anyNA_guard = pcb_altrep_rguard_nona(1:n),
                raw_floor = unsafe_C_pcb_raw_floor(1:n), raw_capi = unsafe_C_pcb_na_capi(1:n),
                raw_elt = unsafe_C_pcb_na_framework(1:n), raw_slice = unsafe_C_pcb_na_slice(1:n)), rep)
      record("altrep", "scan compact dbl", n, "as.double(1:n)",
             mk(ctor_only = length(as.double(1:n)), altrep_floor = pcb_altrep_floor(as.double(1:n)),
                r_anyNA_guard = pcb_altrep_rguard_nona(as.double(1:n)),
                raw_floor = unsafe_C_pcb_raw_floor(as.double(1:n)),
                raw_capi = unsafe_C_pcb_na_capi(as.double(1:n)),
                raw_elt = unsafe_C_pcb_na_framework(as.double(1:n)),
                raw_slice = unsafe_C_pcb_na_slice(as.double(1:n))), rep)
      record("altrep", "no_na &[i32] compact", n, "1:n",
             mk(guard_type_na = pcb_nona_slice_i32(1:n), guard_na = pcb_nona_slice_i32_u(1:n),
                no_check = pcb_slice_i32_u(1:n),
                rust_any_na = pcb_either_i32_nona(1:n), rust_either_base = pcb_either_i32(1:n)), rep)
      record("altrep", "&[i32] compact", n, "1:n",
             mk(guarded = pcb_slice_i32(1:n), unguarded = pcb_slice_i32_u(1:n)), rep)
    }
  }

  if ("whole" %in% groups) {
    for (n in NS_VEC) {
      log("whole n =", n)
      xw <- as.double(as.integer(runif(n) * 1000))
      record("whole", "coerce Vec<i32> from doubles", n, "whole doubles",
             mk(guarded = pcb_coerce_vec_i32(xw), unguarded = pcb_coerce_vec_i32_u(xw),
                r_guard_expr = isTRUE(is.integer(xw) || is.logical(xw) || is.raw(xw) || (is.numeric(xw) && all(is.na(xw) | xw == trunc(xw)))),
                raw_floor = unsafe_C_pcb_raw_floor(xw), raw_slice = unsafe_C_pcb_whole_slice(xw),
                raw_capi = unsafe_C_pcb_whole_capi(xw)), rep)
      if (n > 1) {
        record("whole", "coerce Vec<i32> from compact dbl", n, "as.double(1:n)",
               mk(guarded = pcb_coerce_vec_i32(as.double(1:n)), unguarded = pcb_coerce_vec_i32_u(as.double(1:n)),
                  raw_floor = unsafe_C_pcb_raw_floor(as.double(1:n)),
                  raw_slice = unsafe_C_pcb_whole_slice(as.double(1:n)),
                  raw_capi = unsafe_C_pcb_whole_capi(as.double(1:n))), rep)
      }
    }
  }

  if ("chunked" %in% groups) {
    for (n in NS_VEC) {
      log("chunked n =", n)
      xd <- runif(n)
      xi <- as.integer(runif(n) * 1000)
      xw <- as.double(as.integer(runif(n) * 1000))
      record("chunked", "NA scan dbl", n, "runif(n)",
             mk(r_anyNA = anyNA(xd), raw_floor = unsafe_C_pcb_raw_floor(xd),
                raw_capi = unsafe_C_pcb_na_capi(xd), raw_chunked = unsafe_C_pcb_na_chunked(xd)), rep)
      record("chunked", "NA scan int", n, "int(n)",
             mk(r_anyNA = anyNA(xi), raw_floor = unsafe_C_pcb_raw_floor(xi),
                raw_capi = unsafe_C_pcb_na_capi(xi), raw_chunked = unsafe_C_pcb_na_chunked(xi)), rep)
      record("chunked", "whole-number scan", n, "whole doubles",
             mk(r_expr = all(is.na(xw) | xw == trunc(xw)), raw_floor = unsafe_C_pcb_raw_floor(xw),
                raw_capi = unsafe_C_pcb_whole_capi(xw), raw_chunked = unsafe_C_pcb_whole_chunked(xw)), rep)
      if (n > 1) {
        record("chunked", "whole-number scan", n, "as.double(1:n)",
               mk(r_expr = all(is.na(as.double(1:n)) | as.double(1:n) == trunc(as.double(1:n))),
                  raw_floor = unsafe_C_pcb_raw_floor(as.double(1:n)),
                  raw_capi = unsafe_C_pcb_whole_capi(as.double(1:n)),
                  raw_chunked = unsafe_C_pcb_whole_chunked(as.double(1:n))), rep)
      }
    }
  }

  if ("inherits" %in% groups) {
    log("inherits")
    xl <- structure(list(1, 2, 3), class = "mx_cls")
    xl4 <- structure(list(1, 2, 3), class = c("a", "b", "c", "mx_cls"))
    record("inherits", "inherits List", 3, "list, class mx_cls",
           mk(guard_inh_type = pcb_inherits_list(xl), guard_inh = pcb_inherits_list_u(xl),
              guard_type = pcb_list(xl), no_check = pcb_list_u(xl)), rep)
    record("inherits", "inherits scan", 3, "list, class mx_cls",
           mk(sexp_floor = pcb_sexp_floor(xl), r_inherits_guard = pcb_sexp_rguard_inherits(xl),
              raw_floor = unsafe_C_pcb_raw_floor(xl), raw_rf_inherits = unsafe_C_pcb_inherits(xl)), rep)
    record("inherits", "inherits scan", 3, "list, 4 classes, match last",
           mk(sexp_floor = pcb_sexp_floor(xl4), r_inherits_guard = pcb_sexp_rguard_inherits(xl4),
              raw_floor = unsafe_C_pcb_raw_floor(xl4), raw_rf_inherits = unsafe_C_pcb_inherits(xl4)), rep)
  }

  if ("df" %in% groups) {
    for (n in c(10, 1e3, 1e5)) {
      log("df n =", n)
      d <- data.frame(a = runif(n), b = as.integer(runif(n) * 10), c = runif(n), d = runif(n),
                      e = sample(letters, n, TRUE))
      record("df", "DataFrame, 5 cols", n, "data.frame",
             mk(no_guard = pcb_df(d), guard_inherits = pcb_df_inherits(d), guard_anyNA = pcb_df_nona(d)), rep)
    }
  }

  if ("choices" %in% groups) {
    log("choices")
    record("choices", "choices &str / match_arg enum", 1, "\"slow\"",
           mk(choices_guard = pcb_choices("slow"), choices_guard_u = pcb_choices_u("slow"),
              match_arg_guard = pcb_match_arg("slow"), rust_match_only = pcb_mode_rust("slow"),
              str_floor = pcb_str_u("slow")), rep)
    record("choices", "choices &str / match_arg enum", 1, "\"sl\" (prefix)",
           mk(choices_guard = pcb_choices("sl"), match_arg_guard = pcb_match_arg("sl"),
              rust_match_only = pcb_mode_rust("sl"), str_floor = pcb_str_u("sl")), rep)
    record("choices", "choices &str / match_arg enum", 1, "omitted (default)",
           mk(choices_guard = pcb_choices(), match_arg_guard = pcb_match_arg()), rep)
  }
}
log("done")
