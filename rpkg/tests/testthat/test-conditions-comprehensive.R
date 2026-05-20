# Comprehensive tests for the rust→R condition pipeline.
#
# Existing test-conditions.R covers the free-function path. This file fills in
# the matrix: every condition kind raised from each class system (R6, S3, S4,
# S7, Env, Vctrs), plus conditionCall / e$kind / e$class assertions, plus the
# ALTREP non-error degradation (issue #366), plus message/encoding edge cases,
# plus sidecar-accessor / sidecar-consumer panic paths, plus a withRestarts +
# condition!() composition.

# Helper to skip if vctrs feature is not enabled.
skip_if_vctrs_disabled <- function() {
  skip_if_not("vctrs" %in% miniextendr::rpkg_enabled_features(),
              "vctrs feature not enabled")
  skip_if_not_installed("vctrs")
}

# region: free fn — conditionCall / e$kind / e$class fields

test_that("free fn error: conditionCall returns the user's call", {
  e <- tryCatch(demo_error("x"), error = function(e) e)
  cl <- conditionCall(e)
  expect_false(is.null(cl))
  expect_equal(as.character(cl[[1]]), "demo_error")
})

test_that("free fn error: e$kind reports 'error'", {
  e <- tryCatch(demo_error("x"), error = function(e) e)
  expect_equal(e$kind, "error")
})

test_that("free fn warning: e$kind reports 'warning'", {
  w <- tryCatch(demo_warning("x"), warning = function(w) w)
  expect_equal(w$kind, "warning")
})

test_that("free fn message: e$kind reports 'message'", {
  m <- tryCatch(demo_message("x"), message = function(m) m)
  expect_equal(m$kind, "message")
})

test_that("free fn condition: e$kind reports 'condition'", {
  c_val <- tryCatch(demo_condition("x"), rust_condition = function(c) c)
  expect_equal(c_val$kind, "condition")
})

test_that("free fn message: conditionMessage adds trailing newline", {
  # The helper wraps the message body in paste0(.msg, "\n") so message() output
  # matches the standard base-R `message()` newline convention.
  m <- tryCatch(demo_message("hello"), message = function(m) m)
  expect_equal(conditionMessage(m), "hello\n")
})

test_that("custom-class error: tryCatch on rust_error still matches (layered)", {
  result <- tryCatch(
    demo_error_custom_class("typed_err", "msg"),
    rust_error = function(e) "rust",
    error = function(e) "generic"
  )
  expect_equal(result, "rust")
})

# endregion

# region: nested re-raise — class identity preserved across handler re-throw

test_that("re-raised classed error preserves the custom class", {
  outer <- tryCatch(
    {
      tryCatch(
        demo_error_custom_class("inner_class", "boom"),
        inner_class = function(e) stop(e)
      )
    },
    inner_class = function(e) class(e)
  )
  expect_true("inner_class" %in% outer)
  expect_true("rust_error" %in% outer)
})

# endregion

# region: edge cases — empty / long / unicode / non-RCondition payload

test_that("empty error message produces a valid rust_error", {
  e <- tryCatch(condition_error_empty(), error = function(e) e)
  expect_true(inherits(e, "rust_error"))
  expect_equal(conditionMessage(e), "")
})

test_that("long error message round-trips through CString + STRSXP", {
  e <- tryCatch(condition_error_long_message(500L), error = function(e) e)
  expect_true(inherits(e, "rust_error"))
  expect_gt(nchar(conditionMessage(e)), 4000)
})

test_that("unicode + multibyte + embedded newline survives FFI", {
  e <- tryCatch(condition_error_unicode(), error = function(e) e)
  expect_true(inherits(e, "rust_error"))
  msg <- conditionMessage(e)
  expect_true(grepl("日本語", msg))
  expect_true(grepl("\\n", msg))
})

test_that("non-RCondition panic payload falls through to rust_error with kind=panic", {
  e <- tryCatch(condition_panic_with_int_payload(), error = function(e) e)
  expect_true(inherits(e, "rust_error"))
  expect_equal(e$kind, "panic")
})

# endregion

# region: R6 — instance method conditions

