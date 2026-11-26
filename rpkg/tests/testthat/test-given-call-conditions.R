# Conditions with a given call (#1852): `call = <expr>` on the condition
# macros and `arg_error!`, `RError::with_call()` and
# `#[condition(call = <expr>)]`. Fixtures:
# src/rust/given_call_condition_tests.rs. Each fixture takes the call to
# report as an argument, so `conditionCall()` must be that call, not the
# fixture's own.

assign_call <- quote(x$table <- value)

# region: Immediate conditions

test_that("error!(call = <expr>) reports the given call", {
  e <- tryCatch(given_call_error(assign_call), error = function(e) e)
  expect_identical(conditionCall(e), assign_call)
  expect_identical(
    class(e),
    c("pkg_bad_table", "rust_error", "simpleError", "error", "condition")
  )
  expect_identical(conditionMessage(e), "the table has no `id` column")
  expect_identical(e$column, "id")
  expect_identical(e$kind, "error")
  expect_error(given_call_error(assign_call), "^the table has no", class = "pkg_bad_table")
  # The printed error names the call the user wrote.
  expect_match(
    paste(capture.output(print(e)), collapse = "\n"),
    "x$table <- value",
    fixed = TRUE
  )
})

test_that("warning!(call = <expr>) and condition!(call = <expr>) report the given call", {
  w <- tryCatch(given_call_warning(assign_call), warning = function(w) w)
  expect_identical(conditionCall(w), assign_call)
  expect_s3_class(w, "pkg_table_warning")
  expect_identical(conditionMessage(w), "the table is empty")

  seen <- NULL
  withCallingHandlers(
    given_call_condition(quote(f(1L, n = "a", g(TRUE)))),
    pkg_progress = function(c) seen <<- c
  )
  expect_identical(conditionCall(seen), quote(f(1L, n = "a", g(TRUE))))
  expect_s3_class(seen, "rust_condition")
})

test_that("arg_error!(call = <expr>) reports the given call", {
  e <- tryCatch(given_call_arg_error(assign_call, -1L), error = function(e) e)
  expect_identical(conditionCall(e), assign_call)
  expect_identical(conditionMessage(e), "'value' must be positive, not -1")
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "value")
})

test_that("a call R can't build keeps the wrapper's call", {
  e <- tryCatch(given_call_unbuildable(), error = function(e) e)
  expect_s3_class(e, "pkg_bad_table")
  expect_identical(conditionMessage(e), "the call is too long")
  expect_identical(conditionCall(e), quote(given_call_unbuildable()))
})

# endregion

# region: Deferred conditions and results

test_that("deferred conditions report the given call; a sibling keeps the wrapper's", {
  seen <- list()
  value <- withCallingHandlers(
    given_call_deferred(assign_call),
    warning = function(w) {
      seen[[length(seen) + 1L]] <<- w
      invokeRestart("muffleWarning")
    },
    message = function(m) {
      seen[[length(seen) + 1L]] <<- m
      invokeRestart("muffleMessage")
    }
  )
  expect_identical(value, 1L)
  expect_identical(
    vapply(seen, conditionMessage, ""),
    c("macro", "typed", "3 rows dropped", "noted\n", "sibling keeps its call")
  )
  for (w in seen[1:3]) {
    expect_identical(conditionCall(w), assign_call)
    expect_s3_class(w, "pkg_table_warning")
  }
  # A message has no call, whatever it was given.
  expect_null(conditionCall(seen[[4]]))
  expect_identical(conditionCall(seen[[5]]), quote(given_call_deferred(assign_call)))
})

test_that("a warning deferred from another thread reports the given call", {
  w <- NULL
  value <- withCallingHandlers(
    given_call_off_thread(assign_call),
    pkg_thread_warning = function(c) {
      w <<- c
      invokeRestart("muffleWarning")
    }
  )
  expect_identical(value, 1L)
  expect_identical(conditionMessage(w), "from a thread")
  expect_identical(conditionCall(w), assign_call)
})

test_that("RError::with_call() and #[condition(call = <expr>)] report the given call", {
  e <- tryCatch(given_call_result(assign_call), error = function(e) e)
  expect_identical(conditionCall(e), assign_call)
  expect_s3_class(e, "pkg_locked")
  expect_identical(e$kind, "result_err")

  e <- tryCatch(given_call_derived(assign_call, TRUE), error = function(e) e)
  expect_identical(conditionCall(e), assign_call)
  expect_identical(class(e)[1:2], c("pkg_table_error_replaced", "pkg_table_error"))
  expect_identical(conditionMessage(e), "the new table has no `id` column")
  expect_identical(e$column, "id")
  # The call travels as the call, not as a field.
  expect_false("site" %in% names(e))

  e <- tryCatch(given_call_derived(assign_call, FALSE), error = function(e) e)
  expect_identical(conditionCall(e), quote(given_call_derived(assign_call, FALSE)))
  expect_identical(class(e)[1], "pkg_table_error_missing")
})

# endregion

# region: The call type

test_that("RCallValue reads a call and builds the same call", {
  for (call in list(
    assign_call,
    quote(f()),
    quote(pkg::f(x[1, ], n = 2L, label = c("a", NA), g(TRUE, 1.5))),
    quote(`my fun`(...)),
    quote(function_name(a = NULL, b = 1i, c = as.raw(1)))
  )) {
    expect_identical(given_call_roundtrip(call), call)
  }
})

test_that("RCallValue refuses what is not a call it can read", {
  e <- tryCatch(given_call_roundtrip(1L), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_identical(e$kind, "conversion")
  expect_identical(e$param, "call")
  expect_match(conditionMessage(e), "'call'")
  expect_error(given_call_roundtrip(quote(x)), class = "rust_error")
  # A function object in the call, as `do.call()` builds, is no RValue.
  expect_error(given_call_roundtrip(as.call(list(sum, 1))), class = "rust_error")
  expect_error(given_call_roundtrip(as.call(list(quote(f), globalenv()))), class = "rust_error")
})

# endregion

test_that("given-call fixtures are clean under gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  gc_stress_given_call()
  gc_stress_condition_data()
  e <- tryCatch(given_call_error(assign_call), error = function(e) e)
  w <- tryCatch(given_call_warning(assign_call), warning = function(w) w)
  r <- given_call_roundtrip(quote(pkg::f(x[1, ], n = 2L, g(TRUE))))
  gctorture(FALSE)
  expect_identical(conditionCall(e), assign_call)
  expect_identical(conditionCall(w), assign_call)
  expect_identical(r, quote(pkg::f(x[1, ], n = 2L, g(TRUE))))
})
