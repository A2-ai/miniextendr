# Rebuild hooks that report a failure with fields without raising (#2217).
#
# Fixtures: src/rust/rebuild_error_tests.rs. `HookCondition`'s hook returns
# `Err(RCondition)`, the condition a recipe check raised and the hook caught;
# `HookPair`'s hook returns the caught refusal of the `HookCondition` it holds.
# `HookQuota`'s hook (feature `serde`) returns `Err(serde_condition!(&err))`
# for a `QuotaError`, whose shape the crate declares. The refusal of a
# restored object carries the hook's condition as `e$parent`, unchanged.

reload <- function(x) unserialize(serialize(x, NULL))
catch <- function(expr) tryCatch(expr, error = identity)
refusal_class <- c(
  "mx_hook_saved", "miniextendr_restored_no_value", "miniextendr_restored",
  "rust_error", "simpleError", "error", "condition"
)

test_that("a hook's Err(RCondition) reaches R as e$parent with its classes and fields", {
  back <- reload(hook_condition_new(c(1, -2, -3)))
  e <- catch(hook_condition_total(back))
  expect_s3_class(e, refusal_class, exact = TRUE)
  expect_identical(
    conditionMessage(e),
    paste0(
      "this `HookCondition` object was restored from a saved session and rebuilding it failed: ",
      "the recipe holds 2 negative value(s); see `miniextendr:::HookCondition_saved(x)` for what was saved"
    )
  )
  expect_s3_class(
    e$parent,
    c("mx_hook_negative", "mx_hook_error", "rust_error", "simpleError", "error", "condition"),
    exact = TRUE
  )
  expect_identical(conditionMessage(e$parent), "the recipe holds 2 negative value(s)")
  expect_identical(e$parent$n, 2L)
  expect_identical(e$parent$values, c(-2, -3))
  # `call = none` stays: the cause reports no call.
  expect_null(conditionCall(e$parent))

  # Nothing was installed: the next use tries again, and fails the same way.
  again <- catch(hook_condition_total(back))
  expect_identical(class(again), class(e))
  expect_identical(again$parent$values, e$parent$values)
})

test_that("the cause is what an entry point returning the same Err(RCondition) raises", {
  back <- reload(hook_condition_new(c(1, -2, -3)))
  cause <- catch(hook_condition_total(back))$parent
  direct <- catch(hook_condition_entry(c(1, -2, -3)))
  expect_identical(class(cause), class(direct))
  expect_identical(conditionMessage(cause), conditionMessage(direct))
  expect_identical(cause$n, direct$n)
  expect_identical(cause$values, direct$values)
  expect_identical(conditionCall(cause), conditionCall(direct))
  expect_identical(hook_condition_entry(c(1, 2)), 3)
})

test_that("wrap_sexp_with_error returns a failed rebuild; the argument conversion raises it", {
  back <- reload(hook_condition_new(c(1, -2)))
  # No error: the refusal comes back as a value.
  probed <- hook_condition_probe(back)
  expect_identical(probed$class, refusal_class[1:3])
  raised <- catch(hook_condition_total(back))
  expect_s3_class(raised, refusal_class, exact = TRUE)
  expect_identical(probed$message, conditionMessage(raised))
  expect_s3_class(raised$parent, "mx_hook_negative")

  # A good recipe is rebuilt by either.
  good <- reload(hook_condition_new(c(1, 2)))
  expect_identical(hook_condition_probe(good), 3)
  expect_identical(hook_condition_total(good), 3)
})

test_that("a hook's Err(RCondition) that has a cause keeps the whole chain", {
  back <- reload(hook_pair_new(hook_condition_new(c(1, -2))))
  e <- catch(hook_pair_total(back))
  expect_s3_class(e, refusal_class[-1], exact = TRUE)
  expect_match(conditionMessage(e), "`HookPair`", fixed = TRUE)
  # The cause is the inner refusal, with the inner hook's condition as its own.
  expect_s3_class(e$parent, refusal_class, exact = TRUE)
  expect_match(conditionMessage(e$parent), "`HookCondition`", fixed = TRUE)
  expect_s3_class(e$parent$parent, "mx_hook_negative")
  expect_identical(e$parent$parent$n, 1L)
  expect_identical(e$parent$parent$values, -2)

  expect_identical(hook_pair_total(reload(hook_pair_new(hook_condition_new(c(1, 2))))), 3)
})

test_that("a hook's Err(serde_condition!(..)) carries the declared shape", {
  skip_if_not(exists("hook_quota_new"), "rpkg built without the `serde` feature")
  rows <- seq_len(11L)
  back <- reload(hook_quota_new(rows))
  e <- catch(hook_quota_n(back))
  expect_s3_class(e, refusal_class, exact = TRUE)
  direct <- catch(quota_err_a("rows", rows))
  expect_identical(class(e$parent), class(direct))
  expect_identical(class(e$parent)[1:2], c("quota_exceeded", "quota"))
  expect_identical(conditionMessage(e$parent), conditionMessage(direct))
  for (field in c("what", "n", "limit", "rows")) {
    expect_identical(e$parent[[field]], direct[[field]], label = field)
  }
  expect_identical(e$parent$n, 11L)
  expect_identical(e$parent$limit, 10)

  expect_identical(hook_quota_n(reload(hook_quota_new(1:3))), 3L)
})
