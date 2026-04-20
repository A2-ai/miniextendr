# Error Handling in miniextendr

This guide covers error handling patterns, panic safety, and best practices for robust R-Rust interop.

## Overview

miniextendr handles three types of errors:

1. **Rust panics** → Converted to R errors
2. **R errors** (`Rf_error`) → Rust destructors run, then R unwinds
3. **Result errors** → Can be returned as R values or converted to R errors

---

## Panics

### Basic Panic Handling

Rust panics in `#[miniextendr]` functions are automatically caught and converted to R errors:

```rust
#[miniextendr]
pub fn divide(a: i32, b: i32) -> i32 {
    if b == 0 {
        panic!("Division by zero!");
    }
    a / b
}
```

```r
divide(10L, 0L)
# Error: Division by zero!
```

### Panic with Destructors

RAII resources are properly cleaned up when panics occur:

```rust
#[miniextendr]
pub fn process_file(path: &str) -> i32 {
    let _file = File::open(path)?;  // RAII - closed on drop
    let _lock = mutex.lock();        // RAII - released on drop

    if some_condition {
        panic!("Something went wrong!");
        // _file and _lock are STILL dropped before R sees the error
    }
    42
}
```

### How It Works

miniextendr uses `R_UnwindProtect` to ensure proper cleanup:

```text
┌─────────────────────────────────────────────────────────┐
│  #[miniextendr] function called from R                  │
├─────────────────────────────────────────────────────────┤
│  1. Enter R_UnwindProtect context                       │
│  2. Create Rust resources (RAII)                        │
│  3. Execute user code                                   │
│     ├── Success: return result                          │
│     ├── Panic: catch_unwind catches it                  │
│     │   └── Drop all Rust resources                     │
│     │   └── Convert panic message to R error            │
│     └── R error (longjmp): cleanup handler triggers     │
│         └── Drop all Rust resources                     │
│         └── Continue R's unwind                         │
└─────────────────────────────────────────────────────────┘
```

---

## Result Types

### Automatic Error Conversion

Return `Result<T, E>` where `E: Display` for automatic error conversion:

```rust
#[miniextendr]
pub fn parse_number(s: &str) -> Result<i32, String> {
    s.parse().map_err(|e| format!("Parse error: {}", e))
}
```

```r
parse_number("42")    # 42
parse_number("abc")   # Error: Parse error: invalid digit found in string
```

### Custom Error Types

Any type implementing `Display` works:

```rust
use std::fmt;

#[derive(Debug)]
pub enum MyError {
    InvalidInput(String),
    OutOfRange { min: i32, max: i32, got: i32 },
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::InvalidInput(s) => write!(f, "Invalid input: {}", s),
            MyError::OutOfRange { min, max, got } =>
                write!(f, "Value {} out of range [{}, {}]", got, min, max),
        }
    }
}

#[miniextendr]
pub fn validate(x: i32) -> Result<i32, MyError> {
    if x < 0 || x > 100 {
        Err(MyError::OutOfRange { min: 0, max: 100, got: x })
    } else {
        Ok(x)
    }
}
```

### Using `anyhow` or `thiserror`

For complex error handling, use standard error crates:

```rust
// With thiserror
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error at line {line}: {message}")]
    Parse { line: usize, message: String },
}

#[miniextendr]
pub fn load_data(path: &str) -> Result<Vec<f64>, DataError> {
    let contents = std::fs::read_to_string(path)?;  // Auto-converts io::Error
    // ...
}
```

```rust
// With anyhow
use anyhow::{Context, Result};

#[miniextendr]
pub fn process(path: &str) -> Result<i32> {
    let data = std::fs::read_to_string(path)
        .context("Failed to read input file")?;
    // ...
}
```

---

## R Error API

### panic!() for Errors

Use `panic!()` for unrecoverable errors. The `#[miniextendr]` framework catches panics
and converts them to structured R error conditions via `error_in_r`:

```rust
#[miniextendr]
pub fn validate_positive(x: i32) -> i32 {
    if x < 0 {
        panic!("x must be non-negative, got {}", x);
    }
    x
}
```

For recoverable errors, return `Result<T, E>`:

```rust
#[miniextendr]
pub fn check_input(x: i32) -> Result<i32, String> {
    if x < 0 {
        return Err("x must be positive".to_string());
    }
    Ok(x)
}
```

