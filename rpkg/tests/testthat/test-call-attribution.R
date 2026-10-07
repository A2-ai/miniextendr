# Tests for call attribution behaviour documented in docs/CALL_ATTRIBUTION.md.
# Both fixtures are internal (:::); this file makes the transcript machine-verifiable.

# region: call_attr_with (wrapped path — with call attribution)

test_that("call_attr_with produces rust_error with call attribution", {
  e <- tryCatch(
    miniextendr:::call_attr_with(1L, 2L),
    error = function(e) e
  )
  expect_s3_class(e, "rust_error")
  call_str <- deparse(conditionCall(e))
  expect_match(call_str, "call_attr_with")
})

test_that("call_attr_with conditionCall is the call as written", {
  # sys.call(): positional arguments stay positional.
  e <- tryCatch(
    miniextendr:::call_attr_with(1L, 2L),
    error = function(e) e
  )
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_with(1L, 2L)))
})

# endregion

# region: unsafe_C_call_attr_without (unwrapped path — no call attribution)

test_that("unsafe_C_call_attr_without produces simpleError without rust_error class", {
  e <- tryCatch(
    miniextendr:::unsafe_C_call_attr_without(1L, 2L),
    error = function(e) e
  )
  expect_false(inherits(e, "rust_error"))
  expect_s3_class(e, "simpleError")
})

test_that("wrapped and unwrapped paths differ in condition class", {
  # The wrapped path (call_attr_with) goes through the tagged-condition
  # transport with the wrapper's sys.call(); the unwrapped path (Rf_error) does
  # not. Both name the function as written.
  e_with <- tryCatch(
    miniextendr:::call_attr_with(1L, 2L),
    error = function(e) e
  )
  e_without <- tryCatch(
    miniextendr:::unsafe_C_call_attr_without(1L, 2L),
    error = function(e) e
  )
  # Only the wrapped path emits rust_error class
  expect_true(inherits(e_with, "rust_error"))
  expect_false(inherits(e_without, "rust_error"))
  expect_equal(conditionCall(e_with), quote(miniextendr:::call_attr_with(1L, 2L)))
  # Unwrapped path's conditionCall is non-NULL and names the function;
  # Rf_error captures the C-level call expression.
  expect_false(is.null(conditionCall(e_without)))
  call_without_str <- deparse(conditionCall(e_without))
  expect_match(call_without_str, "unsafe_C_call_attr_without")
})

# endregion

# region: export check

test_that("call attribution fixtures are not exported", {
  exports <- getNamespaceExports("miniextendr")
  expect_false("call_attr_with" %in% exports)
  expect_false("unsafe_C_call_attr_without" %in% exports)
})

# endregion

# region: call = caller (internal entry point behind a hand-written function, #1450)

test_that("call = caller attributes the condition to the hand-written caller", {
  e <- tryCatch(miniextendr:::call_attr_caller(-1L), error = function(e) e)
  expect_s3_class(e, "rust_error")
  # `Result<_, String>` renders the Err through Debug (quoted) by default.
  expect_match(conditionMessage(e), "x must be positive, got -1", fixed = TRUE)
  # The caller's call, as written.
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_caller(-1L)))
  expect_equal(miniextendr:::call_attr_caller(3L), 3L)
})

test_that("call = caller reports a `...` forwarder's call as written (#1462)", {
  # The caller's call is `miniextendr:::call_attr_caller(...)`, and that is
  # what the condition names, however the dots were filled.
  via_dots <- function(...) miniextendr:::call_attr_caller(...)
  expect_equal(via_dots(3L), 3L)
  e <- tryCatch(via_dots(0L), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_match(conditionMessage(e), "x must be positive, got 0", fixed = TRUE)
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_caller(...)))
  e <- tryCatch(via_dots(value = 0L), error = function(e) e)
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_caller(...)))
  e <- tryCatch(via_dots(-1L), error = function(e) e)
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_caller(...)))
})

