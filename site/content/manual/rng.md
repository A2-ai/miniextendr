+++
title = "RNG (Random Number Generation)"
weight = 38
description = "Safe access to R's random number generators from Rust."
+++

Safe access to R's random number generators from Rust.

## Overview

R's RNG maintains internal state that must be synchronized with R's `.Random.seed`
variable. Before calling any RNG function, you must call `GetRNGstate()` to load
the state. After generating random numbers, you must call `PutRNGstate()` to save
it back -- even if an error occurs. miniextendr provides two approaches: a proc-macro
attribute (recommended) and manual RAII guards.

## Table of Contents

- [Quick Start](#quick-start)
- [The `#[miniextendr(rng)]` Attribute](#the-miniextendrrng-attribute)
- [Manual Control: RngGuard and with_rng](#manual-control-rngguard-and-with_rng)
- [Available RNG Functions](#available-rng-functions)
- [Combining with Other Attributes](#combining-with-other-attributes)
- [R Longjumps and Safety](#r-longjumps-and-safety)
- [Parallel Code (Rayon)](#parallel-code-rayon)

## Quick Start

```rust
use miniextendr_api::ffi::unif_rand;

#[miniextendr(rng)]
pub fn random_sample(n: i32) -> Vec<f64> {
    (0..n).map(|_| unsafe { unif_rand() }).collect()
}
```

From R:

```r
set.seed(42)
random_sample(5)
#> [1] 0.9148060 0.9370754 0.2861395 0.8304476 0.6417455
```

## The `#[miniextendr(rng)]` Attribute

The recommended approach. Works on standalone functions, impl methods, and trait methods.

### Standalone Functions

```rust
use miniextendr_api::ffi::{unif_rand, norm_rand, exp_rand};

#[miniextendr(rng)]
pub fn uniform_sample(n: i32) -> Vec<f64> {
    (0..n).map(|_| unsafe { unif_rand() }).collect()
}

#[miniextendr(rng)]
pub fn normal_sample(n: i32) -> Vec<f64> {
    (0..n).map(|_| unsafe { norm_rand() }).collect()
}
```

### Impl Methods

```rust
#[miniextendr]
impl MySampler {
    #[miniextendr(rng)]
    fn sample_uniform(&self, n: i32) -> Vec<f64> {
        (0..n).map(|_| unsafe { unif_rand() }).collect()
    }

    #[miniextendr(rng)]
    fn static_sample(n: i32) -> Vec<f64> {
        (0..n).map(|_| unsafe { unif_rand() }).collect()
    }
}
```

### Trait Methods

```rust
#[miniextendr(env)]
impl MyTrait for MyStruct {
    #[miniextendr(rng)]
    fn random_value(&self) -> f64 {
        unsafe { unif_rand() }
    }
}
```

### Generated Code Pattern

The attribute generates code that:

1. Calls `GetRNGstate()` at the start
2. Wraps the function body in `catch_unwind`
3. Calls `PutRNGstate()` after `catch_unwind` (runs on both success AND panic)
4. Then handles the result (returns value or re-panics)

This explicit placement ensures `PutRNGstate()` is called before any error
handling, which is robust in the presence of R longjumps when combined with
`with_r_unwind_protect`.

## Manual Control: RngGuard and with_rng

For internal helper functions or code that needs finer control over RNG scope.

### RngGuard

RAII guard that calls `GetRNGstate()` on creation and `PutRNGstate()` on drop.

```rust
use miniextendr_api::rng::RngGuard;
use miniextendr_api::ffi::unif_rand;

fn generate_random() -> f64 {
    let _guard = RngGuard::new();
    unsafe { unif_rand() }
    // PutRNGstate() called automatically when _guard drops
}
```

### with_rng

Convenience function that wraps a closure in an `RngGuard` scope.

```rust
use miniextendr_api::rng::with_rng;
use miniextendr_api::ffi::unif_rand;

let values = with_rng(|| {
    (0..10).map(|_| unsafe { unif_rand() }).collect::<Vec<_>>()
});
```

### When to Use Manual vs Attribute

| Scenario | Use |
|----------|-----|
| Function exposed to R | `#[miniextendr(rng)]` |
| Internal helper (not `#[miniextendr]`) | `RngGuard` or `with_rng` |
| Code already inside `with_r_unwind_protect` | `RngGuard` or `with_rng` |
| Scoped RNG within a larger function | `RngGuard` |

## Available RNG Functions

After initializing RNG state (via attribute or guard), use these from `miniextendr_api::ffi`:

| Function | Distribution | Range |
|----------|-------------|-------|
| `unif_rand()` | Uniform | `[0, 1)` |
| `norm_rand()` | Standard normal | `(-inf, inf)` |
| `exp_rand()` | Standard exponential | `(0, inf)` |
| `R_unif_index(n)` | Uniform integer | `[0, n)` |

All require `unsafe` because they access R's global RNG state.

```rust
use miniextendr_api::ffi::{unif_rand, norm_rand, exp_rand, R_unif_index};

#[miniextendr(rng)]
pub fn rng_demo() -> Vec<f64> {
    unsafe {
        vec![
            unif_rand(),           // Uniform on [0, 1)
            norm_rand(),           // Standard normal
            exp_rand(),            // Standard exponential
            R_unif_index(100.0),   // Uniform integer on [0, 100)
        ]
    }
}
```

## Combining with Other Attributes

`rng` composes with other `#[miniextendr]` options:

```rust
// RNG + interrupt checking
#[miniextendr(rng, check_interrupt)]
pub fn interruptible_random(n: i32) -> Vec<f64> {
    (0..n).map(|_| unsafe { unif_rand() }).collect()
}

// RNG + explicit worker thread
#[miniextendr(rng, worker)]
pub fn worker_random(n: i32) -> Vec<f64> {
    (0..n).map(|_| unsafe { unif_rand() }).collect()
}
```

## R Longjumps and Safety

`RngGuard` and `with_rng` rely on Rust's drop semantics. If R triggers a longjmp
(via `Rf_error`, etc.), the guard's destructor will NOT run unless the code is
wrapped in `with_r_unwind_protect`.

This is why **`#[miniextendr(rng)]` is preferred for R-exposed functions** -- it
places `PutRNGstate()` explicitly after `catch_unwind`, outside the scope where
a longjmp could skip it.

Summary:

| Approach | Handles panics | Handles R longjumps |
|----------|---------------|-------------------|
| `#[miniextendr(rng)]` | Yes | Yes |
| `RngGuard` / `with_rng` | Yes (drop runs) | Only inside `with_r_unwind_protect` |

## Parallel Code (Rayon)

R's RNG is **not thread-safe**. Calling `unif_rand()` and friends from Rayon
threads will panic. For parallel random number generation, use a Rust RNG crate
(`rand`, `rand_chacha`) with deterministic per-chunk seeding:

```rust
use rand::SeedableRng;
use rand_chacha::ChaChaRng;
use rand::Rng;
use miniextendr_api::rayon_bridge::with_r_vec;

#[miniextendr]
fn parallel_random(len: i32, seed: i64) -> SEXP {
    with_r_vec(len as usize, |chunk: &mut [f64], offset| {
        let mut rng = ChaChaRng::seed_from_u64(seed as u64 + offset as u64);
        for slot in chunk.iter_mut() {
            *slot = rng.gen();
        }
    })
}
```

See [Rayon](../rayon/#rng-reproducibility) for full details on reproducible parallel RNG.

## See Also

- [Rayon Integration](../rayon/) -- Parallel computation (including RNG reproducibility)
- [`#[miniextendr]` Attribute](../miniextendr-attribute/) -- Complete attribute reference
- [Threads](../threads/) -- Worker thread architecture
