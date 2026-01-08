# Rayon Integration Guide

Miniextendr provides seamless integration with [Rayon](https://docs.rs/rayon) for parallel computation in R packages. This enables writing high-performance parallel code while maintaining R's safety guarantees.

## Table of Contents

- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [API Overview](#api-overview)
- [Patterns](#patterns)
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
use miniextendr_api::rayon_bridge::{RVec, rayon::prelude::*};

#[miniextendr]
fn parallel_sqrt(x: &[f64]) -> RVec<f64> {
    x.par_iter().map(|&val| val.sqrt()).collect()
}
```

### Zero-Copy Example (Maximum Performance)

```rust
use miniextendr_api::rayon_bridge::*;
use rayon::prelude::*;

#[miniextendr]
fn parallel_sqrt_fast(x: &[f64]) -> SEXP {
    with_r_vec::<f64, _>(x.len(), |output| {
        output.par_iter_mut()
            .zip(x.par_iter())
            .for_each(|(out, &inp)| *out = inp.sqrt());
    })
}
```

## Architecture

### Design Philosophy

**Rust computation: Parallel on Rayon threads (normal 2MB stacks)**
**R API calls: Serial on main thread (via `with_r_thread`)**

### Thread Model

```text
┌─────────────────────────────────────────┐
│   Rayon Thread Pool (2MB stacks)        │
│                                         │
│   Thread 1   Thread 2   Thread 3        │
│      ↓          ↓          ↓            │
│   [Rust]    [Rust]    [Rust]            │ ← Parallel computation
│      ↓          ↓          ↓            │
│   with_r    with_r    with_r            │ ← Need R API?
└──────┬──────────┬──────────┬────────────┘
       │          │          │
       └──────────┴──────────┘
                  ↓
       ┌──────────────────────┐
       │  Main R Thread       │
       │  (channel based)     │ ← Sequential R operations
       │                      │
       │  Rf_allocVector()    │
       │  Rf_ScalarReal()     │
       │  R_eval()            │
       └──────────┬───────────┘
                  ↓
       Results sent back to Rayon threads
```

### Key Insights

1. **Normal Stacks**: Rayon threads use Rust's default 2MB stacks, not R's large stacks
2. **No Stack Checking Disabled**: R's stack checking remains enabled on the main thread
3. **Channel-Based**: Uses the existing `with_r_thread` channel infrastructure
4. **Zero Copy**: Pre-allocation patterns write directly into R memory

## API Overview

### Core Functions

#### `with_r_thread` - Execute R Code on Main Thread

Routes R API calls from Rayon threads to the main thread.

```rust
let sexp = with_r_thread(|| unsafe {
    ffi::Rf_ScalarInteger(42)
});
```

### Pre-Allocation Functions (Zero-Copy)

#### `with_r_vec<T>` - Pre-allocate Vector

```rust
pub fn with_r_vec<T: RNativeType, F>(len: usize, f: F) -> SEXP
where F: FnOnce(&mut [T])
```

**Example:**
```rust
let r_vec = with_r_vec::<f64, _>(1000, |output| {
    output.par_iter_mut()
        .enumerate()
        .for_each(|(i, slot)| *slot = (i as f64).sqrt());
});
```

### Collection Type

#### `RVec<T>` - Parallel Collection

Implements `FromParallelIterator` for use with `.collect()`.

```rust
let results: RVec<f64> = (0..1000)
    .into_par_iter()
    .map(|i| (i as f64).sqrt())
    .collect();  // Parallel collection

let r_vec = results.into_sexp();  // Convert to R on main thread
```

### Reduction Functions

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

## Patterns

### Pattern 1: Zero-Copy Parallel Fill (Maximum Performance)

**Use when:** Transforming R vector element-wise

**Performance:** Best - no intermediate allocation

```rust
#[miniextendr]
fn parallel_sqrt(x: &[f64]) -> SEXP {
    with_r_vec::<f64, _>(x.len(), |output| {
        output.par_iter_mut()
            .zip(x.par_iter())
            .for_each(|(out, &inp)| *out = inp.sqrt());
    })
}
```

### Pattern 2: Collect to RVec (Flexible)

**Use when:** Complex transformations or multiple steps

**Performance:** Moderate - one extra allocation

```rust
#[miniextendr]
fn parallel_pipeline(x: &[f64]) -> RVec<f64> {
    x.par_iter()
        .filter(|&&v| v > 0.0)      // Filter
        .map(|&v| v.log2())          // Transform
        .collect()                    // Collect
}
```

### Pattern 3: Parallel Reduction (Fast)

**Use when:** Computing aggregates (sum, mean, min, max)

**Performance:** Best for aggregations

```rust
use miniextendr_api::rayon_bridge::reduce;

#[miniextendr]
fn parallel_stats(x: &[f64]) -> Vec<f64> {
    vec![
        reduce::sum(x),
        reduce::mean(x),
        reduce::min(x),
        reduce::max(x),
    ]
}
```

### Pattern 4: Chunked Processing (Best Cache Locality)

**Use when:** Processing large datasets with complex per-chunk logic

**Performance:** Good - reduces synchronization, better cache usage

```rust
#[miniextendr]
fn parallel_chunked(x: &[f64]) -> RVec<f64> {
    x.par_chunks(1000)
        .flat_map(|chunk| {
            // Pure Rust processing of chunk (no R calls!)
            chunk.iter()
                .filter(|&&v| v.is_finite())
                .map(|&v| v.powi(2))
        })
        .collect()
}
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
- Small datasets (<1,000 elements - overhead > gains)
- Operations R can vectorize efficiently

**Avoid:**
- Calling R for every element in a tight loop
- Parallel evaluation of R code (R is single-threaded!)
- Simple operations R handles well (addition, multiplication)

### Typical Performance

| Operation | Sequential | Parallel (4 cores) | Speedup |
|-----------|------------|-------------------|---------|
| `sqrt(x)` (1M elements) | 5ms | 1.5ms | 3.3x |
| `sum(x)` (10M elements) | 10ms | 3ms | 3.3x |
| `x^2 + y^2` (1M each) | 12ms | 3.5ms | 3.4x |

### Optimization Tips

1. **Minimize R Calls**: Batch processing, compute in Rust, convert once
2. **Use Zero-Copy**: `with_r_vec<T>` avoids extra allocations
3. **Chunk Wisely**: 100-10,000 elements per chunk depending on cost
4. **Profile First**: Measure before assuming parallelism helps
5. **Consider R Alternatives**: Vectorized R operations are fast

## Safety

### Thread Safety Invariants

**Safe Patterns:**
```rust
// Compute in parallel, create R object once
let result: Vec<f64> = data.par_iter().map(|x| x.sqrt()).collect();
let r_vec = with_r_thread(move || create_r_vector(&result));

// Write directly to pre-allocated R memory
with_r_vec::<f64, _>(n, |output| {
    output.par_iter_mut().for_each(|slot| *slot = compute());
});
```

**Unsafe Patterns:**
```rust
// WRONG: Direct R call from Rayon thread
data.par_iter().map(|x| unsafe {
    ffi::Rf_ScalarInteger(*x)  // CRASH! Wrong thread!
});

// WRONG: Concurrent R access
let sexp = create_once();
data.par_iter().for_each(|x| unsafe {
    ffi::INTEGER_ELT(sexp, *x)  // DATA RACE!
});
```

### Memory Safety

- **GC Protection**: Pre-allocated SEXPs remain valid during parallel writes
- **No Concurrent R Access**: All R operations serialized via `with_r_thread`
- **Ownership**: `with_r_thread` enforces proper ownership transfer

## Examples

### Example 1: Parallel Vector Transformation

```rust
use miniextendr_api::prelude::*;
use miniextendr_api::rayon_bridge::*;
use rayon::prelude::*;

#[miniextendr]
fn parallel_normalize(x: &[f64]) -> SEXP {
    // Compute mean and SD in parallel
    let (sum, sum_sq, count) = x.par_iter().fold(
        || (0.0, 0.0, 0usize),
        |(s, sq, c), &val| (s + val, sq + val * val, c + 1)
    ).reduce(
        || (0.0, 0.0, 0),
        |(s1, sq1, c1), (s2, sq2, c2)| (s1 + s2, sq1 + sq2, c1 + c2)
    );

    let mean = sum / count as f64;
    let variance = (sum_sq / count as f64) - mean * mean;
    let sd = variance.sqrt();

    // Normalize in parallel, write to R vector
    with_r_vec::<f64, _>(x.len(), |output| {
        output.par_iter_mut()
            .zip(x.par_iter())
            .for_each(|(out, &inp)| *out = (inp - mean) / sd);
    })
}
```

### Example 2: Parallel Matrix Operations

```rust
#[miniextendr]
fn parallel_matrix_multiply(a: &[f64], b: &[f64], n: i32) -> SEXP {
    let n = n as usize;

    with_r_vec::<f64, _>(n * n, |output| {
        output.par_chunks_mut(n)
            .enumerate()
            .for_each(|(i, row)| {
                for j in 0..n {
                    let mut sum = 0.0;
                    for k in 0..n {
                        sum += a[i * n + k] * b[k * n + j];
                    }
                    row[j] = sum;
                }
            });
    })
}
```

### Example 3: Parallel Filtering

```rust
#[miniextendr]
fn parallel_filter_positive(x: &[f64]) -> RVec<f64> {
    x.par_iter()
        .copied()
        .filter(|&v| v > 0.0)
        .collect()
}
```

### Example 4: Parallel String Processing

```rust
#[miniextendr]
fn parallel_uppercase(x: Vec<String>) -> Vec<String> {
    x.into_par_iter()
        .map(|s| s.to_uppercase())
        .collect()
}
```

## Chunk Size Selection

```rust
// Good: Large enough to amortize overhead
data.par_chunks(1000).for_each(|chunk| { ... });

// Bad: Too small, too much overhead
data.par_chunks(10).for_each(|chunk| { ... });

// Bad: Too large, not enough parallelism
data.par_chunks(1_000_000).for_each(|chunk| { ... });
```

**Rule of thumb:** Chunk size should result in 10-1000 chunks total.

## Troubleshooting

### Error: "with_r_thread called outside of run_on_worker context"

**Solution:** Rayon integration only works inside `#[miniextendr]` functions
(which use `run_on_worker`).

### Slow Performance

**Check:**
1. Dataset size (< 10K elements might not benefit)
2. Chunk size (adjust with `.par_chunks`)
3. R call frequency (minimize calls to `with_r_thread`)
4. Number of threads (`rayon::current_num_threads()`)

### Debugging

```rust
use miniextendr_api::rayon_bridge::perf;

eprintln!("Rayon threads: {}", perf::num_threads());
eprintln!("In Rayon thread: {}", perf::in_rayon_thread());
eprintln!("Thread index: {:?}", perf::thread_index());
```

## Summary

Miniextendr's Rayon integration provides:

- **Zero configuration** - Just enable the feature
- **Safe** - Compiler prevents R calls from wrong threads
- **Fast** - Zero-copy pre-allocation patterns
- **Flexible** - Multiple APIs for different use cases
- **Idiomatic** - Feels like standard Rayon code

The key rule: **Do not call R APIs inside parallel iterators.** Do R work
before or after the parallel section.

## See Also

- [SAFETY.md](SAFETY.md) - Thread safety invariants
- [ENTRYPOINT.md](ENTRYPOINT.md) - Worker initialization requirements
- `miniextendr-bench/benches/rayon.rs` - Performance benchmarks