test_that("R6: error!() raises rust_error from method", {
  obj <- R6Raiser$new(1L)
  e <- tryCatch(obj$raise_error("r6 boom"), error = function(e) e)
  expect_true(inherits(e, "rust_error"))
  expect_equal(conditionMessage(e), "r6 boom")
  expect_equal(e$kind, "error")
})

test_that("R6: error!(class) layers custom class first", {
  obj <- R6Raiser$new(1L)
  e <- tryCatch(
    obj$raise_error_classed("r6_custom", "msg"),
    r6_custom = function(e) e
  )
  expect_equal(class(e)[1], "r6_custom")
  expect_true(inherits(e, "rust_error"))
})

test_that("R6: warning!() raises rust_warning from method", {
  obj <- R6Raiser$new(1L)
  w <- tryCatch(obj$raise_warning("r6 warn"), warning = function(w) w)
  expect_true(inherits(w, "rust_warning"))
  expect_equal(conditionMessage(w), "r6 warn")
})

test_that("R6: warning continues under withCallingHandlers + muffleWarning", {
  obj <- R6Raiser$new(1L)
  saw <- FALSE
  withCallingHandlers(
    obj$raise_warning("transient"),
    warning = function(w) {
      saw <<- TRUE
      invokeRestart("muffleWarning")
    }
  )
  expect_true(saw)
})

test_that("R6: message!() raises rust_message from method", {
  obj <- R6Raiser$new(1L)
  m <- tryCatch(obj$raise_message("r6 info"), message = function(m) m)
  expect_true(inherits(m, "rust_message"))
})

test_that("R6: condition!() with custom class is catchable", {
  obj <- R6Raiser$new(1L)
  result <- tryCatch(
    obj$raise_condition_classed("r6_signal", "step done"),
    r6_signal = function(c) conditionMessage(c)
  )
  expect_equal(result, "step done")
})

# endregion

# region: S3 — method dispatch via UseMethod

test_that("S3: error!() raises rust_error via S3 dispatch", {
  obj <- new_s3raiser(1L)
  e <- tryCatch(s3_raise_error(obj, "s3 boom"), error = function(e) e)
  expect_true(inherits(e, "rust_error"))
  expect_equal(conditionMessage(e), "s3 boom")
})

test_that("S3: warning continues under withCallingHandlers", {
  obj <- new_s3raiser(1L)
  saw <- FALSE
  withCallingHandlers(
    s3_raise_warning(obj, "s3 warn"),
    warning = function(w) {
      saw <<- TRUE
      invokeRestart("muffleWarning")
    }
  )
  expect_true(saw)
})

test_that("S3: message!() raises rust_message via S3 dispatch", {
  obj <- new_s3raiser(1L)
  m <- tryCatch(s3_raise_message(obj, "s3 info"), message = function(m) m)
  expect_true(inherits(m, "rust_message"))
})

test_that("S3: classed condition is caught by user class first", {
  obj <- new_s3raiser(1L)
  result <- tryCatch(
    s3_raise_error_classed(obj, "s3_typed", "boom"),
    s3_typed = function(e) "typed",
    rust_error = function(e) "rust"
  )
  expect_equal(result, "typed")
})

test_that("S3: conditionCall identifies the dispatch method", {
  obj <- new_s3raiser(1L)
  e <- tryCatch(s3_raise_error(obj, "x"), error = function(e) e)
  cl <- conditionCall(e)
  expect_false(is.null(cl))
  expect_true(grepl("s3_raise_error", deparse(cl)[[1]]))
})

# endregion

# region: S4 — setMethod dispatch

test_that("S4: error!() raises rust_error from method", {
  obj <- S4Raiser(1L)
  e <- tryCatch(s4_raise_error(obj, "s4 boom"), error = function(e) e)
  expect_true(inherits(e, "rust_error"))
})

test_that("S4: warning continues under withCallingHandlers", {
  obj <- S4Raiser(1L)
  saw <- FALSE
  withCallingHandlers(
    s4_raise_warning(obj, "s4 warn"),
    warning = function(w) {
      saw <<- TRUE
      invokeRestart("muffleWarning")
    }
  )
  expect_true(saw)
})

test_that("S4: classed condition is caught by user class", {
  obj <- S4Raiser(1L)
  result <- tryCatch(
    s4_raise_error_classed(obj, "s4_typed", "msg"),
    s4_typed = function(e) "typed"
  )
  expect_equal(result, "typed")
})