test_that("call = caller reports lapply()'s `FUN(X[[i]], ...)` call (#1462)", {
  expect_equal(lapply(list(3L, 4L), miniextendr:::call_attr_caller), list(3L, 4L))
  e <- tryCatch(lapply(list(-1L), miniextendr:::call_attr_caller), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_equal(conditionCall(e), quote(FUN(X[[i]], ...)))
})

test_that("default noexport attribution still names the wrapper", {
  e <- tryCatch(miniextendr:::call_attr_self(-1L), error = function(e) e)
  expect_equal(conditionCall(e), quote(call_attr_self_impl(value)))
})

test_that("call = caller falls back to the wrapper's own call when called directly", {
  e <- tryCatch(miniextendr:::call_attr_caller_impl(-1L), error = function(e) e)
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_caller_impl(-1L)))
})

test_that("call = caller fixtures are not exported", {
  ns <- readLines(system.file("NAMESPACE", package = "miniextendr"))
  expect_false(any(grepl("call_attr_caller", ns)))
  expect_false(any(grepl("call_attr_self", ns)))
  expect_false(any(grepl("call_attr_checked", ns)))
})

# endregion

# region: call = caller covers the R-side checks too (#1548)

test_that("call = caller attributes R-side check failures to the caller (#1548)", {
  expect_equal(
    miniextendr:::call_attr_checked("Sa", c("mea", "sd"), 2L, "hi"),
    "Safe:mean+sd:2:high"
  )
  expect_equal(miniextendr:::call_attr_checked(), "Fast:mean:1:none")
  # Scalar `match_arg`: `base::match.arg()` would say 'arg' and report its own frame.
  e <- tryCatch(miniextendr:::call_attr_checked(mode = "bogus"), error = identity)
  expect_s3_class(e, "simpleError")
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_checked(mode = "bogus")))
  expect_equal(conditionMessage(e), "'mode' should be one of \"Fast\", \"Safe\", \"Debug\"")
  # `several_ok`: the strict helper raises with the caller's call.
  e <- tryCatch(miniextendr:::call_attr_checked(metrics = c("mean", "bogus")), error = identity)
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_checked(metrics = c("mean", "bogus"))))
  expect_match(conditionMessage(e), "'metrics' element 2 (\"bogus\") should be one of", fixed = TRUE)
  # Optional `choices`: NULL skips the check, anything else is validated.
  e <- tryCatch(miniextendr:::call_attr_checked(level = "bogus"), error = identity)
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_checked(level = "bogus")))
  expect_equal(conditionMessage(e), "'level' should be one of \"low\", \"high\"")
  # Typed precondition: the same message `stopifnot()` gave, the caller's call.
  e <- tryCatch(miniextendr:::call_attr_checked(n = 1.5), error = identity)
  expect_s3_class(e, "simpleError")
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_checked(n = 1.5)))
  expect_equal(conditionMessage(e), "'n' must be integer")
  e <- tryCatch(miniextendr:::call_attr_checked(n = 1:2), error = identity)
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_checked(n = 1:2)))
  expect_equal(conditionMessage(e), "'n' must have length 1")
})

test_that("call = caller R-side checks report a `...` forwarder's call as written", {
  via_dots <- function(...) miniextendr:::call_attr_checked(...)
  expect_equal(via_dots("Debug"), "Debug:mean:1:none")
  e <- tryCatch(via_dots(n = 1.5), error = identity)
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_checked(...)))
  expect_equal(conditionMessage(e), "'n' must be integer")
})

test_that("R-side checks fall back to the wrapper's own call when called directly", {
  e <- tryCatch(miniextendr:::call_attr_checked_impl(mode = "bogus"), error = identity)
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_checked_impl(mode = "bogus")))
  e <- tryCatch(miniextendr:::call_attr_checked_impl(n = 1.5), error = identity)
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_checked_impl(n = 1.5)))
})

