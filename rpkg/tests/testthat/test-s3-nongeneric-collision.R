# Regression tests for #1248: an `#[miniextendr(s3)]` impl whose method name
# collides with a plain (non-generic) base/stats closure (`var`) must
# actually dispatch. Before this fix the generated guard's bare `exists()`
# check saw `var` already bound to `stats::var` and skipped creating the
# `UseMethod` dispatcher, so the registered `var.S3NonGenericCollision`
# method was generated but silently never fired — the package installed
# clean and just misbehaved (unlike the S7 sibling, #1114, which failed
# loudly at load).
#
# The S3 codegen now mirrors the S7 (#1114) classifier: it shadows the base
# closure with a package-local `UseMethod` generic and delegates a default
# method to the masked function, so ordinary (non-dispatching) calls keep
# working while S3 objects dispatch. The delegate is registered via
# base::registerS3method() so it lives ONLY in the namespace's S3 methods
# table — a literal `var.default` binding would trip roxygen2's dynamic S3
# scan (warn_missing_s3_exports).

test_that("S3 classes with a base-name-colliding method install and construct", {
  a <- new_s3nongenericcollision(values = c(2, 4, 6))
  b <- new_s3nongenericcollisionsecond(values = c(2, 4, 6))
  expect_s3_class(a, "S3NonGenericCollision")
  expect_s3_class(b, "S3NonGenericCollisionSecond")
})

test_that("colliding generic dispatches to the S3 method for our objects", {
  a <- new_s3nongenericcollision(values = c(2, 4, 6))
  # `var` returns the sum (12), deliberately != stats::var, so a pass here can
  # only mean our method ran (not the masked stats::var fallback).
  expect_equal(var(a), 12)
})

test_that("a second class sharing the shadowed generic also dispatches (reuse path)", {
  # Once the first class's method shadows `stats::var`, this class's guard
  # must classify the resulting package-local generic as already-usable
  # (isS3stdGeneric) and reuse it, rather than re-shadowing it (which would
  # silently drop the first class's registered default delegation).
  b <- new_s3nongenericcollisionsecond(values = c(2, 4, 6))
  # `var` returns the product (48) for this class, distinct from both the
  # sibling class's sum (12) and stats::var's variance.
  expect_equal(var(b), 48)
})

test_that("the masking generic falls back to the shadowed stats function", {
  # `var` masks stats::var when the package is attached, but the delegating
  # default method — present ONLY in the namespace's S3 methods table via
  # registerS3method() — must forward ordinary (non-dispatching) inputs to
  # it. Called from the test environment, outside the package namespace.
  expect_equal(var(1:10), stats::var(1:10))
  expect_equal(var(c(2, 4, 4, 4, 5, 5, 7, 9)), stats::var(c(2, 4, 4, 4, 5, 5, 7, 9)))
})

test_that("no helper bindings leak into the installed namespace (S3 or S7 path)", {
  ns <- getNamespace("miniextendr")
  # The classifier chain runs inside `local({...})`; if it were a bare
  # `else if ({...})` (the pre-fix S7 shape, #1261 item 1), `.mx_gen` would be
  # assigned in the namespace environment at wrapper-source time and leak
  # into every installed package. Pin the fix for both the S3 path (this PR)
  # and the mirrored S7 path.
  expect_false(exists(".mx_gen", envir = ns, inherits = FALSE))
  # The default-method delegate must live ONLY in the S3 methods table:
  # a literal `var.default` binding would trip roxygen2's dynamic S3 scan
  # (warn_missing_s3_exports), and the .mx_shadow_default helper is
  # base::rm()'d after registration.
  expect_false(exists("var.default", envir = ns, inherits = FALSE))
  expect_false(exists(".mx_shadow_default", envir = ns, inherits = FALSE))
  # Table-registered default dispatch is the load-bearing behavior of the
  # registerS3method shape: plain calls from outside the package still reach
  # the masked stats::var.
  expect_equal(var(1:10), stats::var(1:10))
})
