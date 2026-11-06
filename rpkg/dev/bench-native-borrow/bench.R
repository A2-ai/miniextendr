# Success-path cost of the native-borrow preflight (#1504), at the R call
# level: each case calls a `nbb_*` fixture of
# rpkg/src/rust/native_borrow_bench_fixtures.rs through its R wrapper.
#
# Usage, from the repository root (so rv activates the project library):
#   Rscript rpkg/dev/bench-native-borrow/bench.R <out.csv> <label> [rep]
# Appends one row per case to <out.csv>, tagged <label> and <rep>. Set NBB_LIB
# to a library holding another build of miniextendr to measure that build
# instead (it is put first on .libPaths()). NBB_MIN_TIME (seconds, default
# 0.5) is bench::mark's min_time. run.R alternates two builds and calls
# compare.R.

args <- commandArgs(trailingOnly = TRUE)
if (length(args) < 2L) {
  stop("usage: Rscript rpkg/dev/bench-native-borrow/bench.R <out.csv> <label> [rep]", call. = FALSE)
}
out_csv <- args[[1]]
label <- args[[2]]
rep <- if (length(args) >= 3L) as.integer(args[[3]]) else 1L
MIN_TIME <- as.numeric(Sys.getenv("NBB_MIN_TIME", "0.5"))
lib <- Sys.getenv("NBB_LIB", "")
if (nzchar(lib)) .libPaths(c(lib, .libPaths()))

suppressPackageStartupMessages(library(bench))
ns <- asNamespace("miniextendr")
# Bound to local names: a `:::` lookup inside the measured expression would
# cost about as much as some of the cases.
for (nm in grep("^nbb_", ls(ns, all.names = TRUE), value = TRUE)) {
  assign(nm, get(nm, ns))
}

set.seed(1)
x <- runif(1000)
y <- runif(1000)
iv <- as.integer(runif(1000) * 1000)
s <- 2.5

cases <- list(
  plain = quote(nbb_plain(1L)),
  slice = quote(nbb_slice(x)),
  slices = quote(nbb_slices(x, y, iv)),
  str = quote(nbb_str("abc")),
  strs = quote(nbb_strs("abc", "def", c("x", "y", "z"))),
  either_left = quote(nbb_either(1L, x)),
  either_right = quote(nbb_either(y, x)),
  either_alone_left = quote(nbb_either_alone(1L)),
  either_alone_right = quote(nbb_either_alone(x)),
  either_left_aliased = quote(nbb_either_alias(s, s))
)

rows <- list()
for (case in names(cases)) {
  expr <- cases[[case]]
  # Warm-up: one-time costs (lazy loads, first promise forcing) stay out.
  eval(expr)
  bm <- bench::mark(exprs = list(expr), env = environment(), check = FALSE,
                    min_time = MIN_TIME, max_iterations = 1e7, min_iterations = 10,
                    filter_gc = FALSE)
  t <- as.numeric(bm$time[[1]])
  rows[[case]] <- data.frame(
    label = label, rep = rep, case = case,
    median_ns = as.numeric(bm$median) * 1e9,
    # 5%-trimmed mean: the macOS timer ticks at ~41.7 ns, so a median is
    # quantised; the mean dithers across ticks.
    tmean_ns = mean(t, trim = 0.05) * 1e9,
    mem_bytes = as.numeric(bm$mem_alloc),
    n_itr = bm$n_itr,
    stringsAsFactors = FALSE
  )
}
out <- do.call(rbind, rows)
write.table(out, out_csv, sep = ",", row.names = FALSE,
            col.names = !file.exists(out_csv), append = file.exists(out_csv))