test_that("default attribution of R-side checks is unchanged", {
  # `stopifnot()` reports the wrapper's frame, as before (#1548 touches only `call = caller`).
  e <- tryCatch(miniextendr:::call_attr_self_impl(x = 1.5), error = identity)
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_self_impl(x = 1.5)))
  expect_equal(conditionMessage(e), "'x' must be integer")
})

# endregion

# region: the three spellings of the attribution (#1566)

test_that("a `Call` marker hands the wrapper's own call as written to Rust", {
  # The marker is not an R formal.
  expect_equal(names(formals(miniextendr:::call_marker_wrapper_impl)), "x")
  expect_equal(
    miniextendr:::call_marker_wrapper_impl(1L),
    quote(miniextendr:::call_marker_wrapper_impl(1L))
  )
  # Behind a delegate it still names the bridge: `Call` is `wrapper` attribution.
  expect_equal(
    miniextendr:::call_marker_wrapper(2L),
    quote(call_marker_wrapper_impl(value))
  )
})

test_that("a `CallerCall` marker hands the caller's call as written to Rust", {
  # The marker is not an R formal; the `.call` of a `caller` wrapper is (#1613).
  expect_equal(names(formals(miniextendr:::call_marker_caller_impl)), c("x", ".call"))
  expect_equal(
    miniextendr:::call_marker_caller(2L),
    quote(miniextendr:::call_marker_caller(2L))
  )
  # Called directly, the wrapper's own call is the caller's call.
  expect_equal(
    miniextendr:::call_marker_caller_impl(3L),
    quote(miniextendr:::call_marker_caller_impl(3L))
  )
})

test_that("a `Call` marker leaves error attribution at the wrapper", {
  e <- tryCatch(miniextendr:::call_marker_checked_impl(-1L), error = function(e) e)
  expect_s3_class(e, "rust_error")
  expect_match(conditionMessage(e), "x must be positive, got -1", fixed = TRUE)
  expect_equal(conditionCall(e), quote(miniextendr:::call_marker_checked_impl(-1L)))
  expect_equal(miniextendr:::call_marker_checked_impl(4L), 4L)
})

test_that("marker fixtures are not exported", {
  ns <- readLines(system.file("NAMESPACE", package = "miniextendr"))
  expect_false(any(grepl("call_marker_", ns)))
})

# endregion

# region: a helper in between passes on the call to report (#1613)

test_that("a helper without `.call` is what a `caller` entry point reports", {
  e <- tryCatch(miniextendr:::call_attr_via_plain_helper(-1L), error = identity)
  expect_s3_class(e, "rust_error")
  expect_equal(conditionCall(e), quote(.call_attr_prepare_plain(value)))
})

test_that("a helper passing `call = parent.frame()` as `.call` names its caller", {
  e <- tryCatch(miniextendr:::call_attr_via_helper(-1L), error = identity)
  expect_s3_class(e, "rust_error")
  expect_match(conditionMessage(e), "x must be positive, got -1", fixed = TRUE)
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_via_helper(-1L)))
  expect_equal(miniextendr:::call_attr_via_helper(3L), 3L)
  # Through `lapply()` the caller is `FUN`, as without a helper.
  e <- tryCatch(lapply(list(-1L), miniextendr:::call_attr_via_helper), error = identity)
  expect_equal(conditionCall(e), quote(FUN(X[[i]], ...)))
})

test_that("two helpers threading `call` down name the outermost function", {
  e <- tryCatch(miniextendr:::call_attr_via_nested(-1L), error = identity)
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_via_nested(-1L)))
  expect_equal(miniextendr:::call_attr_via_nested(2L), 2L)
})

test_that("a frame passes through `do.call()` without re-running the caller", {
  e <- tryCatch(miniextendr:::call_attr_via_do_call(-1L), error = identity)
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_via_do_call(-1L)))
  runs <- 0L
  counted <- function(value) {
    runs <<- runs + 1L
    miniextendr:::.call_attr_prepare_do(value)
  }
  e <- tryCatch(counted(-1L), error = identity)
  expect_equal(conditionCall(e), quote(counted(-1L)))
  expect_equal(runs, 1L)
})

