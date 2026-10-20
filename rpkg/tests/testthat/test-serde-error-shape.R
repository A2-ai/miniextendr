# Payload field types chosen by the R-facing crate (#2207), for error types
# whose `Serialize` is hand-written and uses no miniextendr field helper: a
# shape declared once for the type (`serde_error_shape!`), and the per-function
# `serde_error(shape = ..)` and `as_*` options that override it.

skip_if_not(exists("rule_err_plain"), "rpkg built without the `serde` feature")

catch <- function(expr) tryCatch(expr, error = identity)
field_types <- function(e, fields) vapply(fields, function(f) typeof(e[[f]]), "")
quota_fields <- c("what", "n", "limit", "rows")

test_that("a shape declared for the type applies to functions that don't name it", {
  a <- catch(quota_err_a(character(0), integer(0)))
  expect_equal(class(a), c("quota_exceeded", "quota", "rust_error",
                           "simpleError", "error", "condition"))
  expect_identical(a$what, character(0))
  expect_identical(a$n, 0L)
  expect_identical(a$limit, 10)
  expect_identical(a$rows, integer(0))

  b <- catch(quota_err_b(character(0), integer(0)))
  expect_identical(class(b), class(a))
  expect_identical(field_types(b, quota_fields), field_types(a, quota_fields))
  expect_identical(catch(quota_err_b(c("x", "y"), c(4L, 9L)))$rows, c(4L, 9L))
})

test_that("a method without an attribute uses the declared shape too", {
  method <- catch(quota_check(new_quotachecker(), character(0), integer(0)))
  plain <- catch(quota_err_a(character(0), integer(0)))
  expect_identical(class(method), class(plain))
  expect_identical(field_types(method, quota_fields), field_types(plain, quota_fields))
})

test_that("a function's own option overrides the declared shape for that field", {
  e <- catch(quota_err_override("x", 1:3))
  expect_s3_class(e, "quota_exceeded")
  expect_identical(e$n, 3)
  expect_identical(e$limit, 10)
  expect_identical(e$what, "x")
  expect_identical(e$rows, 1:3)
})

test_that("the declared shape, a named shape and own options apply in that order", {
  e <- catch(quota_err_chain(character(0), 1:2))
  expect_equal(class(e)[1:2], c("strict_quota_exceeded", "strict_quota"))
  expect_false("n" %in% names(e))
  expect_identical(e$count, 2)
  expect_identical(e$limit, 10)
  expect_identical(e$what, character(0))
  expect_identical(e$rows, 1:2)
})

test_that("a declared shape wins over the type's RConditionData impl", {
  e <- catch(classed_quota_err(3L))
  expect_equal(class(e)[1:3], c("declared_exceeded", "declared", "rust_error"))
  expect_false(inherits(e, "classed_by_impl"))
  expect_identical(e$n, 3)
  expect_false("impl_n" %in% names(e))
})

test_that("a shape can be declared for a type from another crate", {
  skip_if_not(exists("json_err"), "rpkg built without the `serde_json` feature")
  e <- catch(json_err(character(0), 3L))
  expect_equal(class(e)[1:3], c("json_invalid", "json", "rust_error"))
  expect_identical(e$what, character(0))
  expect_identical(e$n, 3)
})

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
  expect_identical(field_types(method, fields), field_types(shaped, fields))
  same <- setdiff(fields, "n")
  expect_identical(field_types(override, fields)[same], field_types(shaped, fields)[same])
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
