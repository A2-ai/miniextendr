## Quick ALTREP smoke tests
## Run from R after building/installing the package:
##   devtools::load_all(".") or library(rpkg)

altrep_smoke <- function() {
  cat("INT compact seq...\n")
  x <- altrep_compact_int(5L, 10L, 1L)
  stopifnot(length(x) == 5L)
  print(x)

  cat("REAL clone...\n")
  y <- altrep_from_doubles(c(1, 2, 3.5))
  stopifnot(length(y) == 3L)
  print(y)

  cat("STRING clone...\n")
  z <- altrep_from_strings(c("a", NA, "ccc"))
  stopifnot(length(z) == 3L)
  print(z)

  cat("LOGICAL clone...\n")
  l <- altrep_from_logicals(c(TRUE, FALSE, NA))
  stopifnot(length(l) == 3L)
  print(l)

  cat("RAW clone...\n")
  r <- altrep_from_raw(as.raw(c(1, 2, 255)))
  stopifnot(length(r) == 3L)
  print(r)

  cat("LIST clone...\n")
  w <- altrep_from_list(list(1L, 2.0, "x"))
  stopifnot(length(w) == 3L)
  print(w)

  invisible(TRUE)
}

