+++
title = "Performance Benchmarks"
weight = 60
description = "Baseline measurements for miniextendr's runtime overhead on Apple M3 Max (macOS, Rust 1.93, R 4.5). Run date: 2026-02-18."
+++

Baseline measurements for miniextendr's runtime overhead on Apple M3 Max (macOS, Rust 1.93, R 4.5).
Run date: 2026-02-18.

## Quick Reference

| Subsystem | Operation | Median | Notes |
|-----------|-----------|--------|-------|
| **Worker thread** (requires `worker-thread` feature) | round-trip | 5 us | `run_on_worker` channel hop |
| **Worker thread** (requires `worker-thread` feature) | `with_r_thread` (main) | 14 ns | already on main thread |
| **Unwind protect** | `with_r_unwind_protect` | 32 ns | overhead vs direct call |
| **Unwind protect** | nested 5 layers | 169 ns | linear scaling |
| **catch_unwind** | success path | 0.5 ns | no panic |
| **catch_unwind** | panic caught | 6.3 us | panic + catch overhead |
| **ExternalPtr** | create (8 B) | 83 ns | vs Box 42 ns (2x) |
| **ExternalPtr** | create (64 KB) | 168 ns | vs Box 1.1 us |
| **Trait ABI** | vtable query | ~1 ns | cache-hot, 2 or 5 methods |
| **Trait ABI** | dispatch (1 method) | 55-63 ns | full view dispatch |
| **Trait ABI** | dispatch (all 5) | 417 ns | multi-method hot loop |
| **R allocator** | small (8 B) | 61 ns | vs system 17 ns (3.6x) |
| **R allocator** | large (64 KB) | 1.2 us | vs system 500 ns (2.4x) |

## Type Conversions

### Rust to R (`into_sexp`)

| Type | Size | Median | Notes |
|------|------|--------|-------|
| i32 | 1 | 12 ns | scalar |
| i32 | 1K | 378 ns | memcpy |
| i32 | 1M | 105 us | |
| f64 | 1M | 220 us | |
| String | 1M | ~60 ms | CHARSXP allocation dominates |
| Option\<i32\> 50% NA | 1M | 391 us | |
| i64 (smart) | scalar | 40-43 ns | INTSXP or REALSXP |

### R to Rust (`try_from_sexp`)

| Type | Size | Median | Notes |
|------|------|--------|-------|
| i32 scalar | 1 | 30 ns | |
| f64 scalar | 1 | 27 ns | |
| f64 slice | any | ~21 ns | zero-copy (pointer cast) |
| i32 slice | any | ~21 ns | zero-copy (pointer cast) |
| String | 1 | 38 ns | UTF-8 (no re-encode needed) |
| String (Latin1) | 1 | 250 ns | requires re-encoding |
| Vec\<i32\> → HashSet | 64K | 1.5 ms | hashing overhead |

### Strict Mode

Negligible overhead for scalar conversions (~2-5 ns). Vec\<i64\> at 10K: strict 6.2 us vs normal 12.4 us (strict is actually faster due to INTSXP-only fast path avoiding REALSXP conversion).

### Coercion

| Operation | Median | Notes |
|-----------|--------|-------|
| scalar int direct | 23 ns | no coercion |
| scalar int→real (R) | 31 ns | `Rf_coerceVector` |
| scalar int→real (Rust) | 23 ns | Rust-side cast |
| vec int→real (256 elts, R) | 350 ns | R `as.double()` |
| vec int→real (256 elts, Rust) | 265 ns | Rust-side conversion |

Rust-side coercion is ~25% faster than R's `Rf_coerceVector` for vectors.

## DataFrames

| Operation | Rows | Median | Notes |
|-----------|------|--------|-------|
| Point3 → SEXP | 100 | 750 ns | 3 f64 columns |
| Point3 → SEXP | 100K | 273 us | |
| Event (enum) → SEXP | 100K | 7.1 ms | 5 columns, string-heavy |
| Mixed → SEXP | 100K | 10.5 ms | 7 columns, mixed types |
| Transpose (Point3) | 100K | 246 us | row→column pivot |
| Transpose (wide 10-col) | 100K | 1.4 ms | |

## ALTREP

| Operation | Size | ALTREP | Plain | Ratio |
|-----------|------|--------|-------|-------|
| element access (elt) | 1 | 220 ns | 9 ns | 24x |
| DATAPTR materialization | 64K | 17-20 us | 9 ns | — |
| full scan (elt loop) | 64K | 5.2 ms | 2.7 us | ~1900x |
| full scan (DATAPTR) | 64K | 20 us | 9 ns | — |

### Guard Modes (64K elements, full scan)

| Guard | Median |
|-------|--------|
| `unsafe` | 16.7 ms |
| `rust_unwind` (default) | 17.5 ms |
| `r_unwind` | 21 ms |
| plain INTSXP | 261 us |

