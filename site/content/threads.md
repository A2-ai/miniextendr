+++
title = "Thread Safety"
weight = 8
description = "R's main-thread contract, parallel Rust, and SEXP safety"
+++

R's API is designed to be called from a single thread -- the main R thread.
miniextendr keeps R work there while allowing pure Rust computation elsewhere.

## The Problem

R's stack-bound check is one immediate failure mode for off-main calls, but
disabling it is not a solution: R's global state, garbage collector, and error
signaling are also main-thread-only. Arbitrary spawned and Rayon threads must
not call R.

## Main Thread Execution

miniextendr runs all generated Rust code inline on R's main thread inside `with_r_unwind_protect`. This is safe, simple, and has no threading overhead.

```rust
#[miniextendr]
pub fn compute(x: &[f64]) -> f64 {
    // Runs on R's main thread -- safe to call R API
    x.iter().sum()
}
```

## SEXP Safety

Never touch a `SEXP` from a thread other than R's main thread. Convert R data to
plain Rust data (`Vec`, slices copied out) before a parallel section, and
convert the result back after it.

## Parallel Computation

For CPU-bound parallel work that doesn't call R:

```rust
use rayon::prelude::*;

#[miniextendr]
pub fn parallel_sum(x: Vec<f64>) -> f64 {
    x.par_iter().sum()
}
```

The `rayon` feature is available behind a feature flag. Rayon threads never call R API, so they're safe.

## Thread Checking

Checked `#[r_ffi_checked]` wrappers enforce the recorded main-thread contract
in every build mode: they run directly on main and panic for any off-main
caller. Use
`*_unchecked` only when an enclosing context has already established the main
thread, such as an ALTREP callback:

```rust
// Checked (default):
Rf_allocVector(INTSXP, 10);

// Unchecked (known-safe context):
Rf_allocVector_unchecked(INTSXP, 10);
```

## Full reference

This page is a curated entry point. See the [user manual](/manual/threads/) for the exhaustive treatment, edge cases, and every feature switch.
