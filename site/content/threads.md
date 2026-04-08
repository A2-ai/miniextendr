+++
title = "Thread Safety"
weight = 8
description = "Calling R APIs from threads, worker thread pattern, and SEXP safety"
+++

R's API is designed to be called from a single thread -- the main R thread. miniextendr provides safe abstractions for multi-threaded Rust code.

## The Problem

When you spawn a new thread and try to call R functions, you get a segfault. R tracks the main thread's stack bounds and asserts calls come from within that range.

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

The `worker-thread` feature enables a dedicated worker thread:

```toml
[dependencies]
miniextendr-api = { features = ["worker-thread"] }
```

With this feature:
- User code runs on a dedicated worker thread
- R API calls are dispatched back to the main thread via `with_r_thread()`
- Panics on the worker are caught and forwarded as R errors

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

Debug builds include thread assertions via `#[r_ffi_checked]`. Every FFI call verifies it's on the main thread. Use `*_unchecked` variants when you're certain of thread safety (e.g., inside ALTREP callbacks):

```rust
// Debug-checked (default):
Rf_allocVector(INTSXP, 10);

// Unchecked (known-safe context):
Rf_allocVector_unchecked(INTSXP, 10);
```
