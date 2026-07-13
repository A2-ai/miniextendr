# Condition system: error!, warning!, message!, condition!

miniextendr provides four macros for raising structured R conditions from Rust.
They ride the tagged-condition transport that every `#[miniextendr]` function
uses.

## Quick reference

| Macro | R equivalent | Default class | Unhandled behaviour |
|---|---|---|---|
| `error!(...)` | `stop()` | `rust_error` | terminates execution |
| `warning!(...)` | `warning()` | `rust_warning` | prints, continues |
| `message!(...)` | `message()` | `rust_message` | prints, continues |
| `condition!(...)` | `signalCondition()` | `rust_condition` | silent no-op |

All four support an optional `class = "name"` argument to prepend a custom class
for programmatic catching, and an optional `data = ...` argument to attach
structured named fields readable as `e$<name>` in handlers.

> **Import note.** `error!` and `condition!` are shadowed by the crate-root
> modules `error` / `condition`, so `use miniextendr_api::*;` (or a direct
> `use miniextendr_api::error;`) resolves to the module, not the macro. Use the
> collision-free aliases `rust_error!` / `rust_condition!` (identical
> expansions), or invoke the bare names fully qualified. `warning!` / `message!`
> have no such clash.

## How it works

Each macro calls `std::panic::panic_any(RCondition::...)`. The panic is caught by
`with_r_unwind_protect` before Rust destructors have unwound, which recognises
the `RCondition` payload and converts it to a tagged SEXP (5-element list:
`error`, `kind`, `class`, `call`, `data`). The generated R wrapper reads the
SEXP and dispatches to the appropriate R signal function.

The `class` slot carries the optional user-supplied class. When non-NULL it is
prepended to the standard layered vector. The `data` slot carries the optional
named-list payload; the R helper splices its fields into the condition object
alongside `message` / `call` / `kind`.

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

### `error!()` with structured `data` payloads

Rust-side, the macros accept `data = ("name", value)` for a single field or
`data = [("a", v1), ("b", v2)]` for several (rlang `abort(data = list(...))`
style). Argument order is fixed: `class = ...` (optional), then `data = ...`
(optional), then the format message:

```rust
// Single field:
error!(class = "range_error", data = ("value", value), "value {value} out of range");

// Multiple fields:
error!(
    class = "validation_error",
    data = [("value", value), ("code", code), ("label", label), ("fatal", true)],
    "validation failed for {label}"
);
```

R-side, handlers read the fields directly from the condition object:

```r
# Raised by: error!(class = "range_error", data = ("value", value), "value {value} out of range")

e <- tryCatch(demo_error_data_scalar(150L), range_error = function(e) e)
e$value
# [1] 150

# Programmatic recovery — clamp instead of parsing the message:
tryCatch(
  demo_error_data_scalar(150L),
  range_error = function(e) min(max(e$value, 0L), 100L)
)
# [1] 100
```

#### Supported `data` value types

| Rust value | R field type |
|---|---|
| `i32` | `integer(1)` |
| `f64` | `double(1)` |
| `bool` | `logical(1)` |
| `&str` / `String` | `character(1)` |
| `Vec<i32>` | `integer(n)` |
| `Vec<f64>` | `double(n)` |
| `Vec<bool>` | `logical(n)` |
| `Vec<String>` / `Vec<&str>` | `character(n)` |
| `Option<T>` / `Vec<Option<T>>` for the scalar families above | typed `NA` |
| `i64` / `u32` | integer when it fits; otherwise double |
| `RValue::Null` | `NULL` |
| `RValue::Complex`, `RValue::Raw` | complex / raw vector |
| `RValue::List` | recursively nested, optionally named list |

`RValue` is the owned, `Send`, R-native value tree used by the condition
transport. Build it directly for nested or heterogeneous values, or use
`RValue::debug(value)` to attach an eager `Debug` rendering when no native R
mapping is appropriate.

#### Worker-thread note

The payload travels through `panic_any`, which requires `Send` — and the macro
may fire on the worker thread, where a live `SEXP` is illegal to carry. Each
field is therefore converted at the call site into `RValue`, and a multi-field
payload becomes `ConditionData` (`Vec<(String, RValue)>`). The actual R objects
are materialised on R's main thread at the unwind boundary. Consequently,
`data = ...` works identically from worker-thread and main-thread code, but a
live `SEXP` or arbitrary `IntoR` value cannot ride along.

Reserved names: fields named `message`, `call`, or `kind` would override the
condition's own slots (the R helper splices via `utils::modifyList`) — avoid
them.

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
  reconstructed [`RCondition`]. The consumer's outer `with_r_unwind_protect`
  guard (every `#[miniextendr]` fn has one) catches the re-panic and produces
  the tagged SEXP for the consumer's R wrapper. End-to-end behavior is identical
  to a same-package call: `tryCatch(rust_error = h, ...)` matches; user
  classes from `error!(class = "...", ...)` match before `rust_error`. Structured
  `data =` fields (tagged SEXP slot [4]) are also preserved across the re-panic
  boundary — `e$field_name` is accessible in R handlers even when the error
  crossed a package boundary (see #996 path-1).

- **ALTREP `r_unwind` callbacks**: the guard raises the R condition by
  evaluating `stop(structure(list(message, call, ...), class = c(...)))`
  directly (no R wrapper required). `tryCatch(rust_error = h, ...)` matches;
  user classes match before `rust_error`. Structured `data =` fields are
  spliced into the condition list after `message`/`call`, so `e$field_name`
  works from ALTREP-raised errors too (see #996 path-2).

### Remaining limitations

Two narrow cases still degrade:

- `warning!()` / `message!()` / `condition!()` from an ALTREP `r_unwind`
  callback. There is no mechanism to suspend execution to deliver a
  non-fatal signal from inside R's vector-dispatch machinery. These produce
  an R error with the message: *"warning!/message!/condition! from ALTREP
  callback context cannot be raised as non-fatal signals; use error!()
  instead"*.

- A trait View method (`view.method()`) called from Rust code that is not
  wrapped in `with_r_unwind_protect` (e.g., a manual call from a test harness
  or init callback). The re-panic from the View has no outer guard to catch it,
  so the worker thread's `catch_unwind` boundary converts it to an R error
  without `rust_*` class layering. In practice, every `#[miniextendr]` fn
  already provides the outer guard, so this only affects unusual call sites.

Functions that explicitly opt out via `#[miniextendr(unwrap_in_r)]` deliver
`Result<T, E>` to R as a list with an `$error` slot rather than treating `Err`
as a Rust-origin failure — `Err` never traverses the condition pipeline.

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
