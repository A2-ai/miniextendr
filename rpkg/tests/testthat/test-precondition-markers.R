# Per-parameter R-side type checks (#1566): `Checked<T>` / `Unchecked<T>`,
# the per-parameter `preconditions` / `no_preconditions`, and the method-level
# bare and list forms. Fixtures: src/rust/precondition_marker_tests.rs.
#
# A kept check words a bad argument in R; a dropped one leaves it to the Rust
# conversion. Both raise the same argument-error condition (`kind =
# "conversion"`, `e$param`); only the conversion sets `e$rust_type`, which
# names the inner type, not the marker.

caught <- function(expr) tryCatch(expr, error = identity)

# The R-side check fired: its message, no Rust type.
expect_r_check <- function(e, param, message) {
  expect_s3_class(e, "rust_error")
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, param)
  expect_identical(conditionMessage(e), message)
  expect_null(e$rust_type)
}

# The Rust conversion refused it: the conversion's message and type.
expect_conversion <- function(e, param, message, rust_type) {
  expect_s3_class(e, "rust_error")
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, param)
  expect_identical(conditionMessage(e), message)
  expect_identical(e$rust_type, rust_type)
}

# region: free functions

test_that("Checked keeps a parameter's checks under the function's no_preconditions", {
  f <- miniextendr:::pm_checked
  expect_identical(f(2L, 0.5), 1)
  expect_r_check(caught(f("x", 0.5)), "n", "'n' must be integer")
  expect_r_check(caught(f(1:2, 0.5)), "n", "'n' must have length 1")
  # `tol` follows the function: the conversion words it.
  expect_conversion(caught(f(1L, "a")), "tol", "'tol' must be a single double: got character", "f64")
})

test_that("Unchecked drops one parameter's checks and keeps the others'", {
  f <- miniextendr:::pm_unchecked
  expect_identical(f(2, c(1, 2)), c(2, 4))
  expect_conversion(caught(f(2, "a")), "xs", "'xs' must be double: got character", "Vec<f64>")
  expect_r_check(caught(f("a", c(1, 2))), "factor", "'factor' must be double")
})

test_that("the per-parameter keywords generate the markers' wrappers", {
  # The same body, but for the C symbol each one calls.
  body_of <- function(f) gsub("pm_keyword_", "pm_", deparse(body(f)), fixed = TRUE)
  expect_identical(
    body_of(miniextendr:::pm_keyword_checked),
    body_of(miniextendr:::pm_checked)
  )
  expect_identical(
    body_of(miniextendr:::pm_keyword_unchecked),
    body_of(miniextendr:::pm_unchecked)
  )
  expect_r_check(caught(miniextendr:::pm_keyword_checked("x", 1)), "n", "'n' must be integer")
  expect_conversion(
    caught(miniextendr:::pm_keyword_unchecked(1, "a")),
    "xs", "'xs' must be double: got character", "Vec<f64>"
  )
})

test_that("no_na still fires under Unchecked", {
  f <- miniextendr:::pm_unchecked_no_na
  expect_identical(f(c(1, 2)), 3)
  expect_r_check(caught(f(c(1, NA))), "xs", "'xs' must not contain NA")
  expect_conversion(caught(f("a")), "xs", "'xs' must be double: got character", "Vec<f64>")
})

test_that("Checked<u16> under coerce keeps the widened guard", {
  f <- miniextendr:::pm_checked_coerce
  expect_identical(f(3), 3L)
  expect_identical(f(3L), 3L)
  e <- caught(f(-1))
  expect_s3_class(e, "rust_error")
  expect_null(e$rust_type)
  expect_identical(e$param, "n")
})

test_that("Checked<Option<i32>> accepts NULL and checks the rest", {
  f <- miniextendr:::pm_checked_option
  expect_identical(f(NULL), -1L)
  expect_identical(f(4L), 4L)
  expect_r_check(caught(f("a")), "n", "'n' must be NULL or integer")
})

test_that("a caller entry point's Unchecked parameter names the helper's call", {
  helper <- function(n) miniextendr:::pm_caller_unchecked(n, .call = parent.frame())
  expect_identical(helper(1L), 2L)
  e <- caught(helper("a"))
  expect_conversion(e, "n", "'n' must be a single integer: got character", "i32")
  expect_equal(conditionCall(e), quote(helper("a")))
})

test_that("NonNa refuses NA and NaN in the R guard and in the C wrapper", {
  f <- miniextendr:::pm_non_na_sum
  expect_identical(f(c(1, 2)), 3)
  expect_r_check(caught(f(c(1, NA))), "xs", "'xs' must not contain NA")
  expect_r_check(caught(f(c(1, NaN))), "xs", "'xs' must not contain NA")
  # A .Call() that skips the R wrapper's guard meets the C wrapper's check.
  direct <- .Call(miniextendr:::C_miniextendr_pm_non_na_sum, NULL, c(1, NA))
  expect_identical(direct$kind, "conversion")
  expect_match(direct$error, "'xs' must not contain NA", fixed = TRUE)
  expect_identical(.Call(miniextendr:::C_miniextendr_pm_non_na_sum, NULL, c(1, 2)), 3)
})

