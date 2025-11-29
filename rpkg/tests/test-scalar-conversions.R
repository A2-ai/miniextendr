# Test scalar input/output conversions

library(rpkg)

cat("=== Testing scalar conversions ===\n\n")

# i32 tests
cat("i32 tests:\n")
stopifnot(test_i32_identity(42L) == 42L)
stopifnot(test_i32_add_one(41L) == 42L)
stopifnot(test_i32_sum(1L, 2L, 3L) == 6L)
stopifnot(test_i32_sum(-10L, 5L, 5L) == 0L)
cat("  PASS: test_i32_identity, test_i32_add_one, test_i32_sum\n")

# f64 tests
cat("f64 tests:\n")
stopifnot(test_f64_identity(3.14) == 3.14)
stopifnot(test_f64_add_one(2.5) == 3.5)
stopifnot(test_f64_multiply(2.0, 3.0) == 6.0)
stopifnot(test_f64_multiply(-2.5, 4.0) == -10.0)
cat("  PASS: test_f64_identity, test_f64_add_one, test_f64_multiply\n")

# u8 (raw) tests
cat("u8 (raw) tests:\n")
stopifnot(test_u8_identity(as.raw(42)) == as.raw(42))
stopifnot(test_u8_add_one(as.raw(41)) == as.raw(42))
stopifnot(test_u8_add_one(as.raw(255)) == as.raw(0))  # wrapping
cat("  PASS: test_u8_identity, test_u8_add_one\n")

# Rboolean tests
cat("logical tests:\n")
stopifnot(test_logical_identity(TRUE) == TRUE)
stopifnot(test_logical_identity(FALSE) == FALSE)
stopifnot(test_logical_not(TRUE) == FALSE)
stopifnot(test_logical_not(FALSE) == TRUE)
stopifnot(test_logical_and(TRUE, TRUE) == TRUE)
stopifnot(test_logical_and(TRUE, FALSE) == FALSE)
stopifnot(test_logical_and(FALSE, TRUE) == FALSE)
stopifnot(test_logical_and(FALSE, FALSE) == FALSE)
cat("  PASS: test_logical_identity, test_logical_not, test_logical_and\n")

# Mixed type tests
cat("mixed type tests:\n")
stopifnot(test_i32_to_f64(42L) == 42.0)
stopifnot(test_i32_to_f64(-10L) == -10.0)
stopifnot(test_f64_to_i32(42.9) == 42L)
stopifnot(test_f64_to_i32(-3.7) == -3L)
cat("  PASS: test_i32_to_f64, test_f64_to_i32\n")

cat("\n=== All scalar conversion tests passed! ===\n")
