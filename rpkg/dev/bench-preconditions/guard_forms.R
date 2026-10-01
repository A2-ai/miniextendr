# Where does a ~340 ns R guard go? The generated guard wraps each predicate in
# isTRUE(), a closure in base R; the byte compiler does not inline it. Compare
# byte-compiled guard forms on a no-op body.
#
# Usage: Rscript rpkg/dev/bench-preconditions/guard_forms.R > guard_forms.log
# PCB_REPS (default 3) and PCB_MIN_TIME (default 0.4) as in bench.R.
suppressPackageStartupMessages(library(bench))
REPS <- as.integer(Sys.getenv("PCB_REPS", "3"))
MIN_TIME <- as.numeric(Sys.getenv("PCB_MIN_TIME", "0.4"))
cmp <- compiler::cmpfun
f_none <- cmp(function(x) x)
f_istrue1 <- cmp(function(x) { if (!isTRUE(is.integer(x))) stop("bad"); x })
f_bare1 <- cmp(function(x) { if (!is.integer(x)) stop("bad"); x })
f_istrue2 <- cmp(function(x) {
  if (!isTRUE(is.integer(x))) stop("bad")
  if (!isTRUE(length(x) == 1L)) stop("bad")
  x
})
f_bare2 <- cmp(function(x) { if (!is.integer(x) || length(x) != 1L) stop("bad"); x })
f_istrue_na <- cmp(function(x) { if (!isTRUE(!anyNA(x))) stop("bad"); x })
f_bare_na <- cmp(function(x) { if (anyNA(x)) stop("bad"); x })
isTRUE_call <- cmp(function(x) isTRUE(x))
res <- list()
for (rep in seq_len(REPS)) {
  bm <- bench::mark(
    none = f_none(1L), istrue_1 = f_istrue1(1L), bare_1 = f_bare1(1L),
    istrue_2 = f_istrue2(1L), bare_2 = f_bare2(1L),
    istrue_anyNA = f_istrue_na(1L), bare_anyNA = f_bare_na(1L),
    isTRUE_alone = isTRUE_call(TRUE),
    check = FALSE, min_time = MIN_TIME, max_iterations = 1e6
  )
  res[[rep]] <- data.frame(rep = rep, variant = as.character(bm$expression),
                           tmean_ns = vapply(bm$time, function(t) mean(as.numeric(t), trim = 0.05), numeric(1)) * 1e9)
}
d <- do.call(rbind, res)
a <- aggregate(tmean_ns ~ variant, d, median)
a$minus_none <- a$tmean_ns - a$tmean_ns[a$variant == "none"]
print(a[order(a$tmean_ns), ], row.names = FALSE, digits = 4)
