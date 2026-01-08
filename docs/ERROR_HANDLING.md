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

```
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

### r_error! Macro

For explicit R errors with formatting:

```rust
use miniextendr_api::r_error;

#[miniextendr]
pub fn validate_positive(x: i32) -> i32 {
    if x < 0 {
        r_error!("x must be non-negative, got {}", x);
    }
    x
}
```

### r_stop Function

The underlying function:

```rust
use miniextendr_api::error::r_stop;

#[miniextendr]
pub fn check_input(x: i32) {
    if x < 0 {
        r_stop("x must be positive");
    }
}
```

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

### Worker Thread Pattern

For background computation, use the worker thread:

```rust
use miniextendr_api::worker::spawn_on_worker;

#[miniextendr]
pub fn expensive_compute(data: Vec<f64>) -> f64 {
    // Computation runs on worker thread
    spawn_on_worker(|| {
        data.iter().sum()
    })
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
        r_error!("rows and cols must be positive");
    }
    let expected_len = (rows * cols) as usize;
    if data.len() != expected_len {
        r_error!("data length {} doesn't match {}x{}", data.len(), rows, cols);
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

Set environment variable for detailed panic info:

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
