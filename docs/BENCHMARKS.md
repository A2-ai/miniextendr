# Benchmarks

Performance-investigation tooling for miniextendr: how to run the
bench suite, how to capture a baseline, and what the published
numbers actually mean.

`miniextendr-bench` embeds R through `miniextendr-engine`, exercises
every FFI path the runtime cares about (conversions, ALTREP,
ExternalPtr, worker routing, trait ABI, unwind protection, GC
protection, class dispatch, …), and prints numbers that engineers can
compare against an earlier baseline before landing a change.

The crate is `publish = false` and is **never** shipped in a CRAN
tarball. It is for maintainer and contributor use only.

## Why benchmarks exist

miniextendr's selling points are safety, small FFI overhead, and
ALTREP throughput. Those are empirical claims — the only way to keep
them true across refactors is to measure them. Every non-trivial
change to a hot path (conversions, ALTREP callbacks, unwind protect,
worker dispatch, class-system codegen) should be accompanied by at
least one `just bench`/`just bench-save` run and, ideally, a
`just bench-drift` check against the prior baseline.

## Running benchmarks

Prerequisites:

- R installed and reachable on `PATH`
- `rustup` with the workspace-pinned toolchain
- `cargo-limit` (optional, via `just dev-tools-install`)

### Everyday recipes

```bash
# Full Rust suite (all targets in miniextendr-bench/benches/)
just bench

# A single target (e.g. trait_abi dispatch microbenchmarks)
just bench --bench trait_abi

# Core high-signal subset (ffi_calls, into_r, from_r, translate,
# strings, externalptr, worker, unwind_protect)
just bench-core

# Feature-gated (connections, rayon, refcount-fast-hash)
just bench-features

# Full matrix: core + feature targets
just bench-full
```

### Specialised targets

```bash
# Lint-scan benchmarks (miniextendr-lint/benches/lint_scan.rs)
just bench-lint

# Macro compile-time benchmark (synthetic crates, cold/warm/incr)
just bench-compile

# R-side benchmarks (requires rpkg installed + the R `bench` package)
just bench-r
```

### Filtering inside a target

Everything after `--` is forwarded to `cargo bench`, and everything
after a second `--` is forwarded to the `divan` harness. Divan
supports regex filtering and baseline save/compare:

```bash
# Only run the `altrep_int_dataptr` benchmarks inside the altrep target
just bench --bench altrep -- altrep_int_dataptr

# Save a divan baseline named "main" for the worker target
cargo bench --manifest-path=miniextendr-bench/Cargo.toml \
  --bench worker -- --save-baseline main

# …make changes, then compare against that baseline
cargo bench --manifest-path=miniextendr-bench/Cargo.toml \
  --bench worker -- --baseline main
```

## Structured baselines and drift detection

The repository ships a small wrapper around divan that stores raw
output, a machine-readable CSV, and git metadata under
`miniextendr-bench/baselines/`.

| Recipe | What it does |
|--------|--------------|
| `just bench-save` | Run the suite, parse output, write `bench-<timestamp>.txt` + `bench-<timestamp>.csv` + `bench-<timestamp>.meta.json` to `miniextendr-bench/baselines/` |
| `just bench-compare [file.csv]` | Print the slowest/fastest benchmarks from the most recent (or specified) baseline |
| `just bench-drift [--threshold=20]` | Compare the two most recent baselines; fail if any median regressed beyond the threshold percentage |
| `just bench-info` | List saved baselines with git-SHA, R version, CPU, date |

The CSV schema is:

```
timestamp,bench_target,group,name,args,median_ns,unit
```

Drift detection uses `median_ns` so unit-changes (ns ↔ us ↔ ms)
never produce false positives. The implementation lives in
`tests/perf/bench_baseline.sh`.

## Environment setup for reproducible runs

Noise is the enemy. Before a baseline run:

1. Close heavy background workloads (editors with LSP servers, Docker,
   video calls). A 10% jitter floor is normal; 30%+ usually means
   something else is hitting the CPU.
