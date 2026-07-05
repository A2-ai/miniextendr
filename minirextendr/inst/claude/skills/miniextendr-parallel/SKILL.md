---
name: miniextendr-parallel
description: Use when writing multi-threaded Rust inside a miniextendr package — rayon parallel iterators feeding R vectors, the SEXP-not-Send rule, "cannot be sent between threads safely" compile errors, panics on worker threads, R stack-limit errors from spawned threads, or the worker-thread feature.
---

# Parallel Rust in an R package

The one rule everything follows from: **R's C API is single-threaded.** Every
R allocation, every read/write of R memory, and R's garbage collector must
happen on the thread R owns. Rust threads may do arbitrary computation — they
just cannot touch R.

miniextendr encodes this in the type system: `SEXP` (and types wrapping it,
like `DataFrame`) are not `Send`. If your closure captures R data, it won't
compile for another thread. That compile error is the framework protecting
you — the fix is to convert to plain Rust data *first*, then parallelize.

## The standard pattern

```
R input → Rust-owned data (Vec, slices)   [main thread]
        → parallel compute (rayon/threads) [any threads]
        → single R output conversion       [main thread]
```

`#[miniextendr]` functions already run their argument conversions on the main
thread, so inside the function body you hold plain Rust types and can fan out
freely.

## Rayon (enable the `rayon` pass-through feature)

```toml
# src/rust/Cargo.toml
[features]
rayon = ["miniextendr-api/rayon"]
```

(or `minirextendr::use_rayon()`, which wires this for you.)

**Best default — `.collect_r()`**: parallel iterator writes directly into a
freshly allocated R vector, zero intermediate copy:

```rust
use miniextendr_api::rayon_bridge::{rayon::prelude::*, ParCollectR};
use miniextendr_api::{miniextendr, SEXP};

#[miniextendr]
pub fn parallel_sqrt(x: &[f64]) -> SEXP {
    x.par_iter().map(|&v| v.sqrt()).collect_r()
}
```

**Closure style** — `par_map` for element-wise, `with_r_vec` when you need
per-chunk state (RNG seeds, scratch buffers):

```rust
use miniextendr_api::rayon_bridge::{par_map, with_r_vec};

#[miniextendr]
pub fn scaled(x: &[f64], k: f64) -> SEXP {
    par_map(x, move |&v| v * k)
}

#[miniextendr]
pub fn noise(n: i32, seed: i64) -> SEXP {
    with_r_vec(n as usize, move |chunk: &mut [f64], offset| {
        let mut rng = make_rng(seed as u64 + offset as u64); // Rust RNG, per chunk
        for slot in chunk.iter_mut() { *slot = rng.random(); }
    })
}
```

**Variable-length output** (filtering, flat-mapping): collect to `Vec<T>` and
return it — the framework converts on the way out:

```rust
#[miniextendr]
pub fn positives(x: &[f64]) -> Vec<f64> {
    x.par_iter().copied().filter(|&v| v > 0.0).collect()
}
```

**DataFrames**: extract rows first (rows are plain Rust data), then rayon is
unrestricted — and the conversions themselves have parallel variants
(`into_dataframe_par`, `from_dataframe_par`; see `miniextendr-dataframe`).

### Rayon rules

- **Never call R from a rayon closure.** No R API, no `RRng` / R's RNG —
  these panic on non-main threads. Use a Rust RNG crate (`rand`,
  `rand_chacha`), seeded per chunk via the `offset` argument for
  reproducibility.
- **Chunk boundaries depend on thread count**, so chunk-seeded randomness
  differs across machines. For cross-machine reproducibility pin the pool:
  `rayon::ThreadPoolBuilder::new().num_threads(4).build_global()`.
- Respect user/CRAN thread limits: honor an env var or R option for thread
  count rather than silently taking all cores (CRAN policy caps checks at 2
  cores — make thread count configurable).

## Long-lived / hand-rolled threads

`std::thread::spawn` from package code works for pure Rust, with one R-side
wrinkle: R's stack-limit checking can misfire on foreign threads that call
back into the package. Spawn through the framework instead:

```rust
use miniextendr_api::thread::{spawn_with_r, scope_with_r, RThreadBuilder};

let handle = spawn_with_r(|| heavy_pure_rust())?;   // stack checking configured
```

`scope_with_r` is the scoped-threads variant; `RThreadBuilder` exposes stack
size when you need it. Same rule as always: these threads must not touch R.

## The `worker-thread` feature (opt-in)

With `worker-thread = ["miniextendr-api/worker-thread"]`, the framework runs
Rust work on a dedicated worker thread (panic isolation from R's stack) and
gives you two primitives:

- `run_on_worker(|| ...)` — run a `Send` closure on the worker; returns
  `Result<T, String>` (`Err` = panic message).
- `with_r_thread(|| ...)` — from inside worker code, hop back to the main
  thread for an R API call, blocking until it completes.

`Sendable<T>` wraps a non-`Send` value (like a SEXP) so it can *travel* to the
worker — but it may only be *used* inside `with_r_thread`. Wrapping a SEXP
and touching it on the worker is undefined behavior (the GC races you).

Without the feature, `run_on_worker(f)` just runs `f()` inline — code is
portable across both configurations. Don't nest `run_on_worker` inside
itself (single worker; the re-entry guard panics with a clear message).

## Debugging

- **"`SEXP` cannot be sent between threads safely"** — you captured R data in
  a parallel closure. Convert to `Vec`/slice before the parallel region.
- **Panic: "must be called from the R main thread"** — some code path called
  R API off-thread; find the R call inside your closure and hoist it out (or
  wrap in `with_r_thread` under the worker model).
- **Crash only under parallel load** — if any `unsafe` R access exists in the
  parallel region, that's the suspect. Also run the GC-torture recipe from
  `miniextendr-debugging`.
- **R CMD check hangs on Windows** — a dependency's multi-threaded Tokio
  runtime keeps stdout open; use a `current_thread` runtime (recipe in
  `miniextendr-debugging`).

Full manual (Rayon and threads chapters): https://a2-ai.github.io/miniextendr