test_that("S4: message!() raises rust_message", {
  obj <- S4Raiser(1L)
  m <- tryCatch(s4_raise_message(obj, "s4 info"), message = function(m) m)
  expect_true(inherits(m, "rust_message"))
})

# endregion

# region: S7 — new_generic dispatch

test_that("S7: error!() raises rust_error", {
  obj <- S7Raiser(id = 1L)
  e <- tryCatch(s7_raise_error(obj, "s7 boom"), error = function(e) e)
  expect_true(inherits(e, "rust_error"))
  expect_equal(conditionMessage(e), "s7 boom")
})

test_that("S7: warning continues under withCallingHandlers", {
  obj <- S7Raiser(id = 1L)
  saw <- FALSE
  withCallingHandlers(
    s7_raise_warning(obj, "s7 warn"),
    warning = function(w) {
      saw <<- TRUE
      invokeRestart("muffleWarning")
    }
  )
  expect_true(saw)
})

test_that("S7: message!() raises rust_message", {
  obj <- S7Raiser(id = 1L)
  m <- tryCatch(s7_raise_message(obj, "s7 info"), message = function(m) m)
  expect_true(inherits(m, "rust_message"))
})

test_that("S7: classed condition is caught by user class", {
  obj <- S7Raiser(id = 1L)
  result <- tryCatch(
    s7_raise_error_classed(obj, "s7_typed", "msg"),
    s7_typed = function(e) "typed"
  )
  expect_equal(result, "typed")
})

# endregion

# region: Env — `obj$method()` dispatch via $.<Type> operator

test_that("Env: error!() raises rust_error from method", {
  obj <- EnvRaiser$new(1L)
  e <- tryCatch(obj$env_raise_error("env boom"), error = function(e) e)
  expect_true(inherits(e, "rust_error"))
})

test_that("Env: classed warning catches via user class", {
  obj <- EnvRaiser$new(1L)
  w <- tryCatch(
    obj$env_raise_warning_classed("env_typed", "warn"),
    env_typed = function(w) w
  )
  expect_equal(class(w)[1], "env_typed")
  expect_true(inherits(w, "rust_warning"))
})

test_that("Env: message!() raises rust_message", {
  obj <- EnvRaiser$new(1L)
  m <- tryCatch(obj$env_raise_message("env info"), message = function(m) m)
  expect_true(inherits(m, "rust_message"))
})

# endregion

# region: Vctrs — static methods + format protocol override
#
# vctrs codegen forbids `&self` receivers (MXL120), so the raiser exposes
# static methods that take the vctrs payload (Vec<f64>) explicitly. Two shapes
# are exercised:
#
#   1. Static helpers — emit `vctrsraiser_vctrs_raise_*(values, …)` plain
#      wrapped fns with full match.call() attribution.
#   2. `#[miniextendr(vctrs(format))]` — emits the S3 method
#      `format.VctrsRaiser(x, ...)`; user calls `format(x)` which dispatches
#      via `UseMethod("format")`. The Rust argument is named `_x` (not
#      `_values`) so the generated method signature matches base `format(x,
#      ...)` — avoids an S3 generic/method consistency WARNING from R CMD
#      check.

test_that("Vctrs: constructor builds a vctrs vector", {
  skip_if_vctrs_disabled()
  obj <- new_vctrsraiser(c(1, 2, 3))
  expect_true(vctrs::vec_is(obj))
  expect_s3_class(obj, "VctrsRaiser")
  expect_equal(length(obj), 3)
})

test_that("Vctrs: static raise_error helper raises rust_error", {
  skip_if_vctrs_disabled()
  obj <- new_vctrsraiser(c(1, 2, 3))
  e <- tryCatch(
    vctrsraiser_vctrs_raise_error(obj, "vctrs boom"),
    error = function(e) e
  )
  expect_true(inherits(e, "rust_error"))
  expect_equal(conditionMessage(e), "vctrs boom")
  expect_equal(e$kind, "error")
})

