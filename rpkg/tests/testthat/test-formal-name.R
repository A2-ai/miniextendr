# Declared R formals (#1842): `formal = "row.names"` gives a parameter an R
# name that is no Rust identifier (src/rust/formal_name_tests.rs).

caught <- function(expr) tryCatch(expr, error = identity)

mx_formal_fit <- function() {
  structure(list(estimate = 2, se = 0.5), class = "mx_formal_fit")
}

# region: S3 methods of base generics

test_that("as.data.frame() takes row.names and optional by name and position", {
  fit <- mx_formal_fit()
  expect_identical(
    names(formals(miniextendr:::as.data.frame.mx_formal_fit)),
    c("x", "row.names", "optional", "...")
  )
  df <- as.data.frame(fit)
  expect_identical(df$estimate, 2)
  expect_identical(df$row_name, NA_character_)
  expect_false(df$optional)
  df <- as.data.frame(fit, row.names = "a", optional = TRUE)
  expect_identical(df$row_name, "a")
  expect_true(df$optional)
  # By position, as `as.data.frame(x, row.names, optional)` reads them.
  df <- as.data.frame(fit, NULL, TRUE)
  expect_identical(df$row_name, NA_character_)
  expect_true(df$optional)
  # By partial name.
  df <- as.data.frame(fit, row = "b", opt = TRUE)
  expect_identical(df$row_name, "b")
  expect_true(df$optional)
  # `data.frame()` passes `optional = TRUE` by name.
  expect_identical(data.frame(fit)$optional, TRUE)
})

test_that("summary() takes conf.int and conf.level", {
  fit <- mx_formal_fit()
  expect_identical(summary(fit), 2)
  expect_equal(summary(fit, conf.int = TRUE), c(2, 2 - 1.959964 * 0.5, 2 + 1.959964 * 0.5))
  expect_equal(
    summary(fit, conf.int = TRUE, conf.level = 0.99),
    c(2, 2 - 2.575829 * 0.5, 2 + 2.575829 * 0.5)
  )
  e <- caught(summary(fit, conf.level = NA_real_))
  expect_identical(e$param, "conf.level")
  expect_equal(conditionCall(e), quote(summary(fit, conf.level = NA_real_)))
})

test_that("the S3 methods pass tools::checkS3methods()", {
  expect_length(tools::checkS3methods(package = "miniextendr"), 0L)
})

# endregion

# region: standalone functions

test_that("a dotted formal with a default is matched like any formal", {
  f <- miniextendr:::mx_formal_mean
  expect_identical(names(formals(f)), c("x", "na.rm"))
  expect_identical(formals(f)$na.rm, FALSE)
  expect_true(is.na(f(c(1, NA, 3))))
  expect_identical(f(c(1, NA, 3), na.rm = TRUE), 2)
  expect_identical(f(c(1, NA, 3), TRUE), 2)
  expect_identical(f(c(1, NA, 3), na = TRUE), 2)
})

test_that("a no_na guard names the dotted formal", {
  e <- caught(miniextendr:::mx_formal_mean(1, na.rm = NA))
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "na.rm")
  expect_null(e$rust_type)
  expect_match(conditionMessage(e), "'na.rm'", fixed = TRUE)
  e <- caught(miniextendr:::mx_formal_mean(1, na.rm = "yes"))
  expect_identical(e$param, "na.rm")
})

test_that("a Missing<T> parameter with a dotted formal", {
  f <- miniextendr:::mx_formal_cap
  expect_identical(names(formals(f)), c("x", "cap.at"))
  # The default is written on the dotted formal (#2161).
  expect_identical(formals(f)$cap.at, quote(Inf))
  expect_identical(f(c(1, 5)), c(1, 5))
  expect_identical(f(c(1, 5), cap.at = Inf), c(1, 5))
  expect_identical(f(c(1, 5), cap.at = 3), c(1, 3))
  expect_identical(f(c(1, 5), 3), c(1, 3))
})

test_that("match_arg and choices parameters with dotted formals", {
  f <- miniextendr:::mx_formal_choice
  expect_identical(names(formals(f)), c("ci.method", "out.type"))
  expect_identical(formals(f)$ci.method, quote(c("wald", "profile")))
  expect_identical(formals(f)$out.type, quote(c("vector", "list")))
  expect_identical(f(), "Wald/vector")
  expect_identical(f(ci.method = "prof", out.type = "list"), "Profile/list")
  expect_identical(f("profile", "list"), "Profile/list")
  e <- caught(f(ci.method = "bogus"))
  expect_identical(e$param, "ci.method")
  expect_match(conditionMessage(e), "ci.method", fixed = TRUE)
  e <- caught(f(out.type = "matrix"))
  expect_identical(e$param, "out.type")
  expect_match(conditionMessage(e), "out.type", fixed = TRUE)
})

test_that("a conversion error raised in Rust names the dotted formal", {
  expect_identical(miniextendr:::mx_formal_unchecked(4L), 8L)
  e <- caught(miniextendr:::mx_formal_unchecked("a"))
  expect_identical(e$kind, "conversion")
  # The conversion in Rust, not an R-side check, refused it.
  expect_identical(e$rust_type, "i32")
  expect_identical(e$param, "n.boot")
  expect_match(conditionMessage(e), "'n.boot'", fixed = TRUE)
})

# endregion

# region: impl-block methods

test_that("an R6 class takes dotted formals in its constructor and methods", {
  expect_identical(
    names(formals(MxFormalTally$public_methods$initialize)),
    c("start.value", ".ptr")
  )
  expect_identical(
    names(formals(MxFormalTally$public_methods$add)),
    c("by.value", "na.rm")
  )
  tally <- MxFormalTally$new(start.value = 1)
  expect_identical(tally$add(2), 3)
  expect_identical(tally$add(by.value = NA_real_, na.rm = TRUE), 3)
  expect_identical(tally$add(4, TRUE), 7)
  e <- caught(tally$add(by.value = "x"))
  expect_identical(e$param, "by.value")
})

test_that("an S7 class takes dotted formals in its constructor and methods", {
  est <- MxFormalEstimate(2, std.error = 0.5)
  expect_equal(formal_interval(est), c(2 - 1.959964 * 0.5, 2 + 1.959964 * 0.5))
  expect_equal(
    formal_interval(est, conf.level = 0.99),
    c(2 - 2.575829 * 0.5, 2 + 2.575829 * 0.5)
  )
  expect_true("std.error" %in% names(formals(MxFormalEstimate)))
  method <- S7::method(formal_interval, MxFormalEstimate)
  expect_identical(names(formals(method)), c("x", "conf.level", "..."))
  e <- caught(formal_interval(est, conf.level = "high"))
  expect_identical(e$param, "conf.level")
})

# endregion

test_that("no Rust parameter name reaches the R side", {
  ns <- asNamespace("miniextendr")
  fns <- list(
    ns$as.data.frame.mx_formal_fit, ns$summary.mx_formal_fit, ns$mx_formal_mean,
    ns$mx_formal_cap, ns$mx_formal_choice, ns$mx_formal_unchecked,
    MxFormalTally$public_methods$initialize, MxFormalTally$public_methods$add,
    MxFormalEstimate, S7::method(formal_interval, MxFormalEstimate)
  )
  for (f in fns) {
    expect_false(any(grepl("fml_", deparse(f), fixed = TRUE)))
  }
})
