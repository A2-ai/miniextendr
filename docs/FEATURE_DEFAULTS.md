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
| `coerce-default` | Widen native `i32`/`f64` (and `Vec`) to the other numeric sources; preserve non-native numeric inputs; also accept integer `0`/`1` for `bool` and `Vec<bool>` | fns + methods | `no_coerce` |
| `no-preconditions-default` | Drop the R-side type checks | fns + impl blocks | `preconditions` |
| `r6-default` | R6 class system for impl blocks (instead of env) | impl blocks | `env`, `s7`, etc. |
| `s7-default` | S7 class system for impl blocks (instead of env) | impl blocks | `env`, `r6`, etc. |
| `worker-default` | Force worker thread execution (implies `worker-thread`) | fns + methods | `no_worker` |

All six `*-default` features above are denylisted from rpkg's auto-detected
default build (`rpkg/tools/detect-features.R`) — enabling any of them flips
codegen semantics crate-wide, so no PR-gating job ever builds or runs the R
wrappers they generate. Their only runtime coverage is the scheduled
`feature-legs` job in `.github/workflows/ci.yml` (weekly + `workflow_dispatch`),
which rebuilds rpkg with one feature bundle on top of the detected base set and
re-runs `tests/testthat/test-feature-defaults.R` against it. The `coerce-default`
leg also runs the coercion regression suites.

### Hardcoded Defaults (No Longer Feature-Controlled)

The following were previously opt-in features but are now **always enabled by default**:

| Default | Effect | Notes |
|---------|--------|-------|
| Tagged-condition transport | Transport Rust errors as R conditions (panics, `Err`, `None` → tagged SEXP → R wrapper raises) | Only path; no opt-out. `unwrap_in_r` is orthogonal (Result-as-value vs Result-as-error-boundary). |
| Main thread | All code runs on R's main thread | Opt into the worker thread with `worker`. |

### Orthogonality

`no-preconditions-default` is **orthogonal** to `strict-default` and
`coerce-default` — any combination is valid:

| Features | Effect |
|----------|--------|
| `no-preconditions-default` only | Unchecked wrappers with permissive conversions |
| `no-preconditions-default` + `strict-default` | Unchecked wrappers with strict i64/u64/… checking |
| `no-preconditions-default` + `coerce-default` | Unchecked wrappers with auto-coercion |
| All three | Unchecked + strict + coerce |

### Mutual Exclusivity

These feature pairs cannot be enabled simultaneously (compile error):

- `r6-default` + `s7-default`

## Feature Forwarding

Features are defined in `miniextendr-macros` and forwarded by `miniextendr-api`:

```text
miniextendr-api/strict-default  →  miniextendr-macros/strict-default
miniextendr-api/no-preconditions-default  →  miniextendr-macros/no-preconditions-default
```

Users should enable features on `miniextendr-api` (or their package's `Cargo.toml`
features section). The forwarding is automatic.

## Detailed Behavior

### Coercion inputs

`coerce` and `coerce-default` preserve normal input types. Non-native numeric
scalars and vectors already accept integer, double, logical, and raw inputs;
coercion keeps those checked conversions. The native `i32` / `Vec<i32>` and
`f64` / `Vec<f64>` accept one `SEXPTYPE` without coerce, and the R precondition
says so (`f(3)` fails for `x: i32` with "'x' must be integer", `f(1L)` for
`x: f64` with "'x' must be double"); with coerce they widen to the same four
sources, `i32` from whole-number doubles only, and the R precondition names the
widened domain. NA propagates where the declared type can carry it (`f64`,
`Vec<f64>`, `Vec<i32>`), as `as.numeric()` / `as.integer()` would; a scalar
`i32` keeps rejecting NA. Boolean scalars and vectors retain logical inputs and
additionally accept integer `0`/`1` (other integers and NA are errors).
`Option<bool>` keeps its nullable logical conversion.

`strict` takes precedence for the lossy integer types it checks. `no_strict`
restores their normal multi-source conversion, including when `coerce-default`
is enabled. See [Conversion Behavior Matrix](CONVERSION_MATRIX.md).

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

### `no-preconditions-default`

The `no-preconditions-default` feature applies `no_preconditions` to every
`#[miniextendr]` function and impl block: the generated wrappers drop their
R-side type checks, one `isTRUE()` guard per check. Type errors still
propagate from Rust's `TryFromSexp`, as the same argument-error condition
(#1591), but worded by the conversion (`'x' must be a single integer: got
character`) rather than by the R check (`'x' must be integer`). Checks named
per parameter (`inherits`, `no_na`) are kept: the Rust conversion does not
repeat them. `no_na` on the reading markers (`AsNumeric*`, `AsCharacter*`)
also checks the converted value, and that check stays too.

The feature does not touch the call a wrapper reports: every wrapper passes
`.call = sys.call()` (or the caller's call under `call = caller`), so
conditions, including deferred warnings, name the call as written either way
(see [CALL_ATTRIBUTION.md](CALL_ATTRIBUTION.md)).

Each dropped check saves one `isTRUE()` guard on every call. On the rpkg
fixtures (installed, byte-compiled wrappers, `bench::mark`, median of five
one-second runs on an arm64 Mac), a one-argument `i32` identity takes 1.4 µs
with the checks and 0.7 µs without (`fast_i32_default` /
`fast_i32_no_preconditions`), about 2×; a three-argument sum takes 3.0 µs and
0.9 µs (`fast_sum3_default` / `fast_sum3_no_preconditions`), about 3.3×.

```rust
// With the no-preconditions-default feature enabled:

#[miniextendr]                   // no_preconditions = true
fn hot_fn(x: i32) -> i32 { x }

// Keep the checks for a function where R-side type errors need the full UX:
#[miniextendr(preconditions)]    // no_preconditions = false
fn user_facing_fn(x: i32) -> i32 { x }
```

On a function the pair also takes `= true` / `= false`
(`preconditions = false` is `no_preconditions`), and on a function or an impl
block the last one written wins, like `worker` / `no_worker`.

The same applies to impl blocks, inherent and trait impls alike (including an
empty-body trait impl and the setter of an R6 active binding):

```rust
// With no-preconditions-default + r6-default:

#[miniextendr]                   // R6, every method unchecked
impl MyCounter { ... }

// One impl block where the full UX matters:
#[miniextendr(preconditions)]
impl UserFacingType { ... }
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
| `preconditions` | `#[miniextendr(preconditions)]` on fn or impl, inherent or trait (`preconditions = true` on a fn) | `no-preconditions-default` feature (keeps the R-side type checks) |
| `no_preconditions` | `#[miniextendr(no_preconditions)]` on fn or impl, inherent or trait (`no_preconditions = true` on a fn) | Built-in default: drops the R-side type checks |
| `worker` | `#[miniextendr(worker)]` on fn, `#[miniextendr(r6(worker))]` on method | Built-in main thread default |
| `no_worker` | `#[miniextendr(no_worker)]` on fn, `#[miniextendr(r6(no_worker))]` on method | `worker-default` feature |
| `env` / `r6` / `s7` / `s3` / `s4` | `#[miniextendr(env)]` on impl | `r6-default` or `s7-default` feature |