> **Note**: `r_stop()` exists internally but is not part of the public API.
> The `r_error!` macro has been removed. Always use `panic!()` or `Err(...)` instead.

### Warnings

Issue warnings without stopping execution:

```rust
use miniextendr_api::error::r_warning;

#[miniextendr]
pub fn risky_operation(x: i32) -> i32 {
    if x > 1000 {
        r_warning("Large value may cause performance issues");
    }
    expensive_computation(x)
}
```

### Console Output

Print to R's console (not stderr):

```rust
use miniextendr_api::error::{r_print, r_println};

#[miniextendr]
pub fn verbose_function() {
    r_println("Starting computation...");
    // ... work ...
    r_println("Done!");
}
```

---

## Error-in-R Mode (`error_in_r`)

By default, Rust-origin failures (panics, `Result::Err`, `Option::None`) are transported as
**tagged SEXP values** back to R, and the generated R wrapper inspects the value and raises a
structured R condition. This ensures all Rust destructors have fully completed before R sees
the error, and gives R code a typed condition class to catch.

This is the `error_in_r` mode, which is **enabled by default** for all `#[miniextendr]`
functions and methods. Opt out with `no_error_in_r` if you need the legacy behavior where
`Rf_error`/`Rf_errorcall` raises the R error immediately (while Rust stack frames are still
active).

### Opting Out

Per-function:

```rust
// Opt out: use legacy Rf_error behavior
#[miniextendr(no_error_in_r)]
pub fn fast_path(x: i32) -> i32 { x }
```

Per-method on an impl block:

```rust
#[miniextendr]
impl MyType {
    #[miniextendr(no_error_in_r)]
    fn legacy_method(&self) -> i32 { 42 }
}
```

### What Gets Caught

`error_in_r` intercepts three failure modes:

| Failure | Error kind | Example |
|---------|-----------|---------|
| Rust panic | `"panic"` | `panic!("oops")` |
| `Result::Err` | `"result_err"` | `Err("bad input".to_string())` |
| `Option::None` | `"none_err"` | Returning `None` from `-> Option<T>` |

**Not affected**: `Result<T, ()>` (unit error type) always returns `NULL` on `Err(())` regardless
of `error_in_r`, since a unit error is a deliberate sentinel, not a failure.

### The Error Value (Rust Side)

When a failure occurs, the generated C wrapper calls `make_rust_error_value(message, kind)` from
`miniextendr_api::error_value`. This builds a tagged SEXP -- a named list with:

- `$error`: the error message (character scalar)
- `$kind`: machine-readable kind (`"panic"`, `"result_err"`, `"none_err"`)
- class attribute: `"rust_error_value"`
- `__rust_error__` attribute: `TRUE`

This value is returned as a normal SEXP from the `.Call()` interface. No R error is raised
at this point -- all Rust stack frames have already unwound.

### The R Wrapper (R Side)

The generated R wrapper checks the return value:

```r
my_function <- function(x) {
  .val <- .Call(C_my_function, x, .call = match.call())
  if (inherits(.val, "rust_error_value") && isTRUE(attr(.val, "__rust_error__"))) {
    stop(structure(
      class = c("rust_error", "simpleError", "error", "condition"),
      list(message = .val$error, call = sys.call(), kind = .val$kind)
    ))
  }
  .val
}
```

The R condition has class `c("rust_error", "simpleError", "error", "condition")`, so it
can be caught at any level of the hierarchy.

### Catching Errors in R

Catch by the specific `rust_error` class:

```r
tryCatch(
  parse_data("not a number"),
  rust_error = function(e) {
    message("Rust error (", e$kind, "): ", e$message)
  }
)
```

Or with `tryCatch`/`withCallingHandlers` using the generic `error` class:

```r
tryCatch(
  parse_data("bad"),
  error = function(e) {
    if (inherits(e, "rust_error")) {
      # Structured Rust error with e$kind
    } else {
      # Other R error
    }
  }
)
```

The condition object has these fields:

| Field | Type | Description |
|-------|------|-------------|
| `message` | character | Human-readable error message |
| `call` | call | The R call that triggered the error |
| `kind` | character | `"panic"`, `"result_err"`, or `"none_err"` |

### Works with All Class Systems

`error_in_r` can be applied to methods in any class system:

