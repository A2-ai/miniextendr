# Summarise a bench.R CSV: median over reps of each variant's median (and
# trimmed mean), the spread of the rep medians, R-heap bytes, then derived
# deltas per case. Writes markdown to stdout.
#
# Usage: Rscript rpkg/dev/bench-preconditions/analyze.R <bench.csv> > summary.md
args <- commandArgs(trailingOnly = TRUE)
if (length(args) < 1L) {
  stop("usage: Rscript rpkg/dev/bench-preconditions/analyze.R <bench.csv>", call. = FALSE)
}
d <- read.csv(args[[1]], stringsAsFactors = FALSE)

key <- c("group", "case", "n", "input", "variant")
agg <- aggregate(cbind(median_ns, tmean_ns, mem_bytes) ~ group + case + n + input + variant, d, median)
lo <- aggregate(median_ns ~ group + case + n + input + variant, d, min)
hi <- aggregate(median_ns ~ group + case + n + input + variant, d, max)
names(lo)[6] <- "lo"; names(hi)[6] <- "hi"
agg <- merge(merge(agg, lo, by = key), hi, by = key)
reps <- aggregate(rep ~ group + case + n + input + variant, d, length)
names(reps)[6] <- "reps"
agg <- merge(agg, reps, by = key)

fmt_t <- function(ns) {
  ifelse(is.na(ns), "",
  ifelse(abs(ns) >= 1e6, sprintf("%.2f ms", ns / 1e6),
  ifelse(abs(ns) >= 1e4, sprintf("%.1f µs", ns / 1e3),
  ifelse(abs(ns) >= 1e3, sprintf("%.2f µs", ns / 1e3), sprintf("%.0f ns", ns)))))
}
fmt_b <- function(b) {
  ifelse(is.na(b), "",
  ifelse(b >= 1024^2, sprintf("%.1f MB", b / 1024^2),
  ifelse(b >= 1024, sprintf("%.1f KB", b / 1024), sprintf("%.0f B", b))))
}

get <- function(g, cs, n, inp, v, col = "median_ns") {
  r <- agg[agg$group == g & agg$case == cs & agg$n == n & agg$input == inp & agg$variant == v, col]
  if (length(r) == 0) NA_real_ else r[[1]]
}

cat("# Raw per-variant results (median of", max(agg$reps), "rep medians; [min-max of rep medians]; trimmed mean; R-heap bytes)\n\n")
for (g in unique(agg$group)) {
  cat("\n## group:", g, "\n\n")
  cat("| case | input | n | variant | median | rep range | trimmed mean | R heap |\n|---|---|---|---|---|---|---|---|\n")
  s <- agg[agg$group == g, ]
  s <- s[order(s$case, s$input, s$n, s$variant), ]
  for (i in seq_len(nrow(s))) {
    cat(sprintf("| %s | %s | %g | %s | %s | %s–%s | %s | %s |\n", s$case[i], s$input[i], s$n[i], s$variant[i],
                fmt_t(s$median_ns[i]), fmt_t(s$lo[i]), fmt_t(s$hi[i]), fmt_t(s$tmean_ns[i]), fmt_b(s$mem_bytes[i])))
  }
}

