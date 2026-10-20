# `serde_error(shape = ..)` and the `as_*` boundary options (#2207): payload
# field types chosen by the R-facing function, for an error type whose
# `Serialize` is hand-written and uses no miniextendr field helper.

skip_if_not(exists("rule_err_plain"), "rpkg built without the `serde` feature")

catch <- function(expr) tryCatch(expr, error = identity)

test_that("as_character gives e$rule a character vector where the plain field is a list", {
  plain <- catch(rule_err_plain(character(0), integer(0)))
  expect_s3_class(plain, "miniextendr_error_failed")
  expect_identical(plain$rule, list())
  expect_identical(plain$rows, list())

  shaped <- catch(rule_err_shaped(character(0), integer(0)))
  expect_equal(class(shaped), c("rule_failed", "rule", "rust_error",
                                "simpleError", "error", "condition"))
  expect_identical(shaped$rule, character(0))
  expect_identical(shaped$rows, integer(0))
  expect_identical(catch(rule_err_shaped(c("a", "b"), 3L))$rule, c("a", "b"))
})

test_that("the integer and the double come through as such", {
  plain <- catch(rule_err_plain("a", c(2L, 5L)))
  expect_identical(plain$n, 2L)
  expect_identical(plain$limit, 10L)

  shaped <- catch(rule_err_shaped("a", c(2L, 5L)))
  expect_identical(shaped$n, 2L)
  expect_identical(shaped$rows, c(2L, 5L))
  expect_identical(shaped$limit, 10)
  expect_equal(conditionMessage(shaped), "rule failed on 2 rows (limit 10)")
})

test_that("both shape functions and the shaped method give the same field types", {
  fields <- c("rule", "n", "limit", "rows")
  shaped <- catch(rule_err_shaped(character(0), integer(0)))
  override <- catch(rule_err_override(character(0), integer(0)))
  method <- catch(rule_check(new_rulechecker(), character(0), integer(0)))
  types <- function(e) vapply(fields, function(f) typeof(e[[f]]), "")
  expect_identical(types(method), types(shaped))
  same <- setdiff(fields, "n")
  expect_identical(types(override)[same], types(shaped)[same])
  expect_s3_class(method, "rule_failed")
})

test_that("the function's own option overrides the shape's for that field", {
  expect_identical(catch(rule_err_shaped("a", 1:3))$n, 3L)
  override <- catch(rule_err_override("a", 1:3))
  expect_identical(override$n, 3)
  expect_identical(override$limit, 10)
  expect_s3_class(override, "rule_failed")
})

test_that("per-function as_* options work without a shape", {
  e <- catch(rule_err_own(character(0), integer(0)))
  expect_equal(class(e)[1:2], c("own_rule_failed", "own_rule"))
  expect_identical(e$rule, character(0))
  expect_identical(e$rows, integer(0))
  expect_identical(e$limit, 10)
  expect_identical(e$n, 0L)
})

test_that("a field the variant lacks is a no-op", {
  e <- catch(rule_err_unknown(character(0)))
  expect_equal(class(e)[1:2], c("rule_unknown", "rule"))
  expect_identical(e$rule, character(0))
  expect_false(any(c("n", "limit", "rows") %in% names(e)))
})

test_that("a field that doesn't fit its option raises an error naming both", {
  e <- catch(rule_err_misfit(c("a", "b"), 1L))
  expect_false(inherits(e, "rule"))
  expect_equal(e$kind, "panic")
  expect_match(
    conditionMessage(e),
    "`as_integer` on condition data field `rule` failed: the field is not an integer that fits an R integer or `None`",
    fixed = TRUE
  )
  expect_match(conditionMessage(e), "the original error was: rule failed on 1 rows (limit 10)",
               fixed = TRUE)
})
