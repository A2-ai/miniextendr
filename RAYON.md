# Rayon Integration Guide

Miniextendr provides a small, explicit Rayon bridge for parallel Rust work with
R-safe boundaries. The rule is simple: do not call R APIs inside parallel
iterators. Do R work before or after the parallel section.

## Enable the feature

```toml
[dependencies]
miniextendr-api = { path = "../miniextendr-api", features = ["rayon"] }
```

## Quick start

### Parallel collection with `RVec<T>`

```rust
use miniextendr_api::{miniextendr, rayon_bridge::RVec};
use miniextendr_api::rayon_bridge::rayon::prelude::*;

#[miniextendr]
fn parallel_sqrt(x: &[f64]) -> RVec<f64> {
    x.par_iter().map(|&v| v.sqrt()).collect()
}
```

`RVec<T>` implements `FromParallelIterator` and `IntoR`, so returning it from a
`#[miniextendr]` function converts on the main thread.

### Zero-copy output with `with_r_vec<T>`

```rust
use miniextendr_api::{miniextendr, rayon_bridge::with_r_vec};
use miniextendr_api::ffi::SEXP;
use miniextendr_api::rayon_bridge::rayon::prelude::*;

#[miniextendr]
fn parallel_sqrt_fast(x: &[f64]) -> SEXP {
    with_r_vec::<f64, _>(x.len(), |out| {
        out.par_iter_mut()
            .zip(x.par_iter())
            .for_each(|(slot, &v)| *slot = v.sqrt());
    })
}
```

### Reductions

```rust
use miniextendr_api::{miniextendr, rayon_bridge::reduce};
use miniextendr_api::ffi::SEXP;

#[miniextendr]
fn parallel_sum(x: &[f64]) -> SEXP {
    reduce::sum(x)
}
```

## Threading model

- `#[miniextendr]` functions run on a worker thread with main-thread routing.
- `with_r_thread` executes closures on the main R thread when needed.
- `with_r_vec<T>` allocates/protects on the main thread, runs your closure on the
  current thread, then unprotects on the main thread.

Do not call R APIs from inside Rayon closures. That includes `IntoR::into_sexp()`
or any `ffi::` function.

## API summary

- `rayon_bridge::rayon` re-exports Rayon to avoid version mismatches.
- `with_r_vec<T>` pre-allocates an R vector and exposes a mutable slice for
  parallel writes (`T: RNativeType + Send + Sync`).
- `RVec<T>` is a parallel collection container that converts via `IntoR`.
- `reduce::*` provides `sum`, `sum_int`, `min`, `max`, and `mean` helpers.
- `perf::*` exposes `num_threads`, `in_rayon_thread`, and `thread_index`.

## Safety rules (short version)

- Never call R APIs inside parallel iterators.
- Only write to disjoint indices when using `with_r_vec<T>`.
- Convert to R on the main thread (return `RVec<T>` or call `with_r_thread`).

If you see a panic about missing worker context, ensure
`miniextendr_worker_init()` is called in `R_init_<pkgname>()`.