# Derived: guarded vs unguarded pairs (tmean is used for the delta: it
# resolves below the 41.7 ns timer tick).
pair_rows <- function(g, a, b, label_a = a, label_b = b) {
  s <- unique(agg[agg$group == g, c("case", "n", "input")])
  s <- s[order(s$case, s$n), ]
  out <- NULL
  for (i in seq_len(nrow(s))) {
    ga <- get(g, s$case[i], s$n[i], s$input[i], a, "tmean_ns")
    gb <- get(g, s$case[i], s$n[i], s$input[i], b, "tmean_ns")
    ma <- get(g, s$case[i], s$n[i], s$input[i], a, "mem_bytes")
    mb <- get(g, s$case[i], s$n[i], s$input[i], b, "mem_bytes")
    if (is.na(ga) || is.na(gb)) next
    out <- rbind(out, data.frame(case = s$case[i], input = s$input[i], n = s$n[i], a = ga, b = gb,
                                 saved = ga - gb, frac = (ga - gb) / ga, ma = ma, mb = mb))
  }
  out
}
print_pairs <- function(title, p, la, lb) {
  if (is.null(p)) return(invisible())
  cat("\n###", title, "\n\n")
  cat(sprintf("| case | input | n | %s | %s | saved | share of %s call | R heap %s | R heap %s |\n|---|---|---|---|---|---|---|---|---|\n", la, lb, la, la, lb))
  for (i in seq_len(nrow(p))) {
    cat(sprintf("| %s | %s | %g | %s | %s | %s | %.0f%% | %s | %s |\n", p$case[i], p$input[i], p$n[i],
                fmt_t(p$a[i]), fmt_t(p$b[i]), fmt_t(p$saved[i]), 100 * p$frac[i], fmt_b(p$ma[i]), fmt_b(p$mb[i])))
  }
}
cat("\n\n# Derived deltas (trimmed means)\n")
print_pairs("Scalars: default vs no_preconditions", pair_rows("scalar", "guarded", "unguarded"), "R guard + Rust", "Rust only")
print_pairs("Vectors: default vs no_preconditions", pair_rows("vector", "guarded", "unguarded"), "R guard + Rust", "Rust only")
print_pairs("no_na typed: R is.double/is.integer + anyNA vs no check", pair_rows("nona", "guard_type_na", "no_check"), "R type + anyNA", "no check")
print_pairs("no_na typed: R anyNA only vs no check", pair_rows("nona", "guard_na", "no_check"), "R anyNA", "no check")
print_pairs("no_na typed: Rust any_na (Either) vs Either without check", pair_rows("nona", "rust_any_na", "rust_either_base"), "Rust any_na", "no check")
print_pairs("NA scan, R anyNA guard vs bare-SEXP floor", pair_rows("nona", "r_anyNA_guard", "sexp_floor"), "R anyNA", "floor")
print_pairs("NA scan, raw C DATAPTR slice vs raw floor", pair_rows("nona", "raw_slice", "raw_floor"), "slice scan", "floor")
print_pairs("NA scan, raw C *_ELT (from_r::any_na shape) vs raw floor", pair_rows("nona", "raw_elt", "raw_floor"), "ELT scan", "floor")
print_pairs("NA scan, raw C NO_NA+OR_NULL+GET_REGION vs raw floor", pair_rows("nona", "raw_capi", "raw_floor"), "C-API scan", "floor")
print_pairs("ALTREP: R anyNA guard vs AltrepSexp floor", pair_rows("altrep", "r_anyNA_guard", "altrep_floor"), "R anyNA", "floor")
print_pairs("ALTREP: raw C-API scan vs raw floor", pair_rows("altrep", "raw_capi", "raw_floor"), "C-API scan", "floor")
print_pairs("ALTREP: raw *_ELT scan vs raw floor", pair_rows("altrep", "raw_elt", "raw_floor"), "ELT scan", "floor")
print_pairs("ALTREP: raw DATAPTR slice scan vs raw floor", pair_rows("altrep", "raw_slice", "raw_floor"), "slice scan", "floor")
print_pairs("ALTREP: typed R anyNA vs no check", pair_rows("altrep", "guard_na", "no_check"), "R anyNA", "no check")
print_pairs("ALTREP: typed Rust any_na vs Either base", pair_rows("altrep", "rust_any_na", "rust_either_base"), "Rust any_na", "no check")
print_pairs("ALTREP: &[i32] default vs no_preconditions", pair_rows("altrep", "guarded", "unguarded"), "R guard + Rust", "Rust only")
print_pairs("Whole-number: coerce default vs no_preconditions", pair_rows("whole", "guarded", "unguarded"), "R guard + Rust", "Rust only")
print_pairs("Whole-number: raw C slice scan vs raw floor", pair_rows("whole", "raw_slice", "raw_floor"), "slice scan", "floor")
print_pairs("Whole-number: raw C OR_NULL/GET_REGION scan vs raw floor", pair_rows("whole", "raw_capi", "raw_floor"), "C-API scan", "floor")
print_pairs("inherits List: inherits+is.list vs no check", pair_rows("inherits", "guard_inh_type", "no_check"), "R inherits+is.list", "no check")
print_pairs("inherits List: inherits only vs no check", pair_rows("inherits", "guard_inh", "no_check"), "R inherits", "no check")
print_pairs("inherits: R guard vs bare-SEXP floor", pair_rows("inherits", "r_inherits_guard", "sexp_floor"), "R inherits", "floor")
print_pairs("inherits: raw Rf_inherits vs raw floor", pair_rows("inherits", "raw_rf_inherits", "raw_floor"), "Rf_inherits", "floor")
print_pairs("DataFrame: R inherits guard vs none", pair_rows("df", "guard_inherits", "no_guard"), "R inherits", "none")
print_pairs("DataFrame: R anyNA guard vs none", pair_rows("df", "guard_anyNA", "no_guard"), "R anyNA", "none")
print_pairs("choices &str: R helper vs &str floor", pair_rows("choices", "choices_guard", "str_floor"), "R helper", "&str floor")
print_pairs("match_arg enum: R helper + Rust vs Rust matching only", pair_rows("choices", "match_arg_guard", "rust_match_only"), "R helper + Rust", "Rust only")
print_pairs("match_arg enum: Rust matching vs &str floor", pair_rows("choices", "rust_match_only", "str_floor"), "Rust match", "&str floor")
