# Condition system: error!, warning!, message!, condition!

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

## Limitations in non-error_in_r mode

If a function opts out of `error_in_r` (via `#[miniextendr(no_error_in_r)]`,
`unwrap_in_r`, or is a trait-ABI shim / ALTREP callback), the condition pipeline
is unavailable because there is no R wrapper to inspect the tagged SEXP.

In that context:

- `error!()` degrades to `Rf_errorcall` (a plain R error, `rust_*` class lost).
- `warning!()`, `message!()`, `condition!()` produce an R error with the message:
  *"warning!/message!/condition! require error_in_r mode (the default); this
  function opted out via no_error_in_r/unwrap_in_r or is a trait-ABI shim /
  ALTREP callback"*.

This is a known limitation. Routing trait-ABI / ALTREP signals through the
tagged-SEXP path to recover `rust_*` class layering is tracked in #345.

## `RErrorAdapter` — wrapping `std::error::Error`

For functions that return `Result<T, E>` where `E: std::error::Error`,
`RErrorAdapter<E>` wraps the error and formats its full cause chain into the
message:

```rust
use miniextendr_api::condition::RErrorAdapter;
use miniextendr_api::miniextendr;

#[miniextendr]
fn parse_number(s: &str) -> Result<i32, RErrorAdapter<std::num::ParseIntError>> {
    s.parse::<i32>().map_err(RErrorAdapter)
}
```

```r
tryCatch(parse_number("abc"), error = function(e) e$message)
# [1] "invalid digit found in string"
```

For errors with a source chain, all causes appear in the message separated by
`\n  caused by: ...`.
