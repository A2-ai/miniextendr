---
name: miniextendr-parallel
description: Use when writing multi-threaded Rust inside a miniextendr package — rayon parallel iterators feeding R vectors, the SEXP-not-Send rule, "cannot be sent between threads safely" compile errors, panics from R API calls off the main thread, or R stack-limit errors from spawned threads.
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

- **Never call R from a rayon closure.** No R API, no R's RNG — the R API
  panics on non-main threads, and `RngGuard` / `RRng` are `!Send`, so a
  closure that draws from them does not compile. Use a Rust RNG crate (`rand`,
  `rand_chacha`), seeded per chunk via the `offset` argument for
  reproducibility.
- **Chunk boundaries depend on thread count**, so chunk-seeded randomness
  differs across machines. For cross-machine reproducibility pin the pool:
  `rayon::ThreadPoolBuilder::new().num_threads(4).build_global()`.
- **Respect user/CRAN thread limits** — don't silently take all cores. On the
  Rust side, `miniextendr_api::optionals::parallel::effective_threads()`,
  `::ensure_pool()`, and `::set_threads()` centralize this;
  `effective_threads()` already applies the CRAN core cap
  (`_R_CHECK_LIMIT_CORES_`). To let R users tune the count, expose thin
  wrappers like `miniextendr_num_threads()` / `miniextendr_set_threads()` — the
  miniextendr exemplar package ships these, but scaffolded packages don't get
  them automatically yet (tracked upstream in issue #1132). For the full
  precedence table (env var, R option, explicit call, CRAN cap), see the
  miniextendr manual's "Controlling Parallelism from R" section — don't
  restate it here.

## Long-lived / hand-rolled threads

`std::thread::spawn` from package code works for pure Rust, with one R-side
rule: the spawned closure must not touch R. Use the standard library directly;
set a larger stack with `std::thread::Builder` if the Rust workload needs it:

```rust
let handle = std::thread::Builder::new()
    .stack_size(8 * 1024 * 1024)
    .spawn(heavy_pure_rust)?;
```

Do not use `StackCheckGuard`, `spawn_with_r`, or `scope_with_r` to call R from
those threads. Disabling R's process-global stack check removes only one crash
path and does not make the API or GC thread-safe; that misleading surface is
tracked for removal in miniextendr issue #1352.

## Debugging

- **"`SEXP` cannot be sent between threads safely"** — you captured R data in
  a parallel closure. Convert to `Vec`/slice before the parallel region.
- **Panic: "R's C API was called from a thread other than R's main thread"** —
  some code path called the R API off-thread; find the R call inside your
  closure and hoist it out, before or after the parallel region.
- **Crash only under parallel load** — if any `unsafe` R access exists in the
  parallel region, that's the suspect. Also run the GC-torture recipe from
  `miniextendr-debugging`.
- **R CMD check hangs on Windows** — a dependency's multi-threaded Tokio
  runtime keeps stdout open; use a `current_thread` runtime (recipe in
  `miniextendr-debugging`).

Full manual (Rayon and threads chapters): https://a2-ai.github.io/miniextendr