test_that("NonNa<String> refuses NA_character_ instead of reading it as empty", {
  f <- miniextendr:::pm_non_na_name
  expect_identical(f("ann"), "ann")
  expect_r_check(caught(f(NA_character_)), "name", "'name' must not be NA")
  direct <- .Call(miniextendr:::C_miniextendr_pm_non_na_name, NULL, NA_character_)
  expect_identical(direct$kind, "conversion")
  expect_match(direct$error, "'name' must not be NA", fixed = TRUE)
})

test_that("Unchecked<NonNa<..>> drops the type check and keeps the NA check", {
  f <- miniextendr:::pm_unchecked_non_na
  expect_identical(f(1:3), 6L)
  expect_r_check(caught(f(c(1L, NA))), "xs", "'xs' must not contain NA")
  e <- caught(f("a"))
  expect_identical(e$kind, "conversion")
  expect_identical(e$rust_type, "Vec<i32>")
})

test_that("NonNa on a borrowed slice checks before the rooted borrow", {
  f <- miniextendr:::pm_non_na_slice
  expect_identical(f(c(1, 2, 3)), 6)
  expect_r_check(caught(f(c(1, NA))), "xs", "'xs' must not contain NA")
  direct <- .Call(miniextendr:::C_miniextendr_pm_non_na_slice, NULL, c(NaN, 1))
  expect_identical(direct$kind, "conversion")
})

# endregion

# region: methods, one class per class system, under the impl's no_preconditions

test_that("env: preconditions(n) and a Checked parameter keep their checks", {
  obj <- miniextendr:::PmEnv$new(1L)
  expect_identical(obj$draw(2L, 2), 6)
  expect_r_check(caught(obj$draw("a", 2)), "n", "'n' must be integer")
  expect_conversion(caught(obj$draw(2L, "a")), "scale", "'scale' must be a single double: got character", "f64")
  expect_identical(obj$reseed(5L), 5L)
  expect_r_check(caught(obj$reseed("a")), "seed", "'seed' must be integer")
})

test_that("R6: preconditions(n) and a Checked parameter keep their checks", {
  obj <- miniextendr:::PmR6$new(1L)
  expect_identical(obj$draw(2L, 2), 6)
  expect_r_check(caught(obj$draw("a", 2)), "n", "'n' must be integer")
  expect_conversion(caught(obj$draw(2L, "a")), "scale", "'scale' must be a single double: got character", "f64")
  expect_identical(obj$reseed(5L), 5L)
  expect_r_check(caught(obj$reseed("a")), "seed", "'seed' must be integer")
})

test_that("R6: a NonNa parameter is the method's no_na", {
  obj <- miniextendr:::PmR6$new(1L)
  expect_identical(obj$offset(2.5), 3.5)
  expect_r_check(caught(obj$offset(NA_real_)), "by", "'by' must not be NA")
})

test_that("S3: preconditions(n) and a Checked parameter keep their checks", {
  obj <- miniextendr:::new_pms3(1L)
  expect_identical(pm_s3_draw(obj, 2L, 2), 6)
  expect_r_check(caught(pm_s3_draw(obj, "a", 2)), "n", "'n' must be integer")
  expect_conversion(caught(pm_s3_draw(obj, 2L, "a")), "scale", "'scale' must be a single double: got character", "f64")
  expect_r_check(caught(pm_s3_reseed(obj, "a")), "seed", "'seed' must be integer")
})

test_that("S4: preconditions(n) and a Checked parameter keep their checks", {
  obj <- miniextendr:::PmS4(1L)
  draw <- miniextendr:::s4_pm_s4_draw
  expect_identical(draw(obj, 2L, 2), 6)
  expect_r_check(caught(draw(obj, "a", 2)), "n", "'n' must be integer")
  expect_conversion(caught(draw(obj, 2L, "a")), "scale", "'scale' must be a single double: got character", "f64")
  expect_r_check(caught(miniextendr:::s4_pm_s4_reseed(obj, "a")), "seed", "'seed' must be integer")
})

test_that("S7: preconditions(n) and a Checked parameter keep their checks", {
  obj <- miniextendr:::PmS7(1L)
  draw <- miniextendr:::pm_s7_draw
  expect_identical(draw(obj, 2L, 2), 6)
  expect_r_check(caught(draw(obj, "a", 2)), "n", "'n' must be integer")
  expect_conversion(caught(draw(obj, 2L, "a")), "scale", "'scale' must be a single double: got character", "f64")
  expect_r_check(caught(miniextendr:::pm_s7_reseed(obj, "a")), "seed", "'seed' must be integer")
})

test_that("vctrs: the static helpers keep their checks", {
  draw <- miniextendr:::pmvctrs_draw
  expect_identical(draw(2L, 2), 4)
  expect_r_check(caught(draw("a", 2)), "n", "'n' must be integer")
  expect_conversion(caught(draw(2L, "a")), "scale", "'scale' must be a single double: got character", "f64")
  expect_r_check(caught(miniextendr:::pmvctrs_reseed("a")), "seed", "'seed' must be integer")
})

test_that("a trait impl's no_preconditions(k) drops k's checks and keeps d's", {
  obj <- miniextendr:::PmR6$new(1L)
  scaled <- miniextendr:::PmR6$PmScale$pm_scaled
  expect_identical(scaled(obj, 2, 3), 7)
  expect_conversion(caught(scaled(obj, "a", 3)), "k", "'k' must be a single double: got character", "f64")
  expect_r_check(caught(scaled(obj, 2, "a")), "d", "'d' must be double")
})

# endregion
