+++
title = "Condition system: error!, warning!, message!, condition!"
weight = 50
description = "miniextendr provides four macros for raising structured R conditions from Rust. They all require errorinr mode — the default for every #[miniextendr] function."
+++

miniextendr provides four macros for raising structured R conditions from Rust.
They all require `error_in_r` mode — the default for every `#[miniextendr]`
function.

## Quick reference

| Macro | R equivalent | Default class | Unhandled behaviour |
|---|---|---|---|
| `error!(...)` | `stop()` | `rust_error` | terminates execution |
| `warning!(...)` | `warning()` | `rust_warning` | prints, continues |
| `message!(...)` | `message()` | `rust_message` | prints, continues |
| `condition!(...)` | `signalCondition()` | `rust_condition` | silent no-op |

All four support an optional `class = "name"` argument to prepend a custom class
for programmatic catching.

## How it works

Each macro calls `std::panic::panic_any(RCondition::...)`. The panic is caught by
`with_r_unwind_protect_error_in_r` before Rust destructors have unwound, which
recognises the `RCondition` payload and converts it to a tagged SEXP (4-element
list: `error`, `kind`, `class`, `call`). The generated R wrapper reads the SEXP
and dispatches to the appropriate R signal function.

The `class` slot carries the optional user-supplied class. When non-NULL it is
prepended to the standard layered vector.

## Class layering

```r
class(e)
# error!("...")         → c("rust_error",     "simpleError",     "error",   "condition")
# warning!("...")       → c("rust_warning",   "simpleWarning",   "warning", "condition")
# message!("...")       → c("rust_message",   "simpleMessage",   "message", "condition")
# condition!("...")     → c("rust_condition", "simpleCondition",            "condition")

# With class = "my_err":
class(e)
# error!(class = "my_err", "...") → c("my_err", "rust_error", "simpleError", "error", "condition")
```

## Runnable examples

### `error!()`

```r
library(miniextendr)

# Raised by: error!("something went wrong: {x}")

e <- tryCatch(demo_error("oops"), error = function(e) e)
class(e)
# [1] "rust_error"  "simpleError" "error"       "condition"
conditionMessage(e)
# [1] "oops"

# Specific handler:
tryCatch(demo_error("x"), rust_error = function(e) "caught by rust_error handler")
# [1] "caught by rust_error handler"
```

### `error!()` with custom class

```r
# Raised by: error!(class = "my_error", "missing field: {name}")

tryCatch(
  demo_error_custom_class("my_error", "missing field: x"),
  my_error   = function(e) paste("custom:", conditionMessage(e)),
  rust_error = function(e) paste("rust:",   conditionMessage(e))
)
# [1] "custom: missing field: x"
```

### `warning!()`

```r
# Raised by: warning!("x is large: {x}")

# tryCatch absorbs the warning and returns the handler result:
tryCatch(demo_warning("watch out"), rust_warning = function(w) "caught!")
# [1] "caught!"

# withCallingHandlers resumes execution after the handler:
result <- withCallingHandlers(
  {
    demo_warning("note")
    42L
  },
  warning = function(w) {
    cat("saw:", conditionMessage(w), "\n")
    invokeRestart("muffleWarning")
  }
)
# saw: note
result
# [1] 42
```

### `message!()`

```r
# Raised by: message!("step {n} complete")

demo_message("hello")
# hello

suppressMessages(demo_message("silenced"))
# (no output)

# withCallingHandlers — muffleMessage restart stops the default printing:
withCallingHandlers(
  demo_message("intercepted"),
  message = function(m) {
    cat("caught:", conditionMessage(m))
    invokeRestart("muffleMessage")
  }
)
# caught: intercepted
```

### `condition!()`

```r
# Raised by: condition!("step 1 of 10")
# Without a handler, signalCondition returns NULL invisibly.

demo_condition("silent signal")
# NULL

# With a handler:
withCallingHandlers(
  demo_condition("progress event"),
  condition = function(c) cat("progress:", conditionMessage(c), "\n")
)
# progress: progress event
# NULL

# With a custom class:
withCallingHandlers(
  demo_condition_custom_class("my_progress", "step 3"),
  my_progress = function(c) cat("progress:", conditionMessage(c), "\n")
)
# progress: step 3
# NULL
```

## Trait-ABI and ALTREP error class layering

Cross-package trait method panics and ALTREP `r_unwind` callback panics
**do** receive `rust_*` class layering, even though there is no R wrapper
to inspect a tagged SEXP. Two different mechanisms cover the two contexts:

- **Trait-ABI shims**: the vtable shim returns a tagged SEXP on panic; the
  generated View method wrapper inspects the result and re-panics with the
  reconstructed [`RCondition`]. The consumer's outer `error_in_r` guard
  (every `#[miniextendr]` fn has one) catches the re-panic and produces the
  tagged SEXP for the consumer's R wrapper. End-to-end behavior is identical
  to a same-package call: `tryCatch(rust_error = h, ...)` matches; user
  classes from `error!(class = "...", ...)` match before `rust_error`.

- **ALTREP `r_unwind` callbacks**: the guard raises the R condition by
  evaluating `stop(structure(list(message, call), class = c(...)))` directly
  (no R wrapper required). `tryCatch(rust_error = h, ...)` matches; user
  classes match before `rust_error`.

### Remaining limitations

Two narrow cases still degrade:

- `warning!()` / `message!()` / `condition!()` from an ALTREP `r_unwind`
  callback. There is no mechanism to suspend execution to deliver a
  non-fatal signal from inside R's vector-dispatch machinery. These produce
  an R error with the message: *"warning!/message!/condition! from ALTREP
  callback context cannot be raised as non-fatal signals; use error!()
  instead"*.

- A trait View method (`view.method()`) called from Rust code that is not
  wrapped in `with_r_unwind_protect_error_in_r` (e.g., a manual call from
  a test harness or init callback). The re-panic from the View has no
  outer guard to catch it, so the worker thread's `catch_unwind` boundary
  converts it to an R error without `rust_*` class layering. In practice,
  every `#[miniextendr]` fn already provides the outer guard, so this only
  affects unusual call sites.

Functions that explicitly opt out of `error_in_r` via
`#[miniextendr(no_error_in_r)]` or `unwrap_in_r` continue to use direct
`Rf_errorcall` — those modes exist precisely to bypass the condition
pipeline.

## `AsRError` — wrapping `std::error::Error`

For functions that return `Result<T, E>` where `E: std::error::Error`,
`AsRError<E>` wraps the error and formats its full cause chain into the
message:

```rust
use miniextendr_api::condition::AsRError;
use miniextendr_api::miniextendr;

#[miniextendr]
fn parse_number(s: &str) -> Result<i32, AsRError<std::num::ParseIntError>> {
    s.parse::<i32>().map_err(AsRError)
}
```

```r
tryCatch(parse_number("abc"), error = function(e) e$message)
# [1] "invalid digit found in string"
```

For errors with a source chain, all causes appear in the message separated by
`\n  caused by: ...`.
