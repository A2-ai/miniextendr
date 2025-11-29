# Test slice input conversions

library(rpkg)

cat("=== Testing slice conversions ===\n\n")

# i32 slice tests
cat("i32 slice tests:\n")
stopifnot(test_i32_slice_len(1:5) == 5L)
stopifnot(test_i32_slice_len(integer(0)) == 0L)
stopifnot(test_i32_slice_sum(1:5) == 15L)
stopifnot(test_i32_slice_sum(integer(0)) == 0L)
stopifnot(test_i32_slice_first(c(10L, 20L, 30L)) == 10L)
stopifnot(test_i32_slice_first(integer(0)) == 0L)
stopifnot(test_i32_slice_last(c(10L, 20L, 30L)) == 30L)
stopifnot(test_i32_slice_last(integer(0)) == 0L)
cat("  PASS: test_i32_slice_len, test_i32_slice_sum, test_i32_slice_first, test_i32_slice_last\n")

# f64 slice tests
cat("f64 slice tests:\n")
stopifnot(test_f64_slice_len(c(1.0, 2.0, 3.0)) == 3L)
stopifnot(test_f64_slice_len(numeric(0)) == 0L)
stopifnot(test_f64_slice_sum(c(1.0, 2.0, 3.0)) == 6.0)
stopifnot(test_f64_slice_sum(numeric(0)) == 0.0)
stopifnot(test_f64_slice_mean(c(2.0, 4.0, 6.0)) == 4.0)
stopifnot(test_f64_slice_mean(numeric(0)) == 0.0)
cat("  PASS: test_f64_slice_len, test_f64_slice_sum, test_f64_slice_mean\n")

# u8 (raw) slice tests
cat("u8 (raw) slice tests:\n")
stopifnot(test_u8_slice_len(as.raw(1:5)) == 5L)
stopifnot(test_u8_slice_len(raw(0)) == 0L)
stopifnot(test_u8_slice_sum(as.raw(1:5)) == 15L)
stopifnot(test_u8_slice_sum(raw(0)) == 0L)
cat("  PASS: test_u8_slice_len, test_u8_slice_sum\n")

# logical slice tests
cat("logical slice tests:\n")
stopifnot(test_logical_slice_len(c(TRUE, FALSE, TRUE)) == 3L)
stopifnot(test_logical_slice_len(logical(0)) == 0L)
stopifnot(test_logical_slice_any_true(c(FALSE, FALSE, TRUE)) == TRUE)
stopifnot(test_logical_slice_any_true(c(FALSE, FALSE, FALSE)) == FALSE)
stopifnot(test_logical_slice_any_true(logical(0)) == FALSE)
stopifnot(test_logical_slice_all_true(c(TRUE, TRUE, TRUE)) == TRUE)
stopifnot(test_logical_slice_all_true(c(TRUE, FALSE, TRUE)) == FALSE)
stopifnot(test_logical_slice_all_true(logical(0)) == TRUE)  # vacuous truth
cat("  PASS: test_logical_slice_len, test_logical_slice_any_true, test_logical_slice_all_true\n")

cat("\n=== All slice conversion tests passed! ===\n")
