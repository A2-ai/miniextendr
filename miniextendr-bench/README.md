# miniextendr-bench

Benchmarks for `miniextendr` conversions and interop behavior.

This crate depends on `miniextendr-engine` to embed R, so it is intended for
local development and performance investigations (not publishing).

## Run benchmarks

From the repo root:

```sh
just bench --bench translate
# or:
cargo bench --manifest-path=miniextendr-bench/Cargo.toml --bench translate
```

## Notes

- Requires R installed and available on PATH.
- Uses `divan` as the benchmark harness.
- See `miniextendr-bench/benches/` for the full target list (including `trait_abi`), and `miniextendr-bench/src/bench_plan/` for a high-level plan.

## What is measured

Some selected targets:

- `translate`: string extraction costs
- `trait_abi`: mx_erased trait dispatch (vtable query + method calls)

Many others cover conversions, FFI calls, ExternalPtr, ALTREP, worker routing, and more.

## Publishing to CRAN

This crate is **not** part of any R package build and should never be shipped
in a CRAN tarball. It embeds R and is purely for developer benchmarking.

## Benchmarking Environment

For reproducible benchmark results, document your environment:

### Recommended Setup

1. **Dedicated machine or VM** - Avoid running other workloads during benchmarks
2. **Fixed CPU frequency** - Disable turbo boost and frequency scaling
3. **Isolated cores** - Use `taskset` or `cpuset` to pin benchmarks to specific cores

### Environment Capture

Before running benchmarks, capture your environment:

```sh
# System info
uname -a
cat /proc/cpuinfo | grep "model name" | head -1  # Linux
sysctl -n machdep.cpu.brand_string                # macOS

# R version
R --version | head -1
Rscript -e "sessionInfo()"

# Rust version
rustc --version
cargo --version

# miniextendr version
grep '^version' ../Cargo.toml | head -1
```

### Running Benchmarks Consistently

```sh
# Run all benchmarks
just bench

# Run specific benchmark
just bench --bench worker

# Compare against baseline (divan feature)
cargo bench --bench worker -- --save-baseline main
# ... make changes ...
cargo bench --bench worker -- --baseline main
```

### Interpreting Results

The benchmarks use [divan](https://github.com/nvzqz/divan) which reports:
- **fastest**: Best observed time (useful for latency comparisons)
- **slowest**: Worst observed time (check for outliers)
- **median**: Typical performance
- **allocs**: Number of allocations (if measured)

### Environment Variables

| Variable | Effect |
|----------|--------|
| `R_HOME` | R installation to use |
| `DIVAN_SKIP_SLOW` | Skip slow benchmarks (set to `1`) |
| `RAYON_NUM_THREADS` | Thread count for parallel benchmarks |

### Benchmark Categories

| Benchmark | Focus |
|-----------|-------|
| `worker` | Worker thread routing overhead |
| `ffi_calls` | Raw R FFI call latency |
| `rffi_checked` | Checked FFI wrapper overhead |
| `into_r` / `from_r` | Type conversion costs |
| `strings` | String encoding/decoding |
| `coerce` | R coercion overhead |
| `altrep` | ALTREP materialization |
| `trait_abi` | Cross-package trait dispatch |
| `externalptr` | ExternalPtr creation/access |
| `preserve` | GC protection overhead |

## Maintainer

- Keep benchmarks aligned with current conversion paths in `miniextendr-api`.
- Update any fixture sizes or data if performance goals change.
- Re-run benchmarks after any substantial FFI or conversion changes.
- Ensure `miniextendr-engine` remains the only embedding dependency.
- Document benchmark environment when publishing results.