2. Disable CPU turbo and frequency scaling if possible.
3. On macOS, disable App Nap for the shell running the bench.
4. On Linux, pin to specific cores via `taskset -c <N>` and disable
   hyper-thread siblings. `DIVAN_SKIP_SLOW=1` skips the 50 K-size
   end of the size matrix for faster iteration while developing.

Before saving a baseline, capture the environment so a later reader
can reproduce it:

```bash
# System / OS
uname -a
sysctl -n machdep.cpu.brand_string            # macOS
cat /proc/cpuinfo | grep -m1 'model name'     # Linux

# R
R --version | head -1
Rscript -e 'sessionInfo()'

# Rust
rustc --version
cargo --version

# miniextendr
git describe --always --dirty
```

`just bench-save` captures most of this automatically into the
`.meta.json` sidecar.

### Environment variables

| Variable | Effect |
|----------|--------|
| `R_HOME` | Selects the R installation the embedder uses |
| `DIVAN_SKIP_SLOW=1` | Skips the largest size in each size matrix |
| `RAYON_NUM_THREADS` | Thread count for the `rayon` benchmarks |
| `CARGO_INCREMENTAL=0` | Matches CI; avoids per-run hash noise in sccache |

## What each target measures

Each file under `miniextendr-bench/benches/` focuses on one subsystem
so iterations stay fast and failures point at a single owner. The
high-level plan lives in `miniextendr-bench/src/bench_plan.rs`.

| Target | Measures |
|--------|----------|
| `allocator` | R allocator vs system allocator across size matrix |
| `altrep` / `altrep_advanced` / `altrep_iter` | ALTREP element access, `DATAPTR` materialisation, guard modes, string ALTREP, zero-alloc patterns |
| `coerce` | R-side `Rf_coerceVector` vs Rust-side coercion |
| `connections` | Custom R connections: build/open, read, write, burst write (feature `connections`) |
| `dataframe` | `Vec<Struct>` → R data.frame transpose + full pipelines |
| `externalptr` | Creation, access, downcast, N-ptr churn; vs plain `Box` baseline |
| `factor` | Cached vs uncached level lookup; `Vec<Factor>` throughput |
| `ffi_calls` | Raw `Rf_*` calls (`ScalarInteger`, `allocVector`, `protect`/`unprotect`, `install`) |
| `from_r` | `TryFromSexp` scalar/slice/map/set paths |
| `gc_protect` | `OwnedProtect`, `ProtectScope`, manual `PROTECT`/`UNPROTECT` |
| `gc_protection_compare` | Seven protection mechanisms head-to-head (stack, vec pool, slotmap, precious list, DLL preserve, reprotect-slot, DLL reinsert) |
| `into_r` | `IntoR` scalar and vector (incl. 1 M-element scale benchmarks) |
| `list` | List construction, named lookup, derive-based builders |
| `native_vs_coerce` | RNative memcpy path vs element-wise coercion |
| `panic_telemetry` | Panic-hook `RwLock` read cost, fire cost |
| `preserve` | Preserve list insert/release vs `PROTECT`/`UNPROTECT` |
| `raw_access` | `r_slice`, unchecked slice, raw pointer access across INTSXP/REALSXP |
| `rarray` | `RArray`/`RMatrix` row/col access patterns |
| `rayon` | `rayon_bridge` parallel helpers (feature `rayon`) |
| `refcount_protect` | `RefCountedArena` vs `ProtectScope` vs raw preserve |
| `rffi_checked` | `#[r_ffi_checked]` overhead vs `*_unchecked` variant |
| `sexp_ext` | `SexpExt` helpers vs raw pointer deref |
| `strict` | Strict-mode scalar/vector conversions vs normal |
| `strings` | UTF-8/Latin-1 extraction, `mkCharLen`, `translateCharUTF8` |
| `trait_abi` | `mx_erased` vtable query + dispatch (single-method and 5-method) |
| `translate` | `R_CHAR` vs `translateCharUTF8` extraction costs |
| `typed_list` | `typed_list!` validation across field count |
| `unwind_protect` | `with_r_unwind_protect` overhead; nested layers; panic path |
| `worker` | Worker-thread round-trip, channel saturation, batching, payload size |
| `wrappers` | Generated R wrapper dispatch (plain, env, R6, S3, S4, S7); requires rpkg installed |

