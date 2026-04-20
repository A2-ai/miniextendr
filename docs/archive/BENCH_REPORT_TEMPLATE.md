# Benchmark Report

## Environment

| Field | Value |
|-------|-------|
| **Date** | YYYY-MM-DD |
| **Commit** | `abc1234` (branch: `main`) |
| **Rust** | rustc 1.XX.0 (hash YYYY-MM-DD) |
| **Cargo** | cargo 1.XX.0 |
| **R** | R X.Y.Z |
| **OS** | macOS / Linux / Windows (arch) |
| **CPU** | Model name, core count |
| **RAM** | XX GB |

## Commands Run

```bash
# Core benchmarks (default features)
just bench-core

# Feature-gated benchmarks
just bench-features

# Full suite
just bench-full

# R-side benchmarks (requires rpkg installed)
just bench-r

# Macro compile-time benchmark
just bench-compile

# Lint scan benchmark
just bench-lint
```

Feature flags used: `--features rayon,rand,...` (list non-default features if any).

## Baseline Comparison

```
Baseline commit: abc1234 (YYYY-MM-DD)
Current commit:  def5678 (YYYY-MM-DD)
Drift threshold: 20%
```

### Regressions

| Benchmark | Baseline (ns) | Current (ns) | Delta |
|-----------|---------------|--------------|-------|
| group::name(args) | 123.4 | 234.5 | +90.1% |

### Improvements

| Benchmark | Baseline (ns) | Current (ns) | Delta |
|-----------|---------------|--------------|-------|
| group::name(args) | 234.5 | 123.4 | -47.4% |

### No Change

_Benchmarks within the drift threshold are not listed._

## Rust-Side Results (divan)

### Core (A1-A9)

| Group | Benchmark | Args | Median | Mean | Notes |
|-------|-----------|------|--------|------|-------|
| into_r | i32_vec | 1000 | X ns | X ns | |
| from_r | f64_scalar | - | X ns | X ns | |
| unwind_protect | success_path | - | X ns | X ns | |
| worker_thread | roundtrip | - | X ns | X ns | |
| gc_protect | protect_unprotect | 1 | X ns | X ns | |
| altrep | integer_elt | 1000 | X ns | X ns | |
| panic_telemetry | fire_with_hook | - | X ns | X ns | |
| raw_access | integer_safe_vs_raw | 1000 | X ns | X ns | |
| typed_list | homogeneous_numeric | 3 | X ns | X ns | |

### Features (B1-B2)

| Group | Benchmark | Args | Median | Mean | Notes |
|-------|-----------|------|--------|------|-------|
| dataframe | transpose | 1000 | X ns | X ns | |
| lint_scan | full_scan::small_10 | - | X ns | X ns | |

### Compile-Time (B3)

| Scenario | Cold (ms) | Warm (ms) | Incremental (ms) |
|----------|-----------|-----------|-------------------|
| fn-heavy (20×50 fns) | X | X | X |
| impl-heavy (20×10 types) | X | X | X |
| trait-heavy (20×5 traits) | X | X | X |
| mixed (20×20+5+2) | X | X | X |

## R-Side Results (bench)

### Class Dispatch (D1)

| Class System | Operation | median | mem_alloc | Notes |
|-------------|-----------|--------|-----------|-------|
| R6 | method call | X µs | X B | |
| S3 | generic dispatch | X µs | X B | |
| S4 | generic dispatch | X µs | X B | |
| S7 | generic dispatch | X µs | X B | |
| Env | $ dispatch | X µs | X B | |

### ALTREP vs SEXP (D2)

| Type | Size | Eager (µs) | Lazy (µs) | Speedup | Notes |
|------|------|------------|-----------|---------|-------|
| integer | 1K | X | X | Xx | |
| integer | 1M | X | X | Xx | |
| real | 1M | X | X | Xx | |

### Vctrs Protocol (D3)

| Operation | Type | median | Notes |
|-----------|------|--------|-------|
| vec_c | Percent | X µs | |
| format | Point | X µs | |

### Dots Overhead (D4)

| Scenario | n_args | median | Notes |
|----------|--------|--------|-------|
| untyped | 0 | X µs | |
| untyped | 20 | X µs | |
| typed_list | 5 | X µs | |

## Analysis

_Summarize key findings: bottlenecks, scaling characteristics, regressions worth investigating, and any action items._

## Reproducing

```bash
# Save a new baseline
just bench-save

# Compare against previous baseline
just bench-drift --threshold=20

# View baseline metadata
just bench-info

# Run R benchmarks interactively
Rscript rpkg/tests/testthat/bench-class-dispatch.R
Rscript rpkg/tests/testthat/bench-altrep-vs-sexp.R
Rscript rpkg/tests/testthat/bench-vctrs-protocol.R
Rscript rpkg/tests/testthat/bench-dots.R
```

Baselines are stored in `miniextendr-bench/baselines/` as:
- `bench-TIMESTAMP.txt` - raw divan output
- `bench-TIMESTAMP.csv` - machine-readable (timestamp, target, group, name, args, median_ns, unit, mean_ns)
- `bench-TIMESTAMP.meta` - environment metadata (commit, rustc, OS, R version)