```rust
#[miniextendr]            // env class (default)
impl Counter {
    #[miniextendr(error_in_r)]
    fn get(&self) -> i32 { self.value }
}

#[miniextendr(r6)]        // R6 class
impl Widget {
    #[miniextendr(error_in_r)]
    pub fn name(&self) -> String { self.name.clone() }
}

#[miniextendr(s7)]        // S7 class
impl Gauge {
    #[miniextendr(error_in_r)]
    pub fn read(&self) -> f64 { self.level }
}
```

It also works on trait impl methods:

```rust
#[miniextendr]
impl Fallible for MyType {
    #[miniextendr(error_in_r)]
    fn get_value(&self) -> i32 { self.value }
}
```

### Object Survival After Errors

Objects remain valid after an `error_in_r` error. Since the error is transported as a value
(not a longjmp), the object's internal state is never corrupted:

```r
counter <- Counter$new()
counter$inc()

# Error does not corrupt the object
tryCatch(counter$panic_method(), error = function(e) NULL)

counter$get()  # Still returns 1
```

### error_in_r vs unwrap_in_r

`error_in_r` and `unwrap_in_r` are mutually exclusive. Both transport errors past the Rust
boundary, but differ in the R-side representation:

| | `error_in_r` | `unwrap_in_r` |
|---|---|---|
| R-side error | Structured condition (`rust_error` class) | Plain `stop()` message |
| Catchable by class | Yes (`tryCatch(..., rust_error = ...)`) | No (generic `error` only) |
| Error kind field | `e$kind` available | Not available |
| Use case | Libraries that need typed error handling | Simple scripts |

---

## Panic Hook and Backtraces

### The Panic Hook

miniextendr installs a custom panic hook at package load time via `miniextendr_panic_hook()`,
which is called from `R_init_<pkg>()` in the C entrypoint. The hook is registered once using
`std::sync::Once` and controls whether Rust backtraces are printed to stderr on panic.

By default, **backtraces are suppressed**. Panic messages are still captured and forwarded to
R as error messages, but the noisy default Rust backtrace output is hidden. This keeps R
console output clean for end users.

### Enabling Backtraces

Set the `MINIEXTENDR_BACKTRACE` environment variable to see full Rust backtraces on panic:

```r
Sys.setenv(MINIEXTENDR_BACKTRACE = "1")
my_function_that_panics()
# thread 'main' panicked at 'something went wrong', src/lib.rs:42:5
# stack backtrace:
#    0: std::panicking::begin_panic
#    1: mypackage::my_function
#    ...
```

Accepted values: `"1"` or `"true"` (case-insensitive). Any other value or unset = backtraces
suppressed.

To disable again:

```r
Sys.unsetenv("MINIEXTENDR_BACKTRACE")
```

This variable is read at panic time, not at hook registration time, so it can be toggled
during a session without restarting R.

### How It Works

The hook replaces Rust's default panic hook with a wrapper that checks `MINIEXTENDR_BACKTRACE`
on every panic:

```default
Panic occurs
  -> Custom hook runs
  -> Reads MINIEXTENDR_BACKTRACE env var
  -> If "true" or "1": delegates to default hook (prints backtrace to stderr)
  -> Otherwise: suppresses output (panic message still captured for R error)
```

The panic message itself is always captured by `catch_unwind` in the generated C wrapper and
forwarded to R -- the hook only controls whether the backtrace is additionally printed to
stderr.

### Poisoning Recovery

If the panic hook installation itself panics (e.g., `take_hook` fails), `Once` marks the
state as poisoned. On the next call, the hook retries via `call_once_force` and prints a
warning:

```default
warning: miniextendr panic hook is retrying after a previous failed attempt
```

This is a defensive measure -- in practice, hook installation should not fail.

---

## Thread Safety

### Main Thread Requirement

R API calls must happen on R's main thread. miniextendr enforces this in debug builds:

```rust
#[miniextendr]
pub fn my_function() {
    // In debug builds, this panics if not on main thread
    r_println("Hello from R!");
}
```

### Thread Detection

Explicitly check thread context:

```rust
use miniextendr_api::worker::is_r_main_thread;

pub fn internal_function() {
    if is_r_main_thread() {
        r_println("On main thread");
    } else {
        eprintln!("On worker thread");
    }
}
```

---

## Type Conversion Errors

### SexpTypeError

When R passes wrong types:

