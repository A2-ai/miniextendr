# Hand-written delegates for the `call = caller` fixture pair in
# src/rust/call_attribution_demo.rs (docs/CALL_ATTRIBUTION.md, #1450). Both are
# package-internal; tests reach them through `:::`.

# Delegates to a `#[miniextendr(noexport, call = caller)]` entry point: a Rust
# error surfaces as `Error in call_attr_caller(value = -1L)`.
call_attr_caller <- function(value) {
  call_attr_caller_impl(value)
}

# Delegates to a default `noexport` entry point: the same error surfaces as
# `Error in call_attr_self_impl(x = value)`, naming the bridge.
call_attr_self <- function(value) {
  call_attr_self_impl(value)
}

# Delegates to a `call = caller` entry point with R-side checks (#1548): a bad
# choice or a failed precondition surfaces as `Error in call_attr_checked(...)`
# naming the argument, the same way a Rust-side error does.
call_attr_checked <- function(mode = "Fast", metrics = "mean", n = 1L, level = NULL) {
  call_attr_checked_impl(mode, metrics, n, level)
}

# Delegates to a `#[miniextendr(noexport)]` entry point taking a `CallerCall`
# marker (#1566): the marker spelling of `call = caller`. The Rust body returns
# the call it was handed, so the test can compare it with the caller's call.
call_marker_caller <- function(value) {
  call_marker_caller_impl(value)
}

# Delegates to an entry point taking a `Call` marker: the wrapper's own
# `match.call()` reaches Rust, naming the bridge with its formals matched.
call_marker_wrapper <- function(value) {
  call_marker_wrapper_impl(value)
}

# Delegates to a `call = caller` entry point whose argument converts Rust-side
# (a `TryFromSexp` newtype with a classed error, src/rust/classed_result_tests.rs):
# the conversion error surfaces as `Error in hyperparams_total_caller(...)`.
hyperparams_total_caller <- function(hyper) {
  hyperparams_total_caller_impl(hyper)
}

# Delegates to a `call = caller` entry point with per-parameter `inherits` /
# `no_na` checks (src/rust/param_check_tests.rs): a failed check surfaces as
# `Error in param_checks_caller(...)`, like the type checks above.
param_checks_caller <- function(x, y) {
  param_checks_caller_impl(x, y)
}

# The same with the author's messages on both checks: a custom message keeps
# the caller's call.
param_checks_caller_msg <- function(x, y) {
  param_checks_caller_msg_impl(x, y)
}

# Delegate to `call = caller` entry points taking `AsNumeric` / `AsNumericVec`
# (src/rust/argument_error_tests.rs, #1591): an argument error surfaces as
# `Error in arg_error_ratio_caller(...)` / `arg_error_peak_caller(...)` whether
# the R-side length check or the Rust conversion catches it.
arg_error_ratio_caller <- function(num, den) {
  arg_error_ratio_caller_impl(num, den)
}

arg_error_peak_caller <- function(dv) {
  arg_error_peak_caller_impl(dv)
}
