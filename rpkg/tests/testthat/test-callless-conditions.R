# Conditions raised without a call (#1725): `call = none` on the condition
# macros, `RError::without_call()` and `#[condition(call = none)]`, the
# per-condition equivalent of R's `call. = FALSE`. Fixtures:
# src/rust/callless_condition_tests.rs. Each fixture's `FALSE` sibling keeps
# the wrapper's call, so a NULL call here is the opt-out, not a lost call.

# Run `expr`, muffling every warning and message and catching an error, and
# return the value plus every condition signalled, in order.
collect_callless <- function(expr) {
  seen <- list()
  value <- tryCatch(
    withCallingHandlers(
      expr,
      condition = function(c) {
        if (!inherits(c, "error")) seen[[length(seen) + 1L]] <<- c
        if (inherits(c, "warning")) invokeRestart("muffleWarning")
        if (inherits(c, "message")) invokeRestart("muffleMessage")
      }
    ),
    error = function(e) {
      seen[[length(seen) + 1L]] <<- e
      NULL
    }
  )
  list(value = value, conditions = seen)
}

called <- function(cond, fn) {
  call <- conditionCall(cond)
  is.call(call) && identical(call[[1]], as.name(fn))
}

# region: Immediate conditions

test_that("warning!(call = none) has no call; its sibling keeps the wrapper's", {
  w <- tryCatch(callless_warning(TRUE), warning = function(w) w)
  expect_null(conditionCall(w))
  expect_identical(
    class(w),
    c("pkg_override", "rust_warning", "simpleWarning", "warning", "condition")
  )
  expect_identical(conditionMessage(w), "row 2 overrides an earlier row for profile 3")
  expect_identical(w$kind, "warning")
  expect_identical(w$row, 2L)
  expect_identical(w$profile, 3L)

  sibling <- tryCatch(callless_warning(FALSE), warning = function(w) w)
  expect_true(called(sibling, "callless_warning"))
  expect_identical(class(sibling), class(w))
  expect_identical(sibling$row, 2L)
})

test_that("a call-less warning matches the user class and rust_warning", {
  expect_identical(
    tryCatch(callless_warning(TRUE), pkg_override = function(w) "user class"),
    "user class"
  )
  expect_identical(
    tryCatch(callless_warning(TRUE), rust_warning = function(w) "rust_warning"),
    "rust_warning"
  )
  seen <- NULL
  withCallingHandlers(
    callless_warning(TRUE),
    pkg_override = function(w) {
      seen <<- w
      invokeRestart("muffleWarning")
    }
  )
  expect_null(conditionCall(seen))
  expect_warning(callless_warning(TRUE), class = "pkg_override")
})

test_that("error!(call = none) has no call; its sibling keeps the wrapper's", {
  e <- tryCatch(callless_error(TRUE), error = function(e) e)
  expect_null(conditionCall(e))
  expect_identical(
    class(e),
    c("pkg_bad_row", "pkg_error", "rust_error", "simpleError", "error", "condition")
  )
  expect_identical(conditionMessage(e), "row 4 is malformed")
  expect_identical(e$kind, "error")
  expect_identical(e$row, 4L)
  expect_identical(
    tryCatch(callless_error(TRUE), pkg_error = function(e) "family"),
    "family"
  )
  expect_identical(
    tryCatch(callless_error(TRUE), rust_error = function(e) "rust_error"),
    "rust_error"
  )

  sibling <- tryCatch(callless_error(FALSE), error = function(e) e)
  expect_true(called(sibling, "callless_error"))
  expect_identical(class(sibling), class(e))
})

test_that("condition!(call = none) has no call; its sibling keeps the wrapper's", {
  seen <- list()
  for (callless in c(TRUE, FALSE)) {
    withCallingHandlers(
      callless_condition(callless),
      pkg_progress = function(c) seen[[length(seen) + 1L]] <<- c
    )
  }
  expect_length(seen, 2L)
  expect_null(conditionCall(seen[[1]]))
  expect_true(called(seen[[2]], "callless_condition"))
  for (c in seen) {
    expect_identical(
      class(c),
      c("pkg_progress", "rust_condition", "simpleCondition", "condition")
    )
    expect_identical(c$pct, 50L)
    expect_identical(c$kind, "condition")
  }
  # Silent without a handler, like any condition.
  expect_null(callless_condition(TRUE))
})

# endregion

# region: Printed form

test_that("a call-less condition prints like R's call. = FALSE", {
  w <- tryCatch(callless_warning(TRUE), warning = function(w) w)
  expect_identical(
    capture.output(print(w)),
    "<pkg_override: row 2 overrides an earlier row for profile 3>"
  )
  sibling <- tryCatch(callless_warning(FALSE), warning = function(w) w)
  expect_identical(
    capture.output(print(sibling)),
    "<pkg_override in callless_warning(FALSE): row 2 overrides an earlier row for profile 3>"
  )
  e <- tryCatch(callless_error(TRUE), error = function(e) e)
  expect_identical(capture.output(print(e)), "<pkg_bad_row: row 4 is malformed>")
})

