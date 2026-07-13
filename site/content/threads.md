+++
title = "Thread Safety"
weight = 8
description = "R's main-thread contract, opt-in worker dispatch, and SEXP safety"
+++

R's API is designed to be called from a single thread -- the main R thread.
miniextendr keeps R work there while allowing pure Rust computation elsewhere.

## The Problem

R's stack-bound check is one immediate failure mode for off-main calls, but
disabling it is not a solution: R's global state, garbage collector, and error
signaling are also main-thread-only. Arbitrary spawned and Rayon threads must
not call R.

## Default: Main Thread Execution

By default, miniextendr runs all Rust code inline on R's main thread inside `with_r_unwind_protect`. This is safe, simple, and has no threading overhead.

```rust
#[miniextendr]
pub fn compute(x: &[f64]) -> f64 {
    // Runs on R's main thread -- safe to call R API
    x.iter().sum()
}
```

## Worker Thread (Opt-in)

The `worker-thread` feature enables the dedicated-worker infrastructure:

```toml
[dependencies]
miniextendr-api = { features = ["worker-thread"] }
```

Select worker execution per export with `#[miniextendr(worker)]`, or crate-wide
with `worker-default`. Then:

- The opted-in Rust body runs on the dedicated worker
- R API calls are dispatched back to the main thread via `with_r_thread()`
- Panics on the worker are caught and forwarded as R errors

Enabling `worker-thread` alone does not change the proc-macro default.

## SEXP Safety

`SEXP` values are **not** `Send` -- they cannot be safely shared across threads. Functions that take `SEXP` parameters automatically run on the main thread (even with `worker-thread` enabled).

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
in every build mode: they run directly on main, route from an active
miniextendr worker context, and panic for arbitrary off-main callers. Use
`*_unchecked` only when an enclosing context has already established the main
thread, such as an ALTREP callback:

```rust
// Checked/routed (default):
Rf_allocVector(INTSXP, 10);

// Unchecked (known-safe context):
Rf_allocVector_unchecked(INTSXP, 10);
```

## Full reference

This page is a curated entry point. See the [user manual](/manual/threads/) for the exhaustive treatment, edge cases, and every feature switch.