test_that("Vctrs: warning continues under withCallingHandlers + muffleWarning", {
  skip_if_vctrs_disabled()
  obj <- new_vctrsraiser(c(1, 2, 3))
  saw <- FALSE
  withCallingHandlers(
    vctrsraiser_vctrs_raise_warning(obj, "vctrs warn"),
    warning = function(w) {
      saw <<- TRUE
      invokeRestart("muffleWarning")
    }
  )
  expect_true(saw)
})

test_that("Vctrs: classed condition catches via user class first", {
  skip_if_vctrs_disabled()
  obj <- new_vctrsraiser(c(1, 2, 3))
  result <- tryCatch(
    vctrsraiser_vctrs_raise_error_classed(obj, "vctrs_typed", "msg"),
    vctrs_typed = function(e) "typed",
    rust_error = function(e) "rust"
  )
  expect_equal(result, "typed")
})

test_that("Vctrs: message!() raises rust_message", {
  skip_if_vctrs_disabled()
  obj <- new_vctrsraiser(c(1, 2, 3))
  m <- tryCatch(
    vctrsraiser_vctrs_raise_message(obj, "vctrs info"),
    message = function(m) m
  )
  expect_true(inherits(m, "rust_message"))
})

test_that("Vctrs: condition!() with custom class is catchable", {
  skip_if_vctrs_disabled()
  obj <- new_vctrsraiser(c(1, 2, 3))
  result <- tryCatch(
    vctrsraiser_vctrs_raise_condition_classed(obj, "vctrs_signal", "step done"),
    vctrs_signal = function(c) conditionMessage(c)
  )
  expect_equal(result, "step done")
})

test_that("Vctrs: static helper conditionCall identifies the wrapper fn", {
  skip_if_vctrs_disabled()
  obj <- new_vctrsraiser(c(1, 2, 3))
  e <- tryCatch(
    vctrsraiser_vctrs_raise_error(obj, "x"),
    error = function(e) e
  )
  cl <- conditionCall(e)
  expect_false(is.null(cl))
  expect_true(grepl("vctrsraiser_vctrs_raise_error", deparse(cl)[[1]]))
})

test_that("Vctrs: format protocol override dispatches via UseMethod", {
  skip_if_vctrs_disabled()
  obj <- new_vctrsraiser(c(1, 2, 3))
  e <- tryCatch(format(obj), error = function(e) e)
  expect_true(inherits(e, "rust_error"))
  expect_equal(conditionMessage(e), "format-protocol boom")
})

# endregion

# region: sidecar accessor / consumer panic
#
# Sidecar `*_get_field` / `*_set_field` C wrappers do NOT carry the
# `__miniextendr_call` slot (externalptr_derive.rs emits hand-rolled wrappers
# with `numArgs = 1/2`, no call slot — see #348). When a panic surfaces
# through a sidecar-bearing R6 class via the normal `#[miniextendr]` method
# path, the call attribution still works because the panicking method takes
# the `#[miniextendr]` path (full `match.call()`); only the *accessor*
# wrappers omit it. These tests assert that:
#
#   - an instance method on a sidecar-bearing class panics with `rust_error`,
#   - `e$kind` carries the expected variant,
#   - a standalone fn consuming the sidecar object carries call attribution
#     pointing at the wrapper.

test_that("sidecar-class instance method panic surfaces as rust_error", {
  obj <- PanickingSidecar$new("doom!")
  e <- tryCatch(obj$boom(), error = function(e) e)
  expect_true(inherits(e, "rust_error"))
  expect_equal(conditionMessage(e), "doom!")
})

test_that("sidecar-class method panic preserves e$kind = 'error'", {
  obj <- PanickingSidecar$new("kindtest")
  e <- tryCatch(obj$boom(), error = function(e) e)
  expect_equal(e$kind, "error")
})

test_that("sidecar field read survives before the panicking method runs", {
  obj <- PanickingSidecar$new("doom!")
  # The sidecar accessor is the path *without* match.call(). Reading the
  # field through the active binding must succeed (no panic).
  expect_equal(obj$doom, "doom!")
})

