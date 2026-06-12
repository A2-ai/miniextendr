# Tests for `#[miniextendr(error_direct)]` (issue #665).
#
# `error_direct` raises error-shaped failures (panic!() / error!() /
# RCondition::Error) DIRECTLY from C via Rf_eval(stop(structure(...))), skipping
# the R-side `.miniextendr_raise_condition` re-raise. The C-built condition
# object MUST carry the same `rust_*` class layering the R-side path produces, so
# a tryCatch over an error_direct fixture sees identical classes to the default
# (tagged-SEXP) path.
#
# Non-error signals (warning!/message!) fall back to the R-side raise, since
# stop() only does errors.
#
# Soundness note (the reason this knob is safe): by the time the direct stop()
# longjmps, with_r_unwind_protect_error_direct has already caught the Rust panic
# (catch_unwind unwound every Drop value in the user closure) and dropped the
# payload box, so NO Rust destructor is skipped by the longjmp. See
# miniextendr-api/src/unwind_protect.rs `with_r_unwind_protect_error_direct`.

# region: class layering — must match the R-side .miniextendr_raise_condition

test_that("error_direct: rust_error class layering matches the R-side path", {
  e <- tryCatch(demo_error_direct("boom"), error = function(e) e)
  expect_equal(
    class(e),
    c("rust_error", "simpleError", "error", "condition")
  )
})

test_that("error_direct and the indirect (R-side) path produce identical classes", {
  e_direct <- tryCatch(demo_error_direct("boom"), error = function(e) e)
  e_indirect <- tryCatch(demo_error_indirect("boom"), error = function(e) e)
  expect_equal(class(e_direct), class(e_indirect))
})

test_that("error_direct: panic!() layers as rust_error (kind=panic arm)", {
  e <- tryCatch(demo_panic_direct("kaboom"), error = function(e) e)
  expect_equal(
    class(e),
    c("rust_error", "simpleError", "error", "condition")
  )
})

test_that("error_direct: custom class is prepended before rust_error layering", {
  e <- tryCatch(
    demo_error_direct_custom_class("typed_err", "msg"),
    error = function(e) e
  )
  expect_equal(
    class(e),
    c("typed_err", "rust_error", "simpleError", "error", "condition")
  )
})

# endregion

# region: tryCatch by class still dispatches correctly

test_that("error_direct: tryCatch(rust_error=) matches", {
  result <- tryCatch(
    demo_error_direct("boom"),
    rust_error = function(e) "rust",
    error = function(e) "generic"
  )
  expect_equal(result, "rust")
})

test_that("error_direct: tryCatch on the custom class matches first", {
  result <- tryCatch(
    demo_error_direct_custom_class("typed_err", "msg"),
    typed_err = function(e) "typed",
    rust_error = function(e) "rust"
  )
  expect_equal(result, "typed")
})

# endregion

# region: message round-trips through CString + STRSXP + C-built condition

test_that("error_direct: conditionMessage round-trips", {
  e <- tryCatch(demo_error_direct("the exact message"), error = function(e) e)
  expect_equal(conditionMessage(e), "the exact message")
})

test_that("error_direct: empty message produces a valid rust_error", {
  e <- tryCatch(demo_error_direct(""), error = function(e) e)
  expect_true(inherits(e, "rust_error"))
  expect_equal(conditionMessage(e), "")
})

test_that("error_direct: unicode + embedded newline survives the C raise", {
  e <- tryCatch(demo_error_direct("日本語\nsecond line"), error = function(e) e)
  expect_true(inherits(e, "rust_error"))
  msg <- conditionMessage(e)
  expect_true(grepl("日本語", msg))
  expect_true(grepl("\\n", msg))
})

# endregion

# region: conditionCall — the user's call is attached

test_that("error_direct: conditionCall carries the wrapper call", {
  e <- tryCatch(demo_error_direct("x"), error = function(e) e)
  cl <- conditionCall(e)
  expect_false(is.null(cl))
  expect_equal(as.character(cl[[1]]), "demo_error_direct")
})

# endregion

# region: documented difference vs the R-side path

test_that("error_direct: the C-built condition omits the $kind field (scope note)", {
  # The R-side .miniextendr_raise_condition attaches kind = "error"; the direct
  # C path builds list(message, call) with class set and no kind slot. This is a
  # deliberate, documented difference — tryCatch dispatch (by class) and
  # conditionMessage()/conditionCall() are unaffected. Tracked in the #665 PR.
  e <- tryCatch(demo_error_direct("x"), error = function(e) e)
  expect_null(e$kind)
})

# endregion

# region: non-error signals fall back to the R-side raise

test_that("error_direct: warning!() still signals as a warning (fallback path)", {
  w <- tryCatch(demo_warning_direct("warn"), warning = function(w) w)
  expect_true(inherits(w, "rust_warning"))
  expect_equal(conditionMessage(w), "warn")
})

test_that("error_direct: message!() still emits a message (fallback path)", {
  m <- tryCatch(demo_message_direct("msg"), message = function(m) m)
  expect_true(inherits(m, "rust_message"))
  expect_equal(conditionMessage(m), "msg\n")
})

# endregion

# region: error carrying data = falls back to the tagged-SEXP path

# The direct C-side stop(structure(...)) raise has no slot for condition data, so
# an error!() that carries `data =` falls back to the tagged-SEXP path (the same
# one used without error_direct). These tests prove error_direct does NOT silently
# drop the data payload — a regression that was latent in the original PR.

test_that("error_direct: data-carrying error preserves e$value (fallback path)", {
  e <- tryCatch(demo_error_direct_with_data(42L), error = function(e) e)
  expect_true(inherits(e, "range_error"))
  expect_equal(e$value, 42L)
  expect_equal(
    class(e),
    c("range_error", "rust_error", "simpleError", "error", "condition")
  )
})

test_that("error_direct data error matches the indirect data path", {
  e_direct <- tryCatch(demo_error_direct_with_data(7L), error = function(e) e)
  e_indirect <- tryCatch(demo_error_data_scalar(7L), error = function(e) e)
  expect_equal(class(e_direct), class(e_indirect))
  expect_equal(e_direct$value, e_indirect$value)
})

# endregion

# region: no-arg fixture (gctorture sweep target)

test_that("error_direct: no-arg fixture raises a rust_error", {
  e <- tryCatch(demo_error_direct_fixed(), error = function(e) e)
  expect_true(inherits(e, "rust_error"))
  expect_equal(conditionMessage(e), "error_direct fixed message")
})

# endregion
