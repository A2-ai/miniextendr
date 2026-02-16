# Feature-Controlled Defaults

Project-wide defaults for `#[miniextendr]` options, controlled via Cargo features.

## Problem

Options like `strict`, `coerce`, and `error_in_r` must normally be specified on every
`#[miniextendr]` annotation:

```rust
#[miniextendr(strict, coerce, error_in_r)]
fn add(a: i64, b: i64) -> i64 { a + b }

#[miniextendr(strict, coerce, error_in_r)]
fn mul(a: i64, b: i64) -> i64 { a * b }
```

This is repetitive for packages that want a consistent policy across all exported functions.

## Solution

Enable a Cargo feature to apply the option everywhere. Individual functions can still
opt out with `no_` prefixed keywords.

```toml
# Cargo.toml
[dependencies]
miniextendr-api = { version = "0.1", features = ["default-strict"] }
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
| `default-strict` | Strict checked conversions for lossy types (i64, u64, isize, usize) | fns + impl blocks | `no_strict` |
| `default-coerce` | Auto-coerce parameters (e.g., `f32` from `f64`) | fns + methods | `no_coerce` |
| `default-error-in-r` | Transport Rust errors as R conditions | fns + methods | `no_error_in_r` |
| `default-r6` | R6 class system for impl blocks (instead of env) | impl blocks | `env`, `s7`, etc. |
| `default-s7` | S7 class system for impl blocks (instead of env) | impl blocks | `env`, `r6`, etc. |
| `default-worker` | Force worker thread execution | fns + methods | `no_worker` |
| `default-main-thread` | Force main thread execution | methods | `no_main_thread` (standalone fns use `unsafe(main_thread)` syntax instead) |

### Mutual Exclusivity

These feature pairs cannot be enabled simultaneously (compile error):

- `default-r6` + `default-s7`
- `default-worker` + `default-main-thread`

### When No Feature Is Enabled

With no `default-*` features enabled, behavior is identical to before: all options
default to off/env, exactly as if the features didn't exist. This is a zero-cost,
backwards-compatible addition.

## Feature Forwarding

Features are defined in `miniextendr-macros` and forwarded by `miniextendr-api`:

```
miniextendr-api/default-strict  →  miniextendr-macros/default-strict
```

Users should enable features on `miniextendr-api` (or their package's `Cargo.toml`
features section). The forwarding is automatic.

## Detailed Behavior

### Standalone Functions

Feature defaults apply to `#[miniextendr]` on standalone functions:

```rust
// With default-strict + default-coerce features enabled:

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

`default-strict` applies to the impl block level. `default-r6`/`default-s7`
set the class system default:

```rust
// With default-r6 + default-strict features enabled:

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

Per-method options (`worker`, `main_thread`, `coerce`, `error_in_r`) also respect
feature defaults:

```rust
// With default-worker + default-coerce features enabled:

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

### error_in_r + unwrap_in_r Conflict

`error_in_r` and `unwrap_in_r` are mutually exclusive. When `default-error-in-r` is
enabled and a function specifies `unwrap_in_r`, the compiler emits a helpful error:

```
error: `error_in_r` (from `default-error-in-r` feature) and `unwrap_in_r` are
       mutually exclusive; use `no_error_in_r` to opt out
```

Fix by adding `no_error_in_r`:

```rust
#[miniextendr(unwrap_in_r, no_error_in_r)]
fn fallible() -> Result<i32, String> { Ok(42) }
```

## Resolution Order

For each option, the resolution is:

1. **Explicit attribute** -- `strict` or `no_strict` on the item → uses that value
2. **Feature default** -- `cfg!(feature = "default-strict")` → uses the feature setting
3. **Built-in default** -- `false` (off) for all boolean options, `Env` for class system

Explicit attributes always win over feature defaults.

## Example: Strict-by-Default Package

```toml
# Cargo.toml
[features]
default = ["default-strict"]
default-strict = ["miniextendr-api/default-strict"]

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
default = ["default-r6"]
default-r6 = ["miniextendr-api/default-r6"]
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
| `no_strict` | `#[miniextendr(no_strict)]` on fn, `#[miniextendr(no_strict)]` on impl | `default-strict` feature |
| `no_coerce` | `#[miniextendr(no_coerce)]` on fn, `#[miniextendr(r6(no_coerce))]` on method | `default-coerce` feature |
| `no_error_in_r` | `#[miniextendr(no_error_in_r)]` on fn, `#[miniextendr(r6(no_error_in_r))]` on method | `default-error-in-r` feature |
| `no_worker` | `#[miniextendr(no_worker)]` on fn, `#[miniextendr(r6(no_worker))]` on method | `default-worker` feature |
| `no_main_thread` | `#[miniextendr(r6(no_main_thread))]` on method | `default-main-thread` feature |
| `env` / `r6` / `s7` / `s3` / `s4` | `#[miniextendr(env)]` on impl | `default-r6` or `default-s7` feature |