test_that("a helper's frame reaches the R-side checks", {
  e <- tryCatch(miniextendr:::call_attr_checked_via_helper(n = 1.5), error = identity)
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_checked_via_helper(n = 1.5)))
  expect_equal(conditionMessage(e), "'n' must be integer")
  expect_equal(miniextendr:::call_attr_checked_via_helper(4L), "Fast:mean:4:none")
})

test_that("a `CallerCall` body receives the call a helper passed on", {
  expect_equal(
    miniextendr:::call_marker_via_helper(2L),
    quote(miniextendr:::call_marker_via_helper(2L))
  )
})

test_that("a `Call` parameter on an S3 method receives the method's call", {
  obj <- structure(1, class = "mx_call_marker")
  # `UseMethod()` dispatch: the method frame's call names the method.
  expect_equal(format(obj), quote(format.mx_call_marker(obj)))
  expect_equal(format(obj, extra = 2), quote(format.mx_call_marker(obj, extra = 2)))
})

test_that("`.call` takes a call object as is", {
  e <- tryCatch(
    miniextendr:::call_attr_caller_impl(-1L, .call = quote(verb(v = 1))),
    error = identity
  )
  expect_equal(conditionCall(e), quote(verb(v = 1)))
})

test_that("`.call = environment()` names the function that passed it", {
  own <- function(value) miniextendr:::call_attr_caller_impl(value, .call = environment())
  e <- tryCatch(own(-1L), error = identity)
  expect_equal(conditionCall(e), quote(own(-1L)))
})

test_that("an environment that is no closure's frame counts as NULL", {
  # Called from the test block, NULL means the wrapper's own call, without `.call`.
  e <- tryCatch(
    miniextendr:::call_attr_caller_impl(-1L, .call = globalenv()),
    error = identity
  )
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_caller_impl(-1L)))
  # Called from a closure, NULL means that closure's call.
  via <- function(value) miniextendr:::call_attr_caller_impl(value, .call = globalenv())
  e <- tryCatch(via(-1L), error = identity)
  expect_equal(conditionCall(e), quote(via(-1L)))
})

test_that("anything else passed as `.call` is an argument error on `.call`", {
  # A positional argument too many lands in `.call` on a wrapper without `...`.
  e <- tryCatch(miniextendr:::call_attr_caller_impl(-1L, 2), error = identity)
  expect_s3_class(e, "rust_error")
  expect_equal(e$param, ".call")
  expect_equal(conditionMessage(e), "'.call' must be NULL, an environment or a call")
  expect_equal(conditionCall(e), quote(miniextendr:::call_attr_caller_impl(-1L, 2)))
})

test_that("`.call` leaves an omitted `Missing<T>` argument missing", {
  expect_equal(miniextendr:::call_attr_omitted_impl(.call = quote(f())), "absent")
})

test_that("on a `caller` wrapper with `...`, `.call` follows the dots", {
  expect_equal(names(formals(miniextendr:::call_attr_dots_impl)), c("x", "...", ".call"))
  # Positional extras go to the dots; `.call` stays NULL.
  expect_equal(miniextendr:::call_attr_dots_impl(1L, 2, 3), 3L)
  via <- function(v) miniextendr:::call_attr_dots_impl(v, .call = environment())
  e <- tryCatch(via(-1L), error = identity)
  expect_equal(conditionCall(e), quote(via(-1L)))
})

test_that("`caller` wrappers take `.call`; a default one does not", {
  expect_equal(names(formals(miniextendr:::call_attr_caller_impl)), c("x", ".call"))
  expect_equal(names(formals(miniextendr:::call_attr_internal_impl)), c("x", ".call"))
  expect_equal(names(formals(miniextendr:::call_attr_self_impl)), "x")
  expect_equal(miniextendr:::call_attr_internal_impl(5L), 5L)
})