test_that("the printed warning and error have no call prefix (fresh R process)", {
  out <- run_isolated({
    options(warn = 1)
    capture <- function(expr) {
      capture.output(try(expr, silent = FALSE), type = "message")
    }
    base_warning <- function() warning("row 2 overrides an earlier row for profile 3", call. = FALSE)
    base_error <- function() stop("row 4 is malformed", call. = FALSE)
    list(
      callless_warning = capture(callless_warning(TRUE)),
      sibling_warning = capture(callless_warning(FALSE)),
      base_warning = capture(base_warning()),
      callless_error = capture(callless_error(TRUE)),
      sibling_error = capture(callless_error(FALSE)),
      base_error = capture(base_error())
    )
  })
  expect_identical(out$callless_warning, "Warning: row 2 overrides an earlier row for profile 3")
  expect_identical(out$callless_warning, out$base_warning)
  expect_identical(out$sibling_warning[[1]], "Warning in callless_warning(FALSE) :")
  expect_identical(out$callless_error, "Error : row 4 is malformed")
  expect_identical(out$callless_error, out$base_error)
  expect_identical(out$sibling_error, "Error in callless_error(FALSE) : row 4 is malformed")
})

# endregion

# region: Deferred conditions

test_that("deferred call-less conditions have no call; the sibling keeps it", {
  out <- collect_callless(callless_deferred(3L))
  expect_identical(out$value, 3L)
  expect_length(out$conditions, 5L)
  macro <- out$conditions[[1]]
  overridden <- out$conditions[[2]]
  settled <- out$conditions[[3]]
  notice <- out$conditions[[4]]
  reorder <- out$conditions[[5]]

  # defer_warning!(call = none, ...)
  expect_null(conditionCall(macro))
  expect_identical(
    class(macro),
    c("pkg_override", "rust_warning", "simpleWarning", "warning", "condition")
  )
  expect_identical(macro$row, 2L)

  # #[condition(call = none)] on a variant.
  expect_null(conditionCall(overridden))
  expect_identical(
    class(overridden),
    c("pkg_override_overridden", "pkg_override", "rust_warning", "simpleWarning", "warning", "condition")
  )
  expect_identical(conditionMessage(overridden), "row 5 overrides an earlier row for profile 3")
  expect_identical(overridden$row, 5L)
  expect_identical(overridden$profile, 3L)

  # The sibling variant keeps the wrapper's call.
  expect_true(called(settled, "callless_deferred"))
  expect_identical(settled$n, 3L)

  # #[condition(call = none)] on a struct, through defer_condition().
  expect_null(conditionCall(notice))
  expect_identical(
    class(notice),
    c("pkg_notice", "rust_condition", "simpleCondition", "condition")
  )
  expect_identical(notice$n, 3L)

  # RError::without_call().
  expect_null(conditionCall(reorder))
  expect_identical(
    class(reorder),
    c("pkg_reorder", "rust_warning", "simpleWarning", "warning", "condition")
  )
  expect_identical(conditionMessage(reorder), "rows were reordered")
})

test_that("deferred call-less warnings match their classes and keep the value", {
  # The macro warning queued first has only `pkg_override`; muffle it so the
  # exiting handler sees the derived variant.
  caught <- withCallingHandlers(
    tryCatch(callless_deferred(3L), pkg_override_overridden = function(w) w),
    pkg_override = function(w) invokeRestart("muffleWarning")
  )
  expect_s3_class(caught, "pkg_override_overridden")
  expect_null(conditionCall(caught))
  expect_identical(suppressWarnings(callless_deferred(3L)), 3L)
  expect_identical(
    withCallingHandlers(
      callless_deferred(3L),
      rust_warning = function(w) invokeRestart("muffleWarning"),
      pkg_notice = function(c) NULL
    ),
    3L
  )
})

test_that("defer_message!(call = none) is accepted; a message has no call anyway", {
  out <- collect_callless(callless_deferred_message())
  expect_identical(out$value, 1L)
  expect_length(out$conditions, 2L)
  for (m in out$conditions) {
    expect_null(conditionCall(m))
    expect_identical(
      class(m),
      c("pkg_note", "rust_message", "simpleMessage", "message", "condition")
    )
  }
  expect_identical(conditionMessage(out$conditions[[1]]), "noted without a call\n")
  expect_identical(conditionMessage(out$conditions[[2]]), "noted\n")
})

# endregion

# region: Result errors

test_that("RError::without_call() raises a Result error without a call", {
  e <- tryCatch(callless_result(TRUE), error = function(e) e)
  expect_null(conditionCall(e))
  expect_identical(
    class(e),
    c("pkg_empty_profile", "pkg_error", "rust_error", "simpleError", "error", "condition")
  )
  expect_identical(conditionMessage(e), "profile 3 has no rows")
  expect_identical(e$kind, "result_err")
  expect_identical(e$profile, 3L)

  sibling <- tryCatch(callless_result(FALSE), error = function(e) e)
  expect_true(called(sibling, "callless_result"))
  expect_identical(class(sibling), class(e))
})

