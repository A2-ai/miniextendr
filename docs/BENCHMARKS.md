# Performance Benchmarks

Baseline measurements for miniextendr's runtime overhead on Apple M3 Max (macOS, Rust 1.93, R 4.5).

## Quick Reference

| Subsystem | Operation | Median | Notes |
|-----------|-----------|--------|-------|
| **Worker thread** | round-trip | 4-5 us | `run_on_worker` channel hop |
| **Worker thread** | `with_r_thread` (main) | 10 ns | already on main thread |
| **Unwind protect** | `with_r_unwind_protect` | 31-35 ns | overhead vs direct call |
| **Unwind protect** | nested 5 layers | 170 ns | linear scaling |
| **catch_unwind** | success path | 0.5 ns | no panic |
| **catch_unwind** | panic caught | 5 us | panic + catch overhead |
| **ExternalPtr** | create (8 B) | 65 ns | vs Box 12 ns (5x) |
| **ExternalPtr** | create (64 KB) | 727 ns | vs Box 512 ns (1.4x) |
| **Trait ABI** | vtable query | ~1 ns | cache-hot, 2 or 5 methods |
| **Trait ABI** | dispatch (1 method) | 53-62 ns | full view dispatch |
| **Trait ABI** | dispatch (all 5) | 417 ns | multi-method hot loop |
| **Panic telemetry** | RwLock read (no hook) | 1.5 ns | hot-path cost |
| **Panic telemetry** | fire with hook | 65 ns | |
| **R allocator** | small (8 B) | 71 ns | vs system 16 ns (4.4x) |
| **R allocator** | large (64 KB) | 867 ns | vs system 521 ns (1.7x) |

## Type Conversions

### Rust to R (`into_sexp`)

| Type | Size | Median | Notes |
|------|------|--------|-------|
| i32 | 1 | 33-42 ns | scalar |
| i32 | 1K | 370 ns | |
| i32 | 1M | 675 us | |
| f64 | 1M | 1.6 ms | |
| String | 1M | 276 ms | CHARSXP allocation dominates |
| Option\<i32\> 50% NA | 1M | 934 us | |
| i64 (smart) | scalar | 40-43 ns | INTSXP or REALSXP |

### R to Rust (`try_from_sexp`)

| Type | Size | Median | Notes |
|------|------|--------|-------|
| i32 scalar | 1 | ~20 ns | |
| f64 vec | 256 | ~45 ns | memcpy path |
| String | 1 | ~75 ns | UTF-8 re-encoding |

### Strict Mode

Negligible overhead for scalar conversions (~2-5 ns). Vec\<i64\> at 10K: strict 7.9 us vs normal 7.1 us (~11% overhead from runtime range checks).

## ALTREP

| Operation | Size | ALTREP | Plain | Ratio |
|-----------|------|--------|-------|-------|
| element access (elt) | 1 | 200 ns | 9 ns | 22x |
| DATAPTR materialization | 64K | 16-19 us | 9 ns | — |
| full scan (elt loop) | 64K | 5 ms | 258 us | 19x |
| full scan (DATAPTR) | 64K | 18 us | 9 ns | — |

### Guard Modes (64K elements, full scan)

| Guard | Median |
|-------|--------|
| `unsafe` | 16 ms |
| `rust_unwind` (default) | 16 ms |
| `r_unwind` | 20 ms |
| plain INTSXP | 258 us |

`unsafe` and `rust_unwind` are equivalent. `r_unwind` adds ~25% overhead due to `R_UnwindProtect` per callback.

### String ALTREP (64K strings)

| Operation | Median |
|-----------|--------|
| create | 2.6 ms |
| elt access | 2.6 ms |
| force materialize (DATAPTR_RO) | 6.8 ms |
| plain STRSXP elt | 4.8 ms |

### Zero-Allocation (constant real)

| Operation | Size | Median |
|-----------|------|--------|
| create constant | any | 200 ns |
| constant elt | any | 510 ns |
| constant full scan | 64K | 16.8 ms |
| vec-backed full scan | 64K | 5.1 ms |

## Connections

| Operation | Size | Median |
|-----------|------|--------|
| build + open | — | 542 ns |
| write | 128 B | 29 ns |
| read | 64 B | 21 ns |
| read | 16 KB | 1.2 us |
| write | 16 KB | 1.0 us |
| burst write (50x 256 B) | 12.8 KB total | 1.1 us |

## R Wrapper Dispatch

| Class System | Median | Notes |
|-------------|--------|-------|
| plain fn call | 125 ns | baseline |
| env `$` dispatch | 166 ns | native env lookup |
| R6 `$` dispatch | 349 ns | |
| S3 `UseMethod` | 521 ns | |
| S4 `setMethod` | 542 ns | |
| S7 dispatch | 2.6 us | |
| wrapper overhead | 208 ns | wrapper fn -> inner fn |
| `as.integer()` coercion | 250 ns | scalar |
| `as.character()` coercion | 542 ns | scalar |

## Typed List Validation

| Fields | Median |
|--------|--------|
| 3 | 660 ns |
| 10 | 2 us |
| 50 | 12 us |

Linear scaling (~240 ns/field).

## Lint (miniextendr-lint)

| Benchmark | Scale | Median |
|-----------|-------|--------|
| full_scan | 10 modules | 1.8 ms |
| full_scan | 100 modules | 15.9 ms |
| full_scan | 500 modules | 82.6 ms |

## Reproducing

```bash
# Full Rust suite
cargo bench --manifest-path=miniextendr-bench/Cargo.toml

# Connections (feature-gated)
cargo bench --manifest-path=miniextendr-bench/Cargo.toml --features connections --bench connections

# Lint benchmarks
cargo bench --manifest-path=miniextendr-lint/Cargo.toml --bench lint_scan

# Save structured baseline
just bench-save
```

Raw results: `miniextendr-bench/BENCH_RESULTS_2026-02-18.md`
