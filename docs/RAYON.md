# Rayon Integration Guide

Miniextendr provides seamless integration with [Rayon](https://docs.rs/rayon) for parallel computation in R packages. This enables writing high-performance parallel code while maintaining R's safety guarantees.

## Table of Contents

- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [API Overview](#api-overview)
- [Patterns](#patterns)
- [RNG Reproducibility](#rng-reproducibility)
- [Performance](#performance)
- [Safety](#safety)
- [Examples](#examples)

## Quick Start

### Enable the Feature

```toml
[dependencies]
miniextendr-api = { path = "../miniextendr-api", features = ["rayon"] }
```

### Simplest Example

```rust
use miniextendr_api::prelude::*;
use miniextendr_api::rayon_bridge::{rayon::prelude::*, ParCollectR};

#[miniextendr]
fn parallel_sqrt(x: &[f64]) -> SEXP {
    // .collect_r() writes directly into R memory - zero intermediate allocation
    x.par_iter().map(|&val| val.sqrt()).collect_r()
}
```

### Alternative Styles

```rust
use miniextendr_api::rayon_bridge::*;

// par_map: same zero-copy performance, closure style
#[miniextendr]
fn parallel_sqrt_map(x: &[f64]) -> SEXP {
    par_map(x, |&v| v.sqrt())
}

// Vec<T> return: simplest (one extra allocation, miniextendr converts to R)
#[miniextendr]
fn parallel_sqrt_vec(x: &[f64]) -> Vec<f64> {
    x.par_iter().map(|&val| val.sqrt()).collect()
}
```

## Architecture

### Design Philosophy

**Rust computation: Parallel on Rayon threads (normal 2MB stacks)**
**R API calls: Serial on main thread**

The framework handles all parallelism internally. User closures receive **chunks**
of data and never need to call `par_iter()` or manage thread dispatch.

### Thread Model

```text
┌─────────────────────────────────────────┐
│   Rayon Thread Pool (2MB stacks)        │
│                                         │
│   Thread 1   Thread 2   Thread 3        │
│      ↓          ↓          ↓            │
│   chunk[0]  chunk[1]  chunk[2]          │ ← Parallel computation
│      ↓          ↓          ↓            │
│   f(chunk,0) f(chunk,n) f(chunk,2n)     │ ← User closure per chunk
└──────┬──────────┬──────────┬────────────┘
       │          │          │
       └──────────┴──────────┘
                  ↓
       ┌──────────────────────┐
       │  Main R Thread       │
       │  (allocates before,  │ ← R memory alloc/dealloc
       │   returns after)     │
       └──────────────────────┘
```

### Key Insights

1. **Framework-managed parallelism**: `with_r_vec`, `par_map`, etc. split data into chunks internally
2. **Deterministic chunks**: Same `(length, thread_count)` always produces the same chunk boundaries
3. **Zero copy**: Pre-allocation writes directly into R memory
4. **No R calls in closures**: Pure Rust only inside parallel sections

## API Overview

### Chunk-Based Fill

#### `with_r_vec(len, f)`: chunk-based parallel fill

Allocates an R vector of `len` elements, splits into chunks, calls `f(chunk, offset)`
in parallel. The closure receives a mutable slice and the starting index.

```rust
pub fn with_r_vec<T, F>(len: usize, f: F) -> SEXP
where
    T: RNativeType + Send + Sync,
    F: Fn(&mut [T], usize) + Send + Sync,
```

```rust
with_r_vec(1000, |chunk: &mut [f64], offset: usize| {
    for (i, slot) in chunk.iter_mut().enumerate() {
        *slot = ((offset + i) as f64).sqrt();
    }
});
```

#### `with_r_vec_map(len, f)`: index-based fill (sugar)

Each element depends only on its index. Sugar over `with_r_vec`.

```rust
with_r_vec_map(1000, |i: usize| (i as f64).sqrt());
```

### Parallel Map

#### `par_map(input, f)`: transform slice to R vector

Element-wise parallel transform. Writes directly into R memory (zero intermediate allocation).

```rust
pub fn par_map<T, U, F>(input: &[T], f: F) -> SEXP
where
    T: Send + Sync,
    U: RNativeType + Send + Sync,
    F: Fn(&T) -> U + Send + Sync,
```

```rust
// Parallel sqrt
par_map(x, |&v| v.sqrt())

// Type conversion: i32 → f64
par_map(ints, |&v| v as f64)
```

#### `par_map2(a, b, f)`: two-input parallel map

```rust
// Element-wise addition
par_map2(a, b, |&x, &y| x + y)
```

#### `par_map3(a, b, c, f)`: three-input parallel map

```rust
// Fused multiply-add: a * b + c
par_map3(a, b, c, |&x, &y, &z| x * y + z)
```

### Matrix and Array

#### `with_r_matrix(nrow, ncol, f)`: column-wise parallel fill

Each column is a natural chunk (contiguous in R's column-major layout).
The closure receives `(column_slice, col_idx)`.

```rust
with_r_matrix(100, 50, |col: &mut [f64], col_idx: usize| {
    for (row, slot) in col.iter_mut().enumerate() {
        *slot = (row + col_idx * 1000) as f64;
    }
});
```

#### `with_r_array(dims, f)`: slab-wise parallel fill

For dims `[d0, d1, ..., dN]`, each slab has `d0 * d1 * ... * d(N-1)` elements.
The closure receives `(slab_slice, slab_idx)` where `slab_idx` is the index along
the last dimension.

```rust
// 2×3×4 array: 4 slabs of 6 elements each
with_r_array([2, 3, 4], |slab: &mut [f64], slab_idx: usize| {
    for (i, val) in slab.iter_mut().enumerate() {
        *val = (slab_idx * 100 + i) as f64;
    }
});
```

#### Typed wrappers: `new_r_matrix`, `new_r_array`

Same as above but return `RMatrix<T>` / `RArray<T, NDIM>` instead of raw SEXP.

### Reduction

```rust
use miniextendr_api::rayon_bridge::reduce;

let sum = reduce::sum(&data);         // Parallel sum → R scalar
let min = reduce::min(&data);         // Parallel min
let max = reduce::max(&data);         // Parallel max
let mean = reduce::mean(&data);       // Parallel mean
let sum_int = reduce::sum_int(&ints); // Integer sum
```

### Performance Utilities

```rust
use miniextendr_api::rayon_bridge::perf;

perf::num_threads()      // Number of Rayon threads
perf::in_rayon_thread()  // Are we in a Rayon thread?
perf::thread_index()     // Current thread index (if in pool)
```

### `.collect_r()`: parallel iterator to R vector

The `ParCollectR` extension trait adds `.collect_r()` to every indexed parallel
iterator. It allocates an R vector on the main thread, then fills it in parallel
using Rayon's `zip`. No intermediate allocation is needed.

```rust
use miniextendr_api::rayon_bridge::{rayon::prelude::*, ParCollectR};

#[miniextendr]
fn parallel_sqrt(x: &[f64]) -> SEXP {
    x.par_iter().map(|&v| v.sqrt()).collect_r()
}

#[miniextendr]
fn index_fill(n: i32) -> SEXP {
    (0..n).into_par_iter().map(|i| (i as f64).sin()).collect_r()
}
```

**Which iterators are indexed?** Most of them:

| Pattern | Indexed? | `.collect_r()` works? |
|---------|----------|-----------------------|
| `slice.par_iter().map(...)` | Yes | Yes |
| `vec.into_par_iter().map(...)` | Yes | Yes |
| `(0..n).into_par_iter()` | Yes | Yes |
| `.enumerate()`, `.zip()`, `.take()`, `.skip()` | Yes | Yes |
| `.filter(...)` | **No** | No (use `par_collect_sexp`) |
| `.flat_map(...)` | **No** | No (use `par_collect_sexp`) |

### `par_collect_sexp()`: non-indexed fallback

For iterators that lose their index (`.filter()`, `.flat_map()`, `.par_bridge()`),
use this function. It collects to an intermediate `Vec<T>` then converts to R.

```rust
use miniextendr_api::rayon_bridge;

#[miniextendr]
fn parallel_filter(x: &[f64]) -> SEXP {
    // .filter() loses index - can't use .collect_r()
    rayon_bridge::par_collect_sexp(
        x.par_iter().filter(|&&v| v > 0.0).copied()
    )
}
```

### `.collect()` with `Sendable<SEXP>` Return Type

If you prefer standard `.collect()` syntax, return `Sendable<SEXP>` from your function.
The return type drives type inference so no turbofish is needed:

```rust
use miniextendr_api::worker::Sendable;

#[miniextendr]
fn parallel_sqrt(x: &[f64]) -> Sendable<SEXP> {
    x.par_iter().map(|&v| v.sqrt()).collect()
}
```

`Sendable<SEXP>` implements both `IntoR` (works as `#[miniextendr]` return type) and
`From<Sendable<SEXP>> for SEXP` (for manual unwrapping with `.into()`).

This path collects to an intermediate `Vec<T>` before converting to R.
For zero-copy performance, use `.collect_r()`.

### Choosing a Collection Strategy

| Strategy | Allocation | Requires | Best For |
|----------|-----------|----------|----------|
| `.collect_r()` | Zero-copy into R | Indexed iterator | Max performance |
| `-> Sendable<SEXP>` + `.collect()` | Vec + copy to R | Any par iterator | Standard rayon idiom |
| `par_map(x, f)` | Zero-copy into R | Input slice | Simple 1:1 transforms |
| `par_collect_sexp(iter)` | Vec + copy to R | Any par iterator | Filters, flat maps |
| `-> Vec<T>` return | Vec + copy to R | Nothing special | Simplest code |

### Parallel Collection (Vec\<T\>)

For operations that don't fit the chunk model (filtering, variable-length output),
you can also just return `Vec<T>` and let miniextendr convert to R:

```rust
#[miniextendr]
fn parallel_pipeline(x: &[f64]) -> Vec<f64> {
    x.par_iter()
        .filter(|&&v| v > 0.0)
        .map(|&v| v.log2())
        .collect()
}
```

## Patterns

### Pattern 1: `.collect_r()` (Best Default)

**Use when:** Any parallel transform where output length equals input length

**Performance:** Best: zero intermediate allocation, writes directly into R memory

```rust
use miniextendr_api::rayon_bridge::{rayon::prelude::*, ParCollectR};

#[miniextendr]
fn parallel_sqrt(x: &[f64]) -> SEXP {
    x.par_iter().map(|&v| v.sqrt()).collect_r()
}

#[miniextendr]
fn generate_sequence(n: i32) -> SEXP {
    (0..n).into_par_iter().map(|i| (i as f64).sin()).collect_r()
}
```

### Pattern 2: `par_map` / `with_r_vec` (Closure Style)

**Use when:** You prefer a closure-based API, or need per-chunk state (e.g., RNG)

**Performance:** Same as `.collect_r()`: zero copy, deterministic chunks

```rust
// par_map: one-liner for element-wise transforms
#[miniextendr]
fn parallel_sqrt(x: &[f64]) -> SEXP {
    par_map(x, |&v| v.sqrt())
}

// with_r_vec: access to chunk offset for per-chunk state
#[miniextendr]
fn generate_random(n: i32, seed: i64) -> SEXP {
    with_r_vec(n as usize, |chunk: &mut [f64], offset| {
        let mut rng = ChaChaRng::seed_from_u64(seed as u64 + offset as u64);
        for slot in chunk.iter_mut() { *slot = rng.gen(); }
    })
}
```

### Pattern 3: `par_collect_sexp` / `Vec<T>` (Flexible)

**Use when:** Variable-length output (filtering, flat-mapping)

**Performance:** Moderate: one extra allocation (intermediate `Vec<T>`)

```rust
// par_collect_sexp: returns SEXP directly
#[miniextendr]
fn parallel_filter(x: &[f64]) -> SEXP {
    rayon_bridge::par_collect_sexp(
        x.par_iter().filter(|&&v| v > 0.0).copied()
    )
}

// Or just return Vec<T> - miniextendr converts automatically
#[miniextendr]
fn parallel_filter_vec(x: &[f64]) -> Vec<f64> {
    x.par_iter().filter(|&&v| v > 0.0).map(|&v| v.log2()).collect()
}
```

### Pattern 4: Parallel Reduction

**Use when:** Computing aggregates (sum, mean, min, max)

**Performance:** Best for aggregations

```rust
use miniextendr_api::rayon_bridge::reduce;

#[miniextendr]
fn fast_sum(x: &[f64]) -> SEXP {
    reduce::sum(x)
}
```

## RNG Reproducibility

Chunk boundaries are **deterministic** for a given `(length, thread_count)`. This means
you can seed per-chunk RNG from the `offset` parameter and get reproducible results
regardless of thread scheduling.

### How It Works

`with_r_vec` splits the output into `len / (num_threads * 4)` sized chunks using
Rayon's `par_chunks_mut`. The chunk boundaries are fixed: chunk 0 always starts at
index 0, chunk 1 always starts at `chunk_size`, etc. Only the **scheduling order**
varies between runs, not the boundaries.

This means: if you derive each chunk's RNG seed from its `offset`, the same chunk
always gets the same seed, producing the same random values in those positions.

### Pattern: Seed-per-chunk

```rust
use rand::SeedableRng;
use rand_chacha::ChaChaRng;
use rand::Rng;

#[miniextendr]
fn reproducible_random(len: i32, seed: i64) -> SEXP {
    with_r_vec(len as usize, |chunk: &mut [f64], offset| {
        // Deterministic seed derived from base seed + chunk offset
        let mut rng = ChaChaRng::seed_from_u64(seed as u64 + offset as u64);
        for slot in chunk.iter_mut() {
            *slot = rng.gen();
        }
    })
}
```

### Why This Works

```text
Thread count = 4, length = 1000, chunk_size = 62

Run 1 (scheduling order: T2, T0, T3, T1):
  T2 fills chunk[2] (offset=124) with seed=42+124  ← same values
  T0 fills chunk[0] (offset=0)   with seed=42+0    ← same values
  T3 fills chunk[3] (offset=186) with seed=42+186  ← same values
  T1 fills chunk[1] (offset=62)  with seed=42+62   ← same values

Run 2 (scheduling order: T0, T1, T2, T3):
  T0 fills chunk[0] (offset=0)   with seed=42+0    ← same values ✓
  T1 fills chunk[1] (offset=62)  with seed=42+62   ← same values ✓
  T2 fills chunk[2] (offset=124) with seed=42+124  ← same values ✓
  T3 fills chunk[3] (offset=186) with seed=42+186  ← same values ✓
```

The output vector is bitwise identical between runs (given same length and thread count).

### Important: Thread Count Sensitivity

Chunk boundaries depend on `rayon::current_num_threads()`. Different machines with
different core counts will produce different chunk boundaries and therefore different
random sequences. This is fine for most use cases (simulation, Monte Carlo), but if
you need cross-machine reproducibility, pin the thread pool size:

```rust
// Pin to exactly 4 threads regardless of machine
rayon::ThreadPoolBuilder::new()
    .num_threads(4)
    .build_global()
    .unwrap();
```

### Do NOT Use R's RNG in Parallel

R's RNG (`RRng`, `Rf_runif`, etc.) calls R APIs, which **panic** on Rayon threads.
Use a Rust RNG crate (`rand`, `rand_chacha`) instead:

```rust
// WRONG: R's RNG calls R APIs - panics in parallel
with_r_vec(len, |chunk, _| {
    let mut rng = RRng::new();  // PANICS!
    for slot in chunk { *slot = rng.uniform_f64(); }
});

// CORRECT: Rust RNG is thread-safe
with_r_vec(len, |chunk, offset| {
    let mut rng = ChaChaRng::seed_from_u64(seed + offset as u64);
    for slot in chunk { *slot = rng.gen(); }
});
```

## Performance

### When to Use Rayon

**Good Use Cases:**
- CPU-intensive transformations (sqrt, log, trig functions)
- Large datasets (>10,000 elements)
- Embarrassingly parallel problems
- Reductions (sum, min, max, mean)
- Operations with minimal R API interaction

**Be Careful:**
- Frequent R API calls (each call has channel overhead)
- Small datasets (<1,000 elements, where overhead exceeds gains)
- Operations R can vectorize efficiently

**Avoid:**
- Calling R for every element in a tight loop
- Parallel evaluation of R code (R is single-threaded!)
- Simple operations R handles well (addition, multiplication)

### Optimization Tips

1. **Use `.collect_r()`** for par iterator chains: zero-copy into R memory
2. **Use `par_map`** for simple element-wise transforms (same performance, closure style)
3. **Use `with_r_vec`** when you need per-chunk state (RNG, accumulators)
4. **Use `par_collect_sexp`** only for non-indexed iterators (`.filter()`, `.flat_map()`)
5. **Profile First**: Measure before assuming parallelism helps
6. **Consider R Alternatives**: Vectorized R operations are fast

## Safety

### Thread Safety Invariants

**Safe Patterns:**
```rust
// par_map: framework handles everything
par_map(x, |&v| v.sqrt());

// with_r_vec: chunk-based, pure Rust in closure
with_r_vec(n, |chunk, offset| {
    for (i, slot) in chunk.iter_mut().enumerate() {
        *slot = ((offset + i) as f64).sqrt();
    }
});
```

**Unsafe Patterns:**
```rust
// WRONG: R API in closure - PANICS
with_r_vec(n, |chunk, _| {
    let sexp = unsafe { ffi::Rf_ScalarReal(1.0) };  // CRASH!
});

// WRONG: with_r_thread inside closure - PANICS
with_r_vec(n, |chunk, _| {
    with_r_thread(|| { ... });  // PANICS! Not on worker thread
});
```

### Memory Safety

- **GC Protection**: Pre-allocated SEXPs are `Rf_protect`ed during parallel writes
- **No Concurrent R Access**: All R operations happen before/after parallel section
- **RAII Guards**: `WorkerUnprotectGuard` ensures cleanup even on panic

## Examples

### Example 1: Parallel Normalization

```rust
use miniextendr_api::rayon_bridge::*;
use rayon::prelude::*;

#[miniextendr]
fn parallel_normalize(x: &[f64]) -> SEXP {
    let (sum, sum_sq, count) = x.par_iter().fold(
        || (0.0, 0.0, 0usize),
        |(s, sq, c), &val| (s + val, sq + val * val, c + 1)
    ).reduce(
        || (0.0, 0.0, 0),
        |(s1, sq1, c1), (s2, sq2, c2)| (s1 + s2, sq1 + sq2, c1 + c2)
    );

    let mean = sum / count as f64;
    let sd = ((sum_sq / count as f64) - mean * mean).sqrt();

    par_map(x, |&v| (v - mean) / sd)
}
```

### Example 2: Matrix Fill by Column

```rust
#[miniextendr]
fn identity_matrix(n: i32) -> SEXP {
    let n = n as usize;
    with_r_matrix(n, n, |col: &mut [f64], col_idx| {
        // Each column gets a 1.0 at the diagonal position
        col[col_idx] = 1.0;
    })
}
```

### Example 3: Parallel Filtering

```rust
#[miniextendr]
fn parallel_filter_positive(x: &[f64]) -> Vec<f64> {
    x.par_iter()
        .copied()
        .filter(|&v| v > 0.0)
        .collect()
}
```

### Example 4: Two-Input Operations

```rust
#[miniextendr]
fn euclidean_distance(x: &[f64], y: &[f64]) -> SEXP {
    par_map2(x, y, |&a, &b| (a - b).powi(2))
}
```

## Troubleshooting

### Error: "with_r_thread called outside of run_on_worker context"

**Solution:** With the `worker-thread` feature, Rayon integration only works
inside `#[miniextendr]` functions (which use `run_on_worker`). Without the
feature, `with_r_thread` is an inline stub that requires `miniextendr_runtime_init()`
to have been called.

### Slow Performance

**Check:**
1. Dataset size (< 10K elements might not benefit)
2. Number of threads (`perf::num_threads()`)
3. Computation cost per element (must justify thread overhead)

### Debugging

```rust
use miniextendr_api::rayon_bridge::perf;

eprintln!("Rayon threads: {}", perf::num_threads());
eprintln!("In Rayon thread: {}", perf::in_rayon_thread());
eprintln!("Thread index: {:?}", perf::thread_index());
```

## See Also

- [SAFETY.md](SAFETY.md): thread safety invariants
- [ENTRYPOINT.md](ENTRYPOINT.md): worker initialization requirements
- `miniextendr-bench/benches/rayon.rs`: performance benchmarks