```rust
#[miniextendr]
pub fn needs_integer(x: i32) -> i32 {
    x * 2
}
```

```r
needs_integer("abc")
# Error: failed to convert parameter 'x' to i32: type mismatch: expected INTSXP, got STRSXP
```

### NA Handling

Non-`Option` types reject NA values:

```rust
#[miniextendr]
pub fn needs_value(x: i32) -> i32 { x }

#[miniextendr]
pub fn handles_na(x: Option<i32>) -> i32 {
    x.unwrap_or(-1)
}
```

```r
needs_value(NA_integer_)  # Error: contains NA
handles_na(NA_integer_)   # -1
```

### Coercion Errors

When coercion fails:

```rust
#[miniextendr(coerce)]
pub fn needs_int(x: i32) -> i32 { x }
```

```r
needs_int(1.5)   # Error: failed to coerce to i32: fractional value
needs_int(1e20)  # Error: failed to coerce to i32: overflow
```

---

## Best Practices

### 1. Prefer Result Over Panic

```rust
// Good: Clear error handling
#[miniextendr]
pub fn parse_config(json: &str) -> Result<Config, String> {
    serde_json::from_str(json)
        .map_err(|e| format!("Invalid JSON: {}", e))
}

// Avoid: Panics hide the problem
#[miniextendr]
pub fn parse_config(json: &str) -> Config {
    serde_json::from_str(json).unwrap()  // Panic message is cryptic
}
```

### 2. Use Option for NA-Safe Operations

```rust
// Good: Explicit NA handling
#[miniextendr]
pub fn safe_divide(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    match (a, b) {
        (Some(x), Some(y)) if y != 0.0 => Some(x / y),
        _ => None,  // Returns NA
    }
}

// Avoid: Will panic on NA
#[miniextendr]
pub fn unsafe_divide(a: f64, b: f64) -> f64 {
    a / b
}
```

### 3. Validate Early

```rust
#[miniextendr]
pub fn process_matrix(data: Vec<f64>, rows: i32, cols: i32) -> Vec<f64> {
    // Validate at entry point
    if rows <= 0 || cols <= 0 {
        panic!("rows and cols must be positive");
    }
    let expected_len = (rows * cols) as usize;
    if data.len() != expected_len {
        panic!("data length {} doesn't match {}x{}", data.len(), rows, cols);
    }

    // Now process with confidence
    compute(data, rows as usize, cols as usize)
}
```

### 4. Context in Error Messages

```rust
#[miniextendr]
pub fn load_model(path: &str) -> Result<Model, String> {
    let bytes = std::fs::read(path)
        .map_err(|e| format!("Failed to read '{}': {}", path, e))?;

    Model::from_bytes(&bytes)
        .map_err(|e| format!("Invalid model format in '{}': {}", path, e))
}
```

### 5. Don't Swallow Errors

```rust
// Bad: Silent failure
#[miniextendr]
pub fn maybe_compute(x: i32) -> i32 {
    risky_operation(x).unwrap_or(0)  // Error silently becomes 0
}

// Good: Return Option to signal failure
#[miniextendr]
pub fn maybe_compute(x: i32) -> Option<i32> {
    risky_operation(x).ok()  // Caller sees NA on failure
}

// Also good: Explicit error
#[miniextendr]
pub fn compute(x: i32) -> Result<i32, String> {
    risky_operation(x).map_err(|e| e.to_string())
}
```

### 6. Document Error Conditions

```rust
/// Compute factorial.
///
/// @param n Non-negative integer (max 20 to avoid overflow)
/// @return n! as integer
/// @examples
/// factorial(5L)   # 120
/// factorial(-1L)  # Error
#[miniextendr]
pub fn factorial(n: i32) -> Result<i64, String> {
    if n < 0 {
        return Err("n must be non-negative".into());
    }
    if n > 20 {
        return Err("n > 20 would overflow i64".into());
    }
    Ok((1..=n as i64).product())
}
```

---

## Error Recovery Patterns

### Fallback Values

```rust
#[miniextendr]
pub fn safe_log(x: f64) -> f64 {
    if x <= 0.0 {
        f64::NEG_INFINITY  // Mathematical convention
    } else {
        x.ln()
    }
}
```

### Partial Results

```rust
#[miniextendr]
pub fn parse_numbers(strings: Vec<String>) -> Vec<Option<f64>> {
    strings.iter()
        .map(|s| s.parse().ok())  // Failed parses become None (NA in R)
        .collect()
}
```