Each benchmark uses the shared `miniextendr_bench::init()` harness
from `miniextendr-bench/src/lib.rs`, which embeds R on the init
thread and asserts thread affinity on every call. The canonical size
matrix is `[1, 16, 256, 4096, 65536]`; named-list benchmarks use
`[16, 256, 4096]`.

## Interpreting results

Divan prints four numbers per case:

- **fastest** — the single fastest sample. Useful for latency-bound
  comparisons but noisy across runs.
- **slowest** — the slowest sample. A sudden slowest spike hints at GC
  pauses, OS scheduling, or an allocator interaction worth chasing.
- **median** — the canonical number for drift detection. Not skewed
  by a single spike.
- **mean / std-dev** — use when comparing two allocation-heavy paths
  where allocator variance matters.

Rules of thumb:

- For anything under ~50 ns, absolute deltas are almost always below
  measurement noise. Compare *ratios* across a size matrix instead.
- For ALTREP and worker paths, always report both the cold path
  (first call, includes class/channel init) and the hot path.
- String and CHARSXP allocation dominates large-vector conversion
  timings. Changes in `strings`/`into_r` that don't touch
  `mkCharLenCE` should not move those numbers.

## Reference baseline — 2026-02-18 (Apple M3 Max)

Full raw data, all tables, and methodology notes are in
[`miniextendr-bench/BENCH_RESULTS_2026-02-18.md`](../miniextendr-bench/BENCH_RESULTS_2026-02-18.md).
The headline numbers below are a subset for quick reference.

Environment: `rustc 1.93.0`, R 4.5, macOS 15.3 arm64, 36 GB RAM.
Commit `d479886` on `main`.

### Quick reference

| Subsystem | Operation | Median | Notes |
|-----------|-----------|--------|-------|
| Worker thread | `run_on_worker` round-trip | 5 µs | channel hop |
| Worker thread | `with_r_thread` (on main) | 10 ns | near free |
| Unwind protect | `with_r_unwind_protect` | 31 ns | overhead vs direct |
| Unwind protect | nested 5 layers | 170 ns | linear |
| `catch_unwind` | success path | 0.5 ns | |
| `catch_unwind` | panic caught | 5.3 µs | panic + catch |
| ExternalPtr | create (8 B) | 65 ns | Box baseline 12 ns (~5×) |
| ExternalPtr | create (64 KB) | 727 ns | Box baseline 512 ns (~1.4×) |
| Trait ABI | `mx_query_vtable` | 1 ns | cache-hot |
| Trait ABI | single-method dispatch | 55–62 ns | view + call |
| Trait ABI | all-5-method dispatch | 417 ns | multi-method hot loop |
| R allocator | small (8 B) | 71 ns | system 16 ns |
| R allocator | large (64 KB) | 867 ns | system 521 ns |

### Type conversions (64 K elements unless noted)

| Type | `into_sexp` median | `try_from_sexp` median |
|------|--------------------|-------------------------|
| `i32` scalar | 12.5 ns | 3.1 ns |
| `f64` scalar | 12.5 ns | 3.0 ns |
| `Vec<i32>` | 13.3 µs | — |
| `Vec<f64>` | 22.5 µs | — |
| `Vec<String>` | 3.95 ms | — |
| `Vec<Option<i32>>` (50% NA) | 14.1 µs | — |
| `slice_i32` / `slice_f64` | — | 8.0 ns (zero-copy) |
| `HashSet<i32>` from 4 K | — | 68 µs |

**1 M element scale:** `i32` 675 µs; `f64` 1.6 ms; `String` 276 ms
(CHARSXP allocation dominates); `Option<i32>` 934 µs.

### ALTREP (64 K elements)

| Operation | ALTREP | Plain INTSXP |
|-----------|--------|--------------|
| element access (elt) | 16.2 µs | 9.2 ns |
| `DATAPTR` materialisation | 15.1 µs | 9.9 ns |
| full scan via elt loop | 5.1 ms | — |
| full scan via `DATAPTR` | 18.6 µs | 8.9 ns |

