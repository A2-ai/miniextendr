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
