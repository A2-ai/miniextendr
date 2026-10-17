test_that("callbacks accept closures, builtins and specials on every call path", {
  for (apply in list(rfunction_apply, rfunction_apply_unchecked,
                     rfunction_through_view)) {
    offset <- 5L
    expect_identical(apply(function(x) x + offset, 2L), 7L)
    expect_identical(apply(base::sum, c(1L, 2L, 3L)), 6L)
    expr <- quote(stop("must stay unevaluated"))
    expect_identical(apply(base::quote, expr), expr)
  }
  expect_identical(rfunction_through_generic_shim(function(x) x + 1L, 4L), 5L)
  host <- CallbackHost$new()
  expect_identical(host$apply(base::sum, 1:3), 6L)
  expect_identical(host$CallbackTrait$invoke(base::sum, 1:3), 6L)
  expect_identical(host$GenericCallback$invoke_generic(function(x) x + 1L, 4L), 5L)
})

test_that("NULL is optional and non-functions raise classed argument errors", {
  for (apply in list(rfunction_optional, rfunction_optional_unchecked)) {
    expect_identical(apply(NULL, 7L), 7L)
    expect_identical(apply(base::sum, 1:3), 6L)
  }
  for (apply in list(rfunction_apply, rfunction_apply_unchecked,
                     rfunction_optional, rfunction_optional_unchecked)) {
    for (bad in list(1L, "sum", quote(sum), list(), NA)) {
      err <- tryCatch(apply(bad, 1L), error = identity)
      expect_s3_class(err, "rust_error")
      expect_identical(err$kind, "conversion")
      expect_identical(err$param, "f")
      expect_match(conditionMessage(err), "'f' must be (NULL or )?a function")
    }
  }
  expect_error(rfunction_apply(NULL, 1L), "'f' must be a function")
  expect_identical(
    conditionMessage(tryCatch(rfunction_apply_unchecked(1L, 2L), error = identity)),
    "'f' must be a function: got integer"
  )
  expect_identical(
    conditionMessage(tryCatch(rfunction_optional_unchecked(1L, 2L), error = identity)),
    "'f' must be NULL or a function: got integer"
  )
})

test_that("callback exits preserve R handlers and release Rust locals and borrows", {
  host <- CallbackHost$new()
  before <- rfunction_drops()
  condition <- errorCondition("callback failure", class = "callback_failed")
  caught <- tryCatch(host$apply(function(x) stop(condition), 1L), error = identity)
  expect_identical(caught, condition)
  expect_identical(rfunction_drops(), before + 1L)
  expect_identical(host$apply(identity, 2L), 2L)

  before <- rfunction_drops()
  escaped <- withRestarts(
    withCallingHandlers(
      host$apply(function(x) { warning("leave callback"); x }, 3L),
      warning = function(w) invokeRestart("escape", 42L)
    ),
    escape = identity
  )
  expect_identical(escaped, 42L)
  expect_identical(rfunction_drops(), before + 1L)
  expect_identical(host$apply(identity, 4L), 4L)

  observed <- NULL
  value <- withCallingHandlers(
    rfunction_apply(function(x) { warning("callback warning"); x }, 5L),
    warning = function(w) {
      observed <<- conditionMessage(w)
      invokeRestart("muffleWarning")
    }
  )
  expect_identical(value, 5L)
  expect_identical(observed, "callback warning")
})

test_that("trait callback exits retain the native condition and run destructors", {
  for (apply in list(rfunction_through_view, rfunction_through_generic_shim)) {
    condition <- errorCondition("trait callback failure", class = "callback_failed")
    before <- rfunction_drops()
    caught <- tryCatch(apply(function(x) stop(condition), 1L), error = identity)
    expect_identical(caught, condition)
    expect_identical(rfunction_drops(), before + 1L)
    expect_identical(apply(identity, 2L), 2L)
  }
})

test_that("plain string validators retain the conversion condition contract", {
  for (validate in list(validator_str, validator_string)) {
    expect_identical(validate(2L), 2L)
    err <- tryCatch(validate(-1L), error = identity)
    expect_s3_class(err, "rust_error")
    expect_identical(err$kind, "conversion")
    expect_identical(err$param, "x")
    expect_match(conditionMessage(err), "must be positive", fixed = TRUE)
  }
})

test_that("fresh callback closures and builders survive GC torture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  gc_stress_rfunction()
  value <- rfunction_apply(
    local({ offset <- 8L; function(x) { gc(); x + offset } }),
    2L
  )
  gctorture(FALSE)
  expect_identical(value, 10L)
})
