# Correctness of the chunked scans against R, and whether they expand ALTREP.
#
# Usage: Rscript rpkg/dev/bench-preconditions/chunked_check.R > chunked_check.log
ns <- asNamespace("miniextendr")
na <- ns$unsafe_C_pcb_na_chunked
wh <- ns$unsafe_C_pcb_whole_chunked
state <- function(x) {
  s <- paste(capture.output(.Internal(inspect(x))), collapse = " ")
  if (grepl("expanded", s)) "EXPANDED" else if (grepl("compact", s)) "compact" else "not-altrep"
}
set.seed(2)
inputs <- list(
  runif(1000), c(runif(999), NA), c(NaN, runif(100)), c(runif(130), NA_real_),
  sample.int(1000), c(sample.int(1000), NA), 1:1000, as.double(1:1000), numeric(0), integer(0)
)
for (x in inputs) stopifnot(identical(na(x), anyNA(x)))
w_in <- list(as.double(1:1000), c(as.double(1:200), 2.5), c(1, NA, NaN, 3), c(runif(70)), numeric(0), c(1, Inf))
for (x in w_in) stopifnot(identical(wh(x), all(is.na(x) | x == trunc(x))))
cat("chunked scans agree with anyNA() / all(is.na(x) | x == trunc(x))\n")
x <- 1:1e5; invisible(na(x)); cat("na_chunked(1:1e5):", state(x), "\n")
y <- as.double(1:1e5); invisible(na(y)); cat("na_chunked(as.double(1:1e5)):", state(y), "\n")
z <- as.double(1:1e5); invisible(wh(z)); cat("whole_chunked(as.double(1:1e5)):", state(z), "\n")
