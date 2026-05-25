# Feature-Controlled Defaults

Project-wide defaults for `#[miniextendr]` options, controlled via Cargo features.

## Problem

Options like `strict` and `coerce` must normally be specified on every
`#[miniextendr]` annotation:

```rust
#[miniextendr(strict, coerce)]
fn add(a: i64, b: i64) -> i64 { a + b }

#[miniextendr(strict, coerce)]
fn mul(a: i64, b: i64) -> i64 { a * b }
```

This is repetitive for packages that want a consistent policy across all exported functions.

## Solution

Enable a Cargo feature to apply the option everywhere. Individual functions can still
opt out with `no_` prefixed keywords.

```toml
# Cargo.toml
[dependencies]
miniextendr-api = { version = "0.1", features = ["strict-default"] }
```

```rust
// All functions now use strict conversions automatically
#[miniextendr]
fn add(a: i64, b: i64) -> i64 { a + b }

// Opt out for this one function
#[miniextendr(no_strict)]
fn legacy_add(a: i64, b: i64) -> i64 { a + b }
```

## Available Features

| Feature | Effect | Scope | Opt-out keyword |
|---------|--------|-------|-----------------|
| `strict-default` | Strict checked conversions for lossy types (i64, u64, isize, usize) | fns + impl blocks | `no_strict` |
| `coerce-default` | Auto-coerce parameters (e.g., `f32` from `f64`) | fns + methods | `no_coerce` |
| `r6-default` | R6 class system for impl blocks (instead of env) | impl blocks | `env`, `s7`, etc. |
| `s7-default` | S7 class system for impl blocks (instead of env) | impl blocks | `env`, `r6`, etc. |
| `worker-default` | Force worker thread execution (implies `worker-thread`) | fns + methods | `no_worker` |

### Hardcoded Defaults (No Longer Feature-Controlled)

The following were previously opt-in features but are now **always enabled by default**:

| Default | Effect | Notes |
|---------|--------|-------|
| Tagged-condition transport | Transport Rust errors as R conditions (panics, `Err`, `None` → tagged SEXP → R wrapper raises) | Only path; no opt-out. `unwrap_in_r` is orthogonal (Result-as-value vs Result-as-error-boundary). |
| Main thread | All code runs on R's main thread | Opt into the worker thread with `worker`. |

### Mutual Exclusivity

These feature pairs cannot be enabled simultaneously (compile error):

- `r6-default` + `s7-default`

## Feature Forwarding

Features are defined in `miniextendr-macros` and forwarded by `miniextendr-api`:

```text
miniextendr-api/strict-default  →  miniextendr-macros/strict-default
```

Users should enable features on `miniextendr-api` (or their package's `Cargo.toml`
features section). The forwarding is automatic.

## Detailed Behavior

### Standalone Functions

Feature defaults apply to `#[miniextendr]` on standalone functions:

```rust
// With strict-default + coerce-default features enabled:

#[miniextendr]                    // strict=true, coerce=true (from features)
fn f1(x: i64) -> i64 { x }

#[miniextendr(no_strict)]         // strict=false, coerce=true
fn f2(x: i64) -> i64 { x }

#[miniextendr(no_coerce)]         // strict=true, coerce=false
fn f3(x: f32) -> f32 { x }

#[miniextendr(no_strict, no_coerce)]  // strict=false, coerce=false
fn f4(x: i64) -> i64 { x }
```

### Impl Blocks

`strict-default` applies to the impl block level. `r6-default`/`s7-default`
set the class system default:

```rust
// With r6-default + strict-default features enabled:

#[miniextendr]                    // class_system=R6, strict=true
impl MyType { ... }

#[miniextendr(env)]               // class_system=Env (overridden), strict=true
impl MyType { ... }

#[miniextendr(no_strict)]         // class_system=R6, strict=false
impl MyType { ... }

#[miniextendr(s7)]                // class_system=S7 (overridden), strict=true
impl MyType { ... }
```

### Methods

Per-method options (`worker`, `main_thread`, `coerce`) also respect feature
defaults:

```rust
// With worker-default + coerce-default features enabled:

#[miniextendr(r6)]
impl MyType {
    #[miniextendr(r6())]              // worker=true, coerce=true (from features)
    fn method1(&self, x: f32) { }

    #[miniextendr(r6(no_worker))]     // worker=false, coerce=true
    fn method2(&self) { }

    #[miniextendr(r6(no_coerce))]     // worker=true, coerce=false
    fn method3(&self, x: f32) { }
}
```

### `unwrap_in_r`

`unwrap_in_r` is orthogonal to the tagged-condition transport. It controls
whether `Result<T, E>` is treated as a Rust-origin failure (`Err` → tagged
condition → `stop()`) or as a value to surface to R as a list with an `$error`
slot. There is no conflict to resolve:

```rust
#[miniextendr(unwrap_in_r)]
fn fallible() -> Result<i32, String> { Ok(42) }
```

## Resolution Order

For each option, the resolution is:

1. **Explicit attribute** -- `strict` or `no_strict` on the item → uses that value
2. **Feature default** -- `cfg!(feature = "strict-default")` → uses the feature setting (for feature-controlled options)
3. **Built-in default** -- `main_thread=true`, tagged-condition transport always on, `false` for other boolean options, `Env` for class system

Explicit attributes always win over feature/built-in defaults.

## Example: Strict-by-Default Package

```toml
# Cargo.toml
[features]
default = ["strict-default"]
strict-default = ["miniextendr-api/strict-default"]

[dependencies]
miniextendr-api = { version = "0.1" }
```

```rust
// All functions use strict conversions
#[miniextendr]
fn process(x: i64) -> i64 { x * 2 }

// This specific function needs lossy behavior for backwards compat
#[miniextendr(no_strict)]
fn legacy_process(x: i64) -> i64 { x * 2 }
```

## Example: R6-by-Default Package

```toml
# Cargo.toml
[features]
default = ["r6-default"]
r6-default = ["miniextendr-api/r6-default"]
```

```rust
// All impl blocks generate R6 classes
#[miniextendr]
impl Counter { ... }    // R6

// This one needs env for specific reasons
#[miniextendr(env)]
impl LightWrapper { ... }  // env (overridden)
```

## Complete Opt-Out Keywords Reference

| Keyword | Where | Cancels |
|---------|-------|---------|
| `no_strict` | `#[miniextendr(no_strict)]` on fn, `#[miniextendr(no_strict)]` on impl | `strict-default` feature |
| `no_coerce` | `#[miniextendr(no_coerce)]` on fn, `#[miniextendr(r6(no_coerce))]` on method | `coerce-default` feature |
| `worker` | `#[miniextendr(worker)]` on fn, `#[miniextendr(r6(worker))]` on method | Built-in main thread default |
| `no_worker` | `#[miniextendr(no_worker)]` on fn, `#[miniextendr(r6(no_worker))]` on method | `worker-default` feature |
| `env` / `r6` / `s7` / `s3` / `s4` | `#[miniextendr(env)]` on impl | `r6-default` or `s7-default` feature |
