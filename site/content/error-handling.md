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

Return `Result<T, E>` for structured error handling. By default (`error_in_r` is on), `Err` values are transported as tagged SEXP values across the Rust boundary, then the generated R wrapper raises a structured R condition:

```rust
#[miniextendr]
pub fn parse_int(s: &str) -> Result<i32, String> {
    s.parse().map_err(|e| format!("Parse error: {}", e))
}
```

To return `Result<T, E>` to R as a value instead of an error, use `#[miniextendr(unwrap_in_r, no_error_in_r)]`:

```rust
#[miniextendr(unwrap_in_r, no_error_in_r)]
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

Note: `with_r_unwind_protect_error_in_r` leaks ~8 bytes (the `RErrorMarker` + Box header) on the R-longjmp path (`R_ContinueUnwind`). Regular Rust panics do not leak. This is why MXL300 flags direct `Rf_error()` calls: they longjmp through Rust frames and bypass destructors unless wrapped correctly.

## Never Call Rf_error() Directly

**Calling `Rf_error()` or `Rf_errorcall()` from Rust is forbidden.**

These functions raise an R error by executing a `longjmp`, which jumps past every Rust
stack frame without running destructors. Heap allocations, locks, and RAII guards all
leak.

Use `panic!()` or `Err(...)` instead. The framework catches both and raises a structured
R condition with class `c("rust_error", "simpleError", "error", "condition")`:

```rust
// Both of these produce a simpleError on the R side:
panic!("something went wrong");
return Err("something went wrong".to_string());
```

miniextendr-lint (MXL300) flags direct `Rf_error()` call sites at build time.

**Note on the `~8-byte leak`**: on the `R_ContinueUnwind` path (R error propagating through
`with_r_unwind_protect_error_in_r`), approximately 8 bytes are leaked per event
(`RErrorMarker` + Box header). This is a documented, bounded cost -- not a bug. Regular
Rust panics do not leak.

## Best Practices

- Use `panic!()` instead of `Rf_error()` -- the framework converts panics safely
- Return `Result<T, String>` for recoverable errors
- Use `#[miniextendr(unwrap_in_r, no_error_in_r)]` when callers should handle errors in R
- Never call `Rf_error()` directly (lint rule MXL300 warns about this)

## Full reference

This page is a curated entry point. See the [user manual](/manual/error-handling/) for the exhaustive treatment, edge cases, and every feature switch.
