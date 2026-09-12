# Deferred conditions (#1448): a call signals a warning / message / condition
# and still returns its value. Fixtures: src/rust/deferred_condition_tests.rs.

# Run `expr`, muffling every warning and message, and return the value plus
# every condition signalled, in order.
collect_conditions <- function(expr) {
  seen <- list()
  value <- withCallingHandlers(
    expr,
    condition = function(c) {
      seen[[length(seen) + 1L]] <<- c
      if (inherits(c, "warning")) invokeRestart("muffleWarning")
      if (inherits(c, "message")) invokeRestart("muffleMessage")
    }
  )
  list(value = value, conditions = seen)
}

test_that("a deferred typed warning arrives with the value, classes, data and call", {
  out <- collect_conditions(deferred_warning_value(5L))
  expect_identical(out$value, 3L)
  expect_length(out$conditions, 1L)
  w <- out$conditions[[1]]
  expect_identical(
    class(w),
    c("pkg_warning_truncated", "pkg_warning", "rust_warning", "simpleWarning", "warning", "condition")
  )
  expect_identical(conditionMessage(w), "dropped 2 of 5 rows")
  expect_identical(w$dropped, 2L)
  expect_identical(w$total, 5L)
  expect_identical(w$kind, "warning")
  expect_identical(conditionCall(w)[[1]], as.name("deferred_warning_value"))

  # Ordinary R semantics otherwise.
  expect_warning(deferred_warning_value(5L), "dropped 2 of 5 rows")
  expect_identical(suppressWarnings(deferred_warning_value(5L)), 3L)
  expect_identical(
    tryCatch(deferred_warning_value(5L), pkg_warning = function(w) "caught"),
    "caught"
  )
  expect_identical(
    withCallingHandlers(deferred_warning_value(7L), warning = function(w) invokeRestart("muffleWarning")),
    5L
  )
})

test_that("queued conditions are signalled in order and keep their kinds", {
  out <- collect_conditions(deferred_mixed_order())
  expect_identical(out$value, 3L)
  expect_length(out$conditions, 3L)
  first <- vapply(out$conditions, function(c) class(c)[[1]], "")
  expect_identical(first, c("pkg_warning_slow_path", "rust_message", "pkg_warning_final"))
  expect_s3_class(out$conditions[[1]], "pkg_warning")
  expect_identical(conditionMessage(out$conditions[[1]]), "took the slow path")
  expect_identical(conditionMessage(out$conditions[[2]]), "step 2 of 3\n")
  expect_identical(out$conditions[[2]]$step, 2L)
  expect_identical(out$conditions[[3]]$code, 7L)
  expect_identical(conditionMessage(out$conditions[[3]]), "final warning")
})

test_that("struct payloads: Display message, renamed and Debug fields", {
  out <- collect_conditions(deferred_message_struct())
  expect_identical(out$value, "done")
  m <- out$conditions[[1]]
  expect_identical(
    class(m),
    c("retry_notice", "rust_message", "simpleMessage", "message", "condition")
  )
  expect_identical(conditionMessage(m), "succeeded after 3 attempts\n")
  expect_identical(m$n_attempts, 3L)
  expect_identical(m$range, "1..=5")
  expect_null(m$attempts)
  expect_identical(suppressMessages(deferred_message_struct()), "done")
  expect_message(deferred_message_struct(), "succeeded after 3 attempts")
})

test_that("deferred plain conditions are silent without a handler", {
  expect_silent(expect_identical(deferred_condition_value(), 0.5))
  seen <- NULL
  value <- withCallingHandlers(
    deferred_condition_value(),
    pkg_progress = function(c) seen <<- c
  )
  expect_identical(value, 0.5)
  expect_s3_class(seen, "rust_condition")
  expect_identical(seen$pct, 50)
  expect_identical(conditionMessage(seen), "halfway")

  seen <- NULL
  value <- withCallingHandlers(deferred_condition_typed(), pkg_audit = function(c) seen <<- c)
  expect_identical(value, 4L)
  expect_identical(seen$rows, 4L)
})

test_that("a deferred warning precedes the error or the immediate warning that follows", {
  log <- character()
  e <- withCallingHandlers(
    tryCatch(deferred_then_error(TRUE), error = function(e) {
      log <<- c(log, "error")
      e
    }),
    warning = function(w) {
      log <<- c(log, class(w)[[1]])
      invokeRestart("muffleWarning")
    }
  )
  expect_identical(log, c("pkg_warning_coerced", "error"))
  expect_s3_class(e, "pkg_failed")
  expect_identical(conditionMessage(e), "conversion failed")
  expect_identical(suppressWarnings(deferred_then_error(FALSE)), 1L)

  log <- character()
  e <- withCallingHandlers(
    tryCatch(deferred_then_panic(), error = function(e) {
      log <<- c(log, "error")
      e
    }),
    warning = function(w) {
      log <<- c(log, conditionMessage(w))
      invokeRestart("muffleWarning")
    }
  )
  expect_identical(log, c("about to fail", "error"))
  expect_s3_class(e, "pkg_boom")

  out <- collect_conditions(deferred_then_warning_abort())
  expect_null(out$value)
  expect_identical(
    vapply(out$conditions, conditionMessage, ""),
    c("queued", "immediate")
  )
})

test_that("unit, rng and s3-method wrappers flush the queue too", {
  expect_warning(expect_invisible(deferred_unit()), "side effect only")
  expect_warning(expect_identical(deferred_rng_value(), 1L), "rng arm")

  counter <- new_deferredcounter()
  expect_silent(expect_identical(nudge(counter, 1L), 1L))
  expect_warning(value <- nudge(counter, 1L), "count 2 above 1")
  expect_identical(value, 2L)
  w <- tryCatch(nudge(counter, 1L), pkg_counter_high = function(w) w)
  expect_identical(w$count, 3L)

  expect_message(fork(counter), "forked")
  other <- suppressMessages(fork(counter))
  expect_identical(class(other), class(counter))
  expect_identical(suppressWarnings(nudge(other, 0L)), 4L)
})

test_that("worker-thread wrappers flush the queue on the main thread", {
  skip_if_not(
    exists("deferred_worker_value", envir = asNamespace("miniextendr"), inherits = FALSE),
    "worker-thread feature not enabled"
  )
  out <- collect_conditions(deferred_worker_value())
  expect_identical(out$value, 9L)
  expect_length(out$conditions, 1L)
  expect_s3_class(out$conditions[[1]], "pkg_warning_truncated")
  expect_identical(out$conditions[[1]]$dropped, 1L)
})

test_that("nested calls flush only their own queue", {
  out <- collect_conditions(deferred_nested(function() deferred_warning_value(5L)))
  expect_identical(out$value, 103L)
  expect_identical(
    vapply(out$conditions, function(c) class(c)[[1]], ""),
    c("pkg_warning_truncated", "pkg_outer")
  )
})

test_that("deferred conditions survive gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  ok <- 0L
  for (i in seq_len(20L)) {
    out <- collect_conditions(deferred_mixed_order())
    if (identical(out$value, 3L) && length(out$conditions) == 3L &&
        identical(out$conditions[[3]]$code, 7L)) {
      ok <- ok + 1L
    }
  }
  gctorture(FALSE)
  expect_identical(ok, 20L)
})
