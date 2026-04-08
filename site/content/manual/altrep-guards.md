+++
title = "ALTREP Guard Modes"
weight = 31
description = "Controls how panics and R errors are caught in ALTREP callback trampolines."
+++

Controls how panics and R errors are caught in ALTREP callback trampolines.

## Overview

ALTREP callbacks are called by R's C runtime, which means Rust panics would unwind
through C frames (undefined behavior). Guard modes wrap callbacks in protection:

| Mode | Attribute | Catches | Overhead | Default |
|------|-----------|---------|----------|---------|
| **RUnwind** | `#[altrep(r_unwind)]` or omitted | Rust panics + R longjmps | Minimal | Yes |
| **RustUnwind** | `#[altrep(rust_unwind)]` | Rust panics only | ~1-2ns | No |
| **Unsafe** | `#[altrep(unsafe)]` | Nothing | Zero | No |

## Syntax

```rust
// Default: RUnwind (catches Rust panics + R longjmps)
#[derive(AltrepInteger)]
#[altrep(len = "length")]
pub struct MyInts {
    data: Vec<i32>,
    length: usize,
}

// Unsafe: no protection (maximum performance)
#[derive(AltrepInteger)]
#[altrep(len = "length", unsafe)]
pub struct TrivialInts {
    data: Vec<i32>,
    length: usize,
}

// RUnwind: catches both Rust panics and R longjmps
#[derive(AltrepString)]
#[altrep(len = "length", r_unwind)]
pub struct DynamicStrings {
    data: Vec<String>,
    length: usize,
}
```

## When to Use Each Mode

### RUnwind (default) — Safe for All Callbacks

The default. Catches both Rust panics and R longjmps. Safe for callbacks that
call R API functions (e.g., `Rf_mkCharLenCE`, `Rf_allocVector`).

### RustUnwind — Pure Rust Callbacks Only

Use when callbacks only access Rust data and don't call R API functions.
Slightly lower overhead than RUnwind but **unsafe if callbacks call R APIs**.

```rust
#[derive(AltrepInteger)]
pub struct Sequence {
    start: i32,
    step: i32,
    length: usize,
}

impl AltIntegerData for Sequence {
    fn elt(&self, i: usize) -> i32 {
        self.start + (i as i32) * self.step  // Pure arithmetic, cannot panic
    }
}
```

- Wraps in `catch_unwind`
- On panic: extracts message, fires telemetry, raises R error via `Rf_error()`
- Safe for pure-Rust code only
- **Unsafe if callbacks call R APIs** — R longjmps bypass `catch_unwind`

### RUnwind Example — Callbacks That Call R APIs

The default. Use for all callbacks, especially those that invoke R functions:

```rust
#[derive(AltrepString)]
#[altrep(r_unwind)]
pub struct FormattedValues {
    values: Vec<f64>,
    length: usize,
}

impl AltStringData for FormattedValues {
    fn elt(&self, i: usize) -> String {
        // If this called Rf_eval or Rf_allocVector internally,
        // those could longjmp on error. r_unwind catches that.
        format!("{:.2}", self.values[i])
    }
}
```

Situations requiring `r_unwind`:
- `Rf_allocVector()` in a callback (can trigger GC longjmp)
- `Rf_eval()` or evaluating R expressions
- `Rf_coerceVector()` or type conversions
- Any R API call that might error

Uses `R_UnwindProtect` + `catch_unwind` (nested guards).

### Unsafe — Trivial Callbacks

Use when callbacks are guaranteed not to panic and you need maximum performance:

```rust
#[derive(AltrepReal)]
#[altrep(unsafe)]
pub struct ConstantVec {
    value: f64,
    length: usize,
}

impl AltRealData for ConstantVec {
    fn elt(&self, _i: usize) -> f64 {
        self.value  // Trivial field access, cannot panic
    }
}
```

- No protection at all
- If the callback panics, behavior is **undefined** (crash/corruption)
- Only use for trivial operations (field access, simple arithmetic)
- Saves ~1-2ns per callback invocation

## How It Works

The guard mode is set via the `const GUARD` associated constant on the `Altrep` trait:

```rust
pub trait Altrep {
    const GUARD: AltrepGuard = AltrepGuard::RustUnwind;
    // ...
}
```

The `#[altrep(unsafe)]` / `#[altrep(r_unwind)]` attributes override this constant
in the generated code.

At the trampoline level, `guarded_altrep_call` dispatches based on `T::GUARD`:

```rust
fn guarded_altrep_call<T: Altrep, F, R>(f: F) -> R
where F: FnOnce() -> R {
    match T::GUARD {
        AltrepGuard::Unsafe    => f(),
        AltrepGuard::RustUnwind => guarded_ffi_call(f, CatchUnwind, Altrep),
        AltrepGuard::RUnwind   => guarded_ffi_call(f, RUnwind, Altrep),
    }
}
```

Since `T::GUARD` is a `const`, the compiler eliminates dead branches at
monomorphization time — zero runtime cost for the chosen mode.

## See Also

- [ALTREP.md](ALTREP.md) — Full ALTREP guide
- [ALTREP_QUICKREF.md](ALTREP_QUICKREF.md) — Quick reference checklist
- [ERROR_HANDLING.md](ERROR_HANDLING.md) — Panic handling across FFI boundaries