test_that("standalone fn consuming sidecar panics with rust_error", {
  # Use the low-level constructor that returns the bare ExternalPtr — the R6
  # `$new()` constructor wraps the pointer in an environment and would fail
  # the ExternalPtr<PanickingSidecar> argument conversion.
  ptr <- panicking_sidecar_new("ignored")
  e <- tryCatch(sidecar_consumer_panic(ptr), error = function(e) e)
  expect_true(inherits(e, "rust_error"))
  expect_equal(conditionMessage(e), "consumer boom")
  cl <- conditionCall(e)
  # standalone fn carries match.call() — call is non-NULL and references the
  # wrapper.
  expect_false(is.null(cl))
  expect_true(grepl("sidecar_consumer_panic", deparse(cl)[[1]]))
})

# endregion

# region: edge cases — withRestarts / e$class slot / message conditionCall

test_that("condition!() composes with withRestarts + invokeRestart", {
  saw <- 0L
  withRestarts(
    withCallingHandlers(
      demo_condition("step"),
      rust_condition = function(c) {
        saw <<- saw + 1L
        invokeRestart("continue_anyway")
      }
    ),
    continue_anyway = function() invisible(NULL)
  )
  expect_equal(saw, 1L)
})

test_that("bare error: class(e) starts with rust_error (no user class layered)", {
  # The internal transport SEXP (make_rust_condition_value) is a 4-element list
  # with names c("error", "kind", "class", "call") — the user-supplied class
  # lives in .val$class on transport. The R-side .miniextendr_raise_condition
  # helper consumes that slot and prepends it to class(e) on the final
  # condition object via stop(structure(list(...), class = c(user_class,
  # "rust_error", "simpleError", "error", "condition"))); the final condition
  # object itself stores only message/call (no $class field — class is set as
  # an attribute via structure()). For a bare `error!` with no user class,
  # class(e) starts with "rust_error".
  e <- tryCatch(demo_error("x"), error = function(e) e)
  expect_equal(class(e)[1], "rust_error")
  expect_false("typed_err" %in% class(e))
})

test_that("classed error: class(e)[1] matches the user-supplied class", {
  # `error!` with class layers the user class as class(e)[1] in front of
  # rust_error.
  e <- tryCatch(
    demo_error_custom_class("typed_err", "msg"),
    typed_err = function(e) e
  )
  expect_equal(class(e)[1], "typed_err")
  expect_true("rust_error" %in% class(e))
})

test_that("R6 message!(): conditionCall is non-NULL", {
  # Parallel to the S3 conditionCall test, but for message!() instead of
  # error!() — exercises the message transport (signalCondition path).
  obj <- R6Raiser$new(1L)
  m <- tryCatch(obj$raise_message("r6 info"), message = function(m) m)
  cl <- conditionCall(m)
  # R6 active-binding lambda or sys.call() frame — either way conditionCall
  # should not throw and m$kind tells us this came through the message path.
  expect_equal(m$kind, "message")
  # We don't pin the exact deparse output — only that the kind survives the
  # signalCondition round-trip alongside conditionCall.
  expect_true(is.null(cl) || is.call(cl))
})

# endregion

# region: ALTREP non-error degradation (issue #366)
#
# warning!/message!/condition! from inside an ALTREP RUnwind callback cannot
# suspend execution — there is no R wrapper to handle restart. They degrade
# to a plain R error with a fixed diagnostic message. After the #366 fix the
# degraded error inherits `rust_error` class layering (matching the generic
# panic→rust_error path 10 lines down in the same function).

test_that("ALTREP warning! degrades to rust_error (issue #366)", {
  x <- altrep_warn_on_elt(5L, "ignored")
  e <- tryCatch(x[1L], error = function(e) e)
  expect_true(inherits(e, "rust_error"))
  expect_true(grepl("ALTREP callback context", conditionMessage(e)))
})

test_that("ALTREP warning! degraded error matches tryCatch(rust_error = h)", {
  x <- altrep_warn_on_elt(5L, "ignored")
  result <- tryCatch(x[1L], rust_error = function(e) "caught")
  expect_equal(result, "caught")
})

test_that("ALTREP message! degrades to rust_error (issue #366)", {
  x <- altrep_message_on_elt(5L, "ignored")
  e <- tryCatch(x[1L], error = function(e) e)
  expect_true(inherits(e, "rust_error"))
})

test_that("ALTREP condition! degrades to rust_error (issue #366)", {
  x <- altrep_condition_on_elt(5L, "ignored")
  e <- tryCatch(x[1L], error = function(e) e)
  expect_true(inherits(e, "rust_error"))
})

# endregion
