## Comprehensive ALTREP tests
## Run from R: library(rpkg); source("dev/altrep.R")

test_altrep <- function() {
  # Use internal namespace for unexported functions
  altrep_compact_int <- rpkg:::altrep_compact_int
  altrep_from_doubles <- rpkg:::altrep_from_doubles
  altrep_from_strings <- rpkg:::altrep_from_strings
  altrep_from_logicals <- rpkg:::altrep_from_logicals
  altrep_from_raw <- rpkg:::altrep_from_raw
  altrep_from_list <- rpkg:::altrep_from_list
  altrep_constant_int <- rpkg:::altrep_constant_int

  cat("========================================\n")
  cat("ALTREP Comprehensive Test Suite\n")
  cat("========================================\n\n")

  # ============================================
  # 1. MANUAL BACKEND TESTS (dyn trait approach)
  # ============================================

  cat("--- 1. INTEGER: compact sequence ---\n")
  x <- altrep_compact_int(5L, 10L, 2L)  # 10, 12, 14, 16, 18
  stopifnot(length(x) == 5L)
  stopifnot(x[1] == 10L)
  stopifnot(x[5] == 18L)
  stopifnot(sum(x) == 70L)  # 10+12+14+16+18
  stopifnot(all(x == c(10L, 12L, 14L, 16L, 18L)))
  cat("  length:", length(x), "elements:", paste(x, collapse=","), "sum:", sum(x), "\n")

  cat("--- 2. INTEGER: descending sequence ---\n")
  x2 <- altrep_compact_int(4L, 100L, -10L)  # 100, 90, 80, 70
  stopifnot(length(x2) == 4L)
  stopifnot(x2[1] == 100L)
  stopifnot(x2[4] == 70L)
  stopifnot(sum(x2) == 340L)
  cat("  length:", length(x2), "elements:", paste(x2, collapse=","), "\n")

  cat("--- 3. REAL: from doubles ---\n")
  y <- altrep_from_doubles(c(1.5, 2.5, 3.5, NA_real_))
  stopifnot(length(y) == 4L)
  stopifnot(y[1] == 1.5)
  stopifnot(y[3] == 3.5)
  stopifnot(is.na(y[4]))
  stopifnot(is.na(sum(y)))         # NA propagates
  stopifnot(sum(y, na.rm = TRUE) == 7.5)
  cat("  length:", length(y), "sum(na.rm=T):", sum(y, na.rm=TRUE), "\n")

  cat("--- 4. STRING: from character ---\n")
  z <- altrep_from_strings(c("hello", NA_character_, "world", ""))
  stopifnot(length(z) == 4L)
  stopifnot(z[1] == "hello")
  stopifnot(is.na(z[2]))
  stopifnot(z[3] == "world")
  stopifnot(z[4] == "")
  stopifnot(nchar(z[1]) == 5L)
  cat("  length:", length(z), "first:", z[1], "has NA:", any(is.na(z)), "\n")

  cat("--- 5. LOGICAL: from logicals ---\n")
  l <- altrep_from_logicals(c(TRUE, FALSE, NA, TRUE))
  stopifnot(length(l) == 4L)
  stopifnot(l[1] == TRUE)
  stopifnot(l[2] == FALSE)
  stopifnot(is.na(l[3]))
  stopifnot(sum(l, na.rm = TRUE) == 2L)  # TRUE + TRUE
  stopifnot(any(l, na.rm = TRUE) == TRUE)
  stopifnot(all(l, na.rm = TRUE) == FALSE)  # FALSE is present
  cat("  length:", length(l), "sum(na.rm=T):", sum(l, na.rm=TRUE), "\n")

  cat("--- 6. RAW: from raw bytes ---\n")
  r <- altrep_from_raw(as.raw(c(0, 127, 255)))
  stopifnot(length(r) == 3L)
  stopifnot(r[1] == as.raw(0))
  stopifnot(r[2] == as.raw(127))
  stopifnot(r[3] == as.raw(255))
  cat("  length:", length(r), "bytes:", paste(as.integer(r), collapse=","), "\n")

  cat("--- 7. LIST: from list ---\n")
  w <- altrep_from_list(list(a = 1L, b = "two", c = 3.0))
  stopifnot(length(w) == 3L)
  stopifnot(w[[1]] == 1L)
  stopifnot(w[[2]] == "two")
  stopifnot(w[[3]] == 3.0)
  cat("  length:", length(w), "types:", paste(sapply(w, typeof), collapse=","), "\n")

  # ============================================
  # 2. EDGE CASES
  # ============================================

  cat("\n--- 8. EDGE: empty vectors ---\n")
  empty_int <- altrep_compact_int(0L, 0L, 1L)
  stopifnot(length(empty_int) == 0L)
  stopifnot(sum(empty_int) == 0L)

  empty_real <- altrep_from_doubles(double(0))
  stopifnot(length(empty_real) == 0L)

  empty_str <- altrep_from_strings(character(0))
  stopifnot(length(empty_str) == 0L)
  cat("  empty vectors work correctly\n")

  cat("--- 9. EDGE: single element ---\n")
  single <- altrep_compact_int(1L, 42L, 1L)
  stopifnot(length(single) == 1L)
  stopifnot(single[1] == 42L)
  cat("  single element:", single[1], "\n")

  cat("--- 10. EDGE: subsetting ---\n")
  seq5 <- altrep_compact_int(5L, 1L, 1L)  # 1,2,3,4,5
  sub <- seq5[c(1, 3, 5)]
  stopifnot(length(sub) == 3L)
  stopifnot(all(sub == c(1L, 3L, 5L)))
  cat("  subset [1,3,5]:", paste(sub, collapse=","), "\n")

  # ============================================
  # 3. PROC-MACRO ALTREP (ConstantIntClass)
  # ============================================

  cat("\n--- 11. PROC-MACRO ALTREP: ConstantIntClass ---\n")
  c42 <- altrep_constant_int()
  stopifnot(length(c42) == 10L)
  stopifnot(all(c42 == 42L))
  stopifnot(sum(c42) == 420L)  # 10 * 42
  stopifnot(mean(c42) == 42)
  stopifnot(min(c42) == 42L)
  stopifnot(max(c42) == 42L)
  cat("  length:", length(c42), "all 42?:", all(c42 == 42), "sum:", sum(c42), "\n")

  cat("--- 12. PROC-MACRO: multiple instances ---\n")
  c1 <- altrep_constant_int()
  c2 <- altrep_constant_int()
  c3 <- altrep_constant_int()
  stopifnot(all(c1 == c2))
  stopifnot(all(c2 == c3))
  stopifnot(sum(c1) + sum(c2) + sum(c3) == 1260L)  # 420 * 3
  cat("  3 instances created, all consistent\n")

  cat("--- 13. PROC-MACRO: subsetting ---\n")
  sub42 <- c42[1:5]
  stopifnot(length(sub42) == 5L)
  stopifnot(all(sub42 == 42L))
  cat("  subset [1:5] length:", length(sub42), "\n")

  # ============================================
  # 4. OPERATIONS ON ALTREP
  # ============================================

  cat("\n--- 14. OPERATIONS: arithmetic ---\n")
  seq_a <- altrep_compact_int(3L, 1L, 1L)  # 1,2,3
  result <- seq_a + 10L
  stopifnot(all(result == c(11L, 12L, 13L)))
  cat("  1:3 + 10 =", paste(result, collapse=","), "\n")

  cat("--- 15. OPERATIONS: comparison ---\n")
  cmp <- seq_a > 1L
  stopifnot(cmp[1] == FALSE)
  stopifnot(cmp[2] == TRUE)
  stopifnot(cmp[3] == TRUE)
  cat("  1:3 > 1:", paste(cmp, collapse=","), "\n")

  cat("\n========================================\n")
  cat("All ALTREP tests passed!\n")
  cat("========================================\n")

  invisible(TRUE)
}

# Run tests when sourced
if (interactive() || !exists(".altrep_test_skip")) {
  test_altrep()
}