**Guard modes (64 K full scan):** `unsafe` 15.7 ms • `rust_unwind`
(default) 16.2 ms • `r_unwind` 20.2 ms • plain INTSXP 258 µs.
`r_unwind` adds ~25% over `unsafe` due to per-callback
`R_UnwindProtect`.

**String ALTREP (64 K strings):** create 2.6 ms • elt access 2.6 ms •
elt with NA 2.4 ms • force materialise 6.6 ms • plain STRSXP elt
4.5 ms.

### GC protection (per-op)

| Mechanism | Single op | Notes |
|-----------|-----------|-------|
| Protect stack | 7.4 ns | array write + counter decrement |
| Vec pool (VECSXP) | 9.6 ns | + free list |
| Slotmap pool | 11.4 ns | + generational safety check |
| Precious list | 13.1 ns | CONS alloc + prepend |
| DLL preserve | 28.9 ns | CONS alloc + doubly-linked splice |

**Replace-in-loop (10 K iterations):** ReprotectSlot 37.6 µs •
Pool overwrite 45.2 µs • DLL reinsert 271 µs • Precious churn 15.1 s
(O(n²) — do not use for reassignment).

### Lint (miniextendr-lint)

| Benchmark | Scale | Median |
|-----------|-------|--------|
| `full_scan` | 10 modules | 1.8 ms |
| `full_scan` | 100 modules | 15.6 ms |
| `full_scan` | 500 modules | 82.1 ms |
| `impl_scan` | 10 types | 1.8 ms |
| `impl_scan` | 100 types | 16.1 ms |
| `scaling` | 500 fns / 10 files | 5.8 ms |
| `scaling` | 500 fns / 500 files | 64.1 ms |

Linear in both module count and file count.

### Connections (feature `connections`)

| Operation | Size | Median |
|-----------|------|--------|
| `connection_build` + open | — | 542 ns |
| `connection_write` | 128 B | 29 ns |
| `connection_read` | 64 B | 21 ns |
| `connection_read` | 16 KB | 1.2 µs |
| `connection_write_sized` | 16 KB | 1.0 µs |
| `connection_burst_write` | 50× 256 B | 1.1 µs |

## Publishing new baselines

When a PR changes numbers that show up in this document, attach a
fresh baseline to the PR and point `BENCH_RESULTS_<date>.md` at it.
The workflow:

1. `just bench-save` on a quiet machine.
2. Commit the new raw results file under
   `miniextendr-bench/BENCH_RESULTS_<YYYY-MM-DD>.md` (re-using the
   template from the 2026-02-18 file).
3. Update the "Reference baseline" section in this document to point
   at the new file.
4. Mention in the PR description which subsystem moved and by how
   much. If the reference baseline's machine differs from the one
   used for the run, note it — don't silently mix hardware.

Old baselines stay in the tree. `miniextendr-bench/baselines/` plus
the dated `BENCH_RESULTS_*.md` files are the historical record used
by `just bench-drift`.

## Skipped / known issues

- **`wrappers`**: Requires `rpkg` installed to provide R class
  methods. Skipped on a cold checkout; run
  `just rcmdinstall && just bench --bench wrappers`.
- **R-side benchmarks**: Need rpkg installed and the R `bench`
  package. Run `just bench-r`.
- **Macro compile-time**: Generates synthetic crates on a
  temp-dir. Run `just bench-compile`; the results are written to the
  temp-dir and summarised on stdout.
- **Feature combinations that conflict with `--all-features`**:
  `default-r6` and `default-s7` are mutually exclusive, so
  `--all-features` fails. Use the explicit feature list from
  `CLAUDE.md`'s "Reproducing CI clippy before PR" section when you
  need the CI-equivalent set.

## Reference

- Raw results file: [`miniextendr-bench/BENCH_RESULTS_2026-02-18.md`](../miniextendr-bench/BENCH_RESULTS_2026-02-18.md)
- Benchmark plan: `miniextendr-bench/src/bench_plan.rs`
- Harness and fixtures: `miniextendr-bench/src/lib.rs`
- Baseline tooling: `tests/perf/bench_baseline.sh`
- Cross-reference analysis (GC protection):
  `analysis/gc-protection-strategies.md` and
  `analysis/gc-protection-benchmarks-results.md`