test_that("an `internal` `caller` wrapper documents `.call`", {
  rd_db <- tryCatch(tools::Rd_db("miniextendr"), error = function(e) NULL)
  skip_if(is.null(rd_db), "tools::Rd_db('miniextendr') unavailable — package not installed")
  # A free function documents its own page (#1289).
  expect_true("call_attr_internal_impl.Rd" %in% names(rd_db))
  rd_text <- paste(capture.output(print(rd_db[["call_attr_internal_impl.Rd"]])), collapse = "\n")
  expect_match(rd_text, "call_attr_internal_impl(x, .call = NULL)", fixed = TRUE)
  expect_match(rd_text, "\\item{.call}{", fixed = TRUE)
})

# endregion

# region: one form: the R-side checks and Rust report the same call

test_that("a positional call reports the same form from the R guard and from Rust", {
  # R guard: `num` has length 2. Rust conversion: "BLQ" is not a number.
  guard <- tryCatch(miniextendr:::arg_error_ratio(c(1, 2), 3), error = identity)
  rust <- tryCatch(miniextendr:::arg_error_peak(c("1", "BLQ")), error = identity)
  expect_null(guard$rust_type)
  expect_identical(rust$rust_type, "AsNumericVec")
  expect_equal(conditionCall(guard), quote(miniextendr:::arg_error_ratio(c(1, 2), 3)))
  expect_equal(conditionCall(rust), quote(miniextendr:::arg_error_peak(c("1", "BLQ"))))
})

test_that("a `...` forwarder reports the same form from the R guard and from Rust", {
  via_ratio <- function(...) miniextendr:::arg_error_ratio(...)
  via_peak <- function(...) miniextendr:::arg_error_peak(...)
  guard <- tryCatch(via_ratio(c(1, 2), den = 3), error = identity)
  rust <- tryCatch(via_peak(dv = c("1", "BLQ")), error = identity)
  expect_equal(conditionCall(guard), quote(miniextendr:::arg_error_ratio(...)))
  expect_equal(conditionCall(rust), quote(miniextendr:::arg_error_peak(...)))
})

test_that("an R6 method reports the same form from the R guard and from Rust", {
  counter <- getNamespace("miniextendr")$FastCounter$new(0L)
  # R guard: not an integer. Rust conversion: `NA_integer_` is no `i32`.
  guard <- tryCatch(counter$add("x"), error = identity)
  rust <- tryCatch(counter$add(NA_integer_), error = identity)
  expect_null(guard$rust_type)
  expect_identical(rust$rust_type, "i32")
  expect_equal(conditionCall(guard), quote(counter$add("x")))
  expect_equal(conditionCall(rust), quote(counter$add(NA_integer_)))
})

test_that("a `caller` entry point behind a helper reports the public call from both sides", {
  guard <- tryCatch(miniextendr:::call_attr_checked_via_helper(1.5), error = identity)
  rust <- tryCatch(miniextendr:::call_attr_via_helper(-1L), error = identity)
  expect_equal(conditionMessage(guard), "'n' must be integer")
  expect_match(conditionMessage(rust), "x must be positive, got -1", fixed = TRUE)
  expect_equal(conditionCall(guard), quote(miniextendr:::call_attr_checked_via_helper(1.5)))
  expect_equal(conditionCall(rust), quote(miniextendr:::call_attr_via_helper(-1L)))
})

test_that("a marker body sees the call the R guard reports", {
  guard <- tryCatch(miniextendr:::call_marker_caller_impl(1.5), error = identity)
  expect_equal(conditionCall(guard), quote(miniextendr:::call_marker_caller_impl(1.5)))
  expect_equal(
    miniextendr:::call_marker_caller_impl(3L),
    quote(miniextendr:::call_marker_caller_impl(3L))
  )
})

# endregion
