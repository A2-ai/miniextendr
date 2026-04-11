+++
title = "Error Handling"
weight = 7
description = "Panics, Result types, and unwind protection"
+++

miniextendr handles three types of errors:

1. **Rust panics** -- converted to R errors
2. **R errors** (`Rf_error`) -- Rust destructors run, then R unwinds
3. **Result errors** -- can be returned as R values or converted to R errors

## Panics

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
divide(1L, 0L)
# Error: Division by zero!
```

## Result Types

Return `Result<T, E>` for structured error handling:

```rust
#[miniextendr]
pub fn parse_int(s: &str) -> Result<i32, String> {
    s.parse().map_err(|e| format!("Parse error: {}", e))
}
```

By default, `Err` values cause R errors. Use `#[miniextendr(unwrap_in_r)]` to return errors as R values:

```rust
#[miniextendr(unwrap_in_r)]
pub fn try_parse(s: &str) -> Result<i32, String> {
    s.parse().map_err(|e| e.to_string())
}
```

```r
try_parse("42")      # 42
try_parse("abc")     # list(error = "invalid digit...")
```

## R_UnwindProtect

miniextendr wraps all R API calls in `R_UnwindProtect`, ensuring Rust destructors run even when R longjmps (e.g., on `stop()` or `Rf_error()`).

```text
User Rust code
  |
  v
with_r_unwind_protect(|| { ... })
  |
  +-- Rust panic? --> catch_unwind --> R error
  +-- R longjmp?  --> R_UnwindProtect --> destructors run --> R unwind continues
  +-- Success?    --> return SEXP normally
```

## Best Practices

- Use `panic!()` instead of `Rf_error()` -- the framework converts panics safely
- Return `Result<T, String>` for recoverable errors
- Use `#[miniextendr(unwrap_in_r)]` when callers should handle errors in R
- Never call `Rf_error()` directly (lint rule MXL300 warns about this)