`unsafe` and `rust_unwind` are equivalent. `r_unwind` adds ~25% overhead due to `R_UnwindProtect` per callback.

### String ALTREP (64K strings)

| Operation | Median |
|-----------|--------|
| create | 2.6 ms |
| elt access | 2.7 ms |
| elt with NA | 2.4 ms |
| force materialize (DATAPTR_RO) | 6.9 ms |
| plain STRSXP elt | 4.7 ms |

### Zero-Allocation (constant real)

| Operation | Size | Median |
|-----------|------|--------|
| create constant | any | 229 ns |
| constant elt | any | 513 ns |
| constant full scan | 64K | 17.9 ms |
| vec-backed full scan | 64K | 5.2 ms |

## Connections

| Operation | Size | Median |
|-----------|------|--------|
| build + open | — | 583 ns |
| write | 128 B | 25 ns |
| read | 64 B | 24 ns |
| read | 16 KB | 1.7 us |
| write | 16 KB | 1.0 us |
| burst write (50x 256 B) | 12.8 KB total | 1.2 us |

## R Wrapper Dispatch

| Class System | Median | Notes |
|-------------|--------|-------|
| plain fn call | 125 ns | baseline |
| env `$` dispatch | 166 ns | native env lookup |
| R6 `$` dispatch | 364 ns | |
| S3 `UseMethod` | 521 ns | |
| S4 `setMethod` | 542 ns | |
| S7 dispatch | 2.8 us | |
| wrapper overhead | 229 ns | wrapper fn → inner fn |
| `as.integer()` coercion | 291 ns | scalar |
| `as.character()` coercion | 625 ns | scalar |

## GC Protection

See `analysis/gc-protection-strategies.md` for full analysis and
`analysis/gc-protection-benchmarks-results.md` for detailed results.
All numbers below measure pure protection cost (SEXP allocation excluded).

### Per-operation cost

| Mechanism | Single op | Notes |
|-----------|-----------|-------|
| Protect stack | **7.4 ns** | array write + integer subtract |
| Vec pool (VECSXP) | **9.6 ns** | SET_VECTOR_ELT + free list |
| Slotmap pool | 11.4 ns | + generational safety check |
| Precious list | 13.1 ns | CONS alloc + prepend |
| DLL preserve | 28.9 ns | CONS alloc + doubly-linked splice |

### Batch throughput (protect N, release all)

| Mechanism | 1k | 10k | 50k |
|-----------|----|-----|-----|
| Protect stack | **3.8 µs** | **38 µs** | — (50k limit) |
| Vec pool | 9.6 µs | 97 µs | 486 µs |
| Slotmap pool | 11.7 µs | 116 µs | 575 µs |
| DLL preserve | 27.2 µs | 256 µs | 1.31 ms |
| Precious list | 568 µs | — | — |

### Replace-in-loop (N replacements)

| Mechanism | 10k | Per-op |
|-----------|-----|--------|
| ReprotectSlot | **37.6 µs** | 3.8 ns |
| Pool overwrite | 45.2 µs | 4.5 ns |
| DLL reinsert | 271 µs | 27.1 ns |
| Precious churn | **15.1 s** | 1.5 ms (O(n²)) |

## Typed List Validation

| Fields | Median |
|--------|--------|
| 3 | 682 ns |
| 10 | 2.1 us |
| 50 | 12.8 us |

Linear scaling (~240 ns/field).

## Factors

| Operation | Median |
|-----------|--------|
| single (cached) | 58 ns |
| single (uncached) | 372 ns |
| 100 repeated (cached) | 5.5 us |
| Vec\<Factor\> (4096) | 4.4 us |

## Lint (miniextendr-lint)

| Benchmark | Scale | Median |
|-----------|-------|--------|
| full_scan | 10 modules | 1.9 ms |
| full_scan | 100 modules | 16.3 ms |
| full_scan | 500 modules | 84.9 ms |
| impl_scan | 10 types | 1.9 ms |
| impl_scan | 100 types | 16.8 ms |
| scaling | 500 fns, 10 files | 5.9 ms |
| scaling | 500 fns, 500 files | 67.9 ms |

Linear scaling in both module count and file count.

## FFI Call Overhead

| Operation | Size | Median |
|-----------|------|--------|
| Rf_ScalarInteger | — | 11 ns |
| Rf_ScalarReal | — | 12 ns |
| Rf_ScalarLogical | — | 4 ns |
| INTEGER_ELT | any | 7.5 ns |
| REAL_ELT | any | 7.6 ns |
| Rf_protect/unprotect | 1 | 18 ns |
| Rf_allocVector (INTSXP, 64K) | 64K | ~235 ns |

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
