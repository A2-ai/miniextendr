# The raising guard (`with_r_unwind_protect_or_raise`, and the ALTREP `RUnwind`
# callbacks) raises the condition the generated wrapper raises for the same
# payload (#1768): it hands the tagged value to the wrapper's own
# `.miniextendr_raise_condition`, so the classes, `kind`, the fields and their
# order and the call all match. Fixture: `transport_condition()` in
# src/rust/unwind_protect_tests.rs, which raises one payload either way, given
# the wrapper's call.

# Both sides raise from the same call expression, so the condition calls
# compare equal too.
both_transports <- function(what) {
  raise <- function(guard) tryCatch(transport_condition(what, guard), error = function(e) e)
  list(wrapper = raise(FALSE), guard = raise(TRUE))
}

layers <- c("rust_error", "simpleError", "error", "condition")

test_that("error!: the guard raises the wrapper's condition, kind included", {
  e <- both_transports("error")
  expect_identical(e$guard, e$wrapper)
  expect_identical(class(e$guard), c("pkg_transport", layers))
  expect_identical(names(unclass(e$guard)), c("message", "call", "kind", "step"))
  expect_identical(e$guard$kind, "error")
  expect_identical(e$guard$step, 1L)
  expect_identical(conditionMessage(e$guard), "transport error")
  expect_equal(conditionCall(e$guard), quote(transport_condition(what, guard)))
})

test_that("arg_error!: kind = 'conversion' and e$param through the guard", {
  e <- both_transports("arg")
  expect_identical(e$guard, e$wrapper)
  expect_identical(class(e$guard), layers)
  expect_identical(e$guard$kind, "conversion")
  expect_identical(e$guard$param, "what")
  expect_identical(conditionMessage(e$guard), "'what' is not a choice")
})

test_that("a panic: kind = 'panic' and the same located message", {
  e <- both_transports("panic")
  expect_identical(e$guard, e$wrapper)
  expect_identical(class(e$guard), layers)
  expect_identical(e$guard$kind, "panic")
  expect_match(conditionMessage(e$guard), "transport panic", fixed = TRUE)
  expect_match(conditionMessage(e$guard), "\\(at .*unwind_protect_tests\\.rs:[0-9]+\\)")
})

test_that("data fields named kind and message splice the same way on both (#1315)", {
  # The macros reject these names; a hand-built payload reaches the transports
  # with them. Both splice `data` with `utils::modifyList`, so the fields
  # replace the base slots in place and the order stays message, call, kind.
  e <- both_transports("reserved")
  expect_identical(e$guard, e$wrapper)
  expect_identical(class(e$guard), c("pkg_transport", layers))
  expect_identical(names(unclass(e$guard)), c("message", "call", "kind", "step"))
  expect_identical(conditionMessage(e$guard), "user message")
  expect_identical(e$guard$kind, "user kind")
  expect_identical(e$guard$step, 2L)
})

test_that("warning! inside the guard degrades to a kind = 'panic' error", {
  # A non-fatal signal cannot resume a callback the panic already left.
  e <- tryCatch(transport_condition("warning", TRUE), error = function(e) e)
  expect_identical(class(e), layers)
  expect_identical(e$kind, "panic")
  expect_match(conditionMessage(e), "cannot be raised as non-fatal signals", fixed = TRUE)
  expect_equal(conditionCall(e), quote(transport_condition("warning", TRUE)))
})

test_that("no condition: the guard returns the value", {
  expect_identical(transport_condition("none", TRUE), 0L)
  expect_identical(transport_condition("none", FALSE), 0L)
})

test_that("the guard's conditions survive gctorture", {
  skip_gc_stress_if_disabled()
  gctorture(TRUE)
  on.exit(gctorture(FALSE), add = TRUE)
  ok <- 0L
  for (i in seq_len(5L)) {
    e <- both_transports("error")
    r <- both_transports("reserved")
    if (identical(e$guard, e$wrapper) && identical(e$guard$step, 1L) &&
        identical(r$guard, r$wrapper) && identical(r$guard$kind, "user kind")) {
      ok <- ok + 1L
    }
  }
  gctorture(FALSE)
  expect_identical(ok, 5L)
})