### Cleanup on Error

```rust
#[miniextendr]
pub fn transactional_write(path: &str, data: &[u8]) -> Result<(), String> {
    let temp_path = format!("{}.tmp", path);

    // Write to temp file first
    std::fs::write(&temp_path, data)
        .map_err(|e| format!("Write failed: {}", e))?;

    // Atomic rename - if this fails, temp file is left for debugging
    std::fs::rename(&temp_path, path)
        .map_err(|e| {
            // Clean up temp file on rename failure
            let _ = std::fs::remove_file(&temp_path);
            format!("Rename failed: {}", e)
        })
}
```

---

## Debugging Tips

### Enable Backtraces

See [Panic Hook and Backtraces](#panic-hook-and-backtraces) above for full details.

```r
Sys.setenv(MINIEXTENDR_BACKTRACE = "1")
my_function_that_panics()
```

### Nested Panic Location

Panics preserve the original location:

```rust
fn inner_function() {
    panic!("Inner error");  // This location is reported
}

fn middle_function() {
    inner_function();
}

#[miniextendr]
pub fn outer_function() {
    middle_function();  // Error shows inner_function location
}
```

### Print Debugging

Use R's console for visibility:

```rust
#[miniextendr]
pub fn debug_function(x: i32) -> i32 {
    r_println(&format!("Input: {}", x));
    let result = complex_computation(x);
    r_println(&format!("Result: {}", result));
    result
}
```

Or Rust's stderr (visible in R console):

```rust
#[miniextendr]
pub fn debug_function(x: i32) -> i32 {
    eprintln!("[Rust] Input: {}", x);
    // ...
}
```

---

## Common Pitfalls

### 1. Panicking in Drop

Avoid panicking in destructors - it can cause double-panic abort:

```rust
// Bad
impl Drop for MyResource {
    fn drop(&mut self) {
        if self.is_invalid() {
            panic!("Resource in bad state!");  // Don't do this
        }
    }
}

// Good
impl Drop for MyResource {
    fn drop(&mut self) {
        if self.is_invalid() {
            eprintln!("Warning: dropping invalid resource");
        }
    }
}
```

### 2. Calling R from Wrong Thread

```rust
// Bad: Crashes or causes undefined behavior
std::thread::spawn(|| {
    r_println("Hello!");  // R API call from wrong thread
});

// Good: Use worker thread or avoid R API
std::thread::spawn(|| {
    eprintln!("Hello!");  // Rust-only output is safe
});
```

### 3. Ignoring Result

```rust
// Bad: Error silently ignored
#[miniextendr]
pub fn risky() {
    let _ = fallible_operation();  // What if this failed?
}

// Good: Handle or propagate
#[miniextendr]
pub fn risky() -> Result<(), String> {
    fallible_operation()?;
    Ok(())
}
```

---

## Known Limitations

- **Spawned-thread panics** cannot be cleanly propagated through `extern "C-unwind"` boundaries. Convert thread errors to `Result` instead of using `resume_unwind`. See [GAPS.md](GAPS.md#56-thread-panic-propagation-limitation).
- **Thread safety debug assertions** for SEXP access only run in debug builds. Checked FFI wrappers provide runtime thread checks in all build modes. See [GAPS.md](GAPS.md#55-thread-safety-debug-assertions).

See [GAPS.md](GAPS.md) for the full catalog of known limitations.

---

## See Also

- [FEATURE_DEFAULTS.md](FEATURE_DEFAULTS.md) -- Project-wide feature defaults
- [MINIEXTENDR_ATTRIBUTE.md](MINIEXTENDR_ATTRIBUTE.md) -- Complete `#[miniextendr]` option reference
- [ENVIRONMENT_VARIABLES.md](ENVIRONMENT_VARIABLES.md) -- `MINIEXTENDR_BACKTRACE` and other env vars
- [THREADS.md](THREADS.md) -- Worker thread architecture and thread safety
- [SAFETY.md](SAFETY.md) -- Safety invariants (R_UnwindProtect, GC protection)
- [TYPE_CONVERSIONS.md](TYPE_CONVERSIONS.md#error-cases) -- Type conversion error messages
- [MACRO_ERRORS.md](MACRO_ERRORS.md) -- Proc-macro and lint error codes