test_that("#[condition(call = none)] on a variant drops the call of a Result error", {
  e <- tryCatch(callless_result_derived(TRUE), pkg_row_error = function(e) e)
  expect_null(conditionCall(e))
  expect_identical(
    class(e),
    c("pkg_row_error_malformed", "pkg_row_error", "rust_error", "simpleError", "error", "condition")
  )
  expect_identical(e$row, 7L)

  sibling <- tryCatch(callless_result_derived(FALSE), pkg_row_error = function(e) e)
  expect_true(called(sibling, "callless_result_derived"))
  expect_identical(
    class(sibling),
    c("pkg_row_error_missing", "pkg_row_error", "rust_error", "simpleError", "error", "condition")
  )
})

# endregion

# region: Raising guard

test_that("a call-less error drops the raising guard's call argument", {
  # with_r_unwind_protect_or_raise raises the condition itself (the ALTREP
  # RUnwind transport), through the wrappers' helper: the same kind, classes
  # and data as a wrapper's (#1768).
  e <- tryCatch(callless_raise_guard(TRUE), pkg_guard = function(e) e)
  expect_null(conditionCall(e))
  expect_identical(
    class(e),
    c("pkg_guard", "rust_error", "simpleError", "error", "condition")
  )
  expect_identical(conditionMessage(e), "guard error")
  expect_identical(e$kind, "error")
  expect_identical(e$step, 1L)

  sibling <- tryCatch(callless_raise_guard(FALSE), pkg_guard = function(e) e)
  expect_true(called(sibling, "callless_raise_guard"))
})

# endregion

# region: Worker thread

has_worker_fixtures <- function() {
  exists("callless_worker_warning", envir = asNamespace("miniextendr"), inherits = FALSE)
}

test_that("worker: an immediate call-less warning keeps its class, data and no call", {
  skip_if_not(has_worker_fixtures(), "worker-thread feature not enabled")
  w <- tryCatch(callless_worker_warning(TRUE), warning = function(w) w)
  expect_null(conditionCall(w))
  expect_identical(
    class(w),
    c("pkg_override", "rust_warning", "simpleWarning", "warning", "condition")
  )
  expect_identical(conditionMessage(w), "worker warning")
  expect_identical(w$kind, "warning")
  expect_identical(w$row, 1L)

  sibling <- tryCatch(callless_worker_warning(FALSE), warning = function(w) w)
  expect_true(called(sibling, "callless_worker_warning"))
  expect_identical(class(sibling), class(w))
})

test_that("worker: an immediate call-less error keeps its classes, data and no call", {
  skip_if_not(has_worker_fixtures(), "worker-thread feature not enabled")
  e <- tryCatch(callless_worker_error(TRUE), pkg_worker_error = function(e) e)
  expect_null(conditionCall(e))
  expect_identical(
    class(e),
    c("pkg_worker_error", "pkg_error", "rust_error", "simpleError", "error", "condition")
  )
  expect_identical(conditionMessage(e), "worker error")
  expect_identical(e$kind, "error")
  expect_identical(e$code, 7L)

  sibling <- tryCatch(callless_worker_error(FALSE), pkg_worker_error = function(e) e)
  expect_true(called(sibling, "callless_worker_error"))
})

test_that("worker: deferred call-less warnings keep the value and no call", {
  skip_if_not(has_worker_fixtures(), "worker-thread feature not enabled")
  out <- collect_callless(callless_worker_deferred())
  expect_identical(out$value, 7L)
  expect_length(out$conditions, 2L)
  expect_null(conditionCall(out$conditions[[1]]))
  expect_identical(conditionMessage(out$conditions[[1]]), "worker data warning")
  expect_true(called(out$conditions[[2]], "callless_worker_deferred"))
  expect_identical(out$conditions[[2]]$n, 7L)
})

# endregion

# region: GC stress

test_that("call-less conditions survive gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  ok <- 0L
  for (i in seq_len(10L)) {
    w <- tryCatch(callless_warning(TRUE), warning = function(w) w)
    e <- tryCatch(callless_result(TRUE), error = function(e) e)
    out <- collect_callless(callless_deferred(3L))
    if (is.null(conditionCall(w)) && identical(w$profile, 3L) &&
        is.null(conditionCall(e)) && identical(e$profile, 3L) &&
        identical(out$value, 3L) && length(out$conditions) == 5L &&
        is.null(conditionCall(out$conditions[[2]]))) {
      ok <- ok + 1L
    }
  }
  expect_null(miniextendr:::gc_stress_condition_data())
  gctorture(FALSE)
  expect_identical(ok, 10L)
})

# endregion
