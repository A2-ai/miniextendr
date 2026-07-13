# miniextendr_bench v0.1.0

miniextendr-bench: benchmark harness helpers for miniextendr.

This crate embeds R via `miniextendr-engine` and provides fixtures and
helpers for `divan` benchmarks in `miniextendr-bench/benches/`. It is
intended for local development and performance investigation, not for
publishing or CRAN builds.

## Running benchmarks

```ignore
cargo bench --manifest-path=miniextendr-bench/Cargo.toml --bench translate
```

Notes:
- Requires R installed and available on PATH.
- The benchmark plan lives in `bench_plan` (module-level docs).

---

## Modules

### `bench_plan`

`pub mod bench_plan;`

Comprehensive benchmark plan for `miniextendr-bench`.

This module is documentation-only. It lays out the bench files, modules,
fixtures, and parameter matrices that should exist, but does not include any
executable benchmark code.

---------------------------------------------------------------------------
Planned bench targets (files under `miniextendr-bench/benches/`)

Each target should call `miniextendr_bench::init()` and run on the init
thread. Targets should be organized by topic to keep iteration times low and
to allow selective runs (e.g., `cargo bench --bench from_r`).

- `ffi_calls.rs`        Raw R API calls vs checked wrappers
- `sexp_ext.rs`         `SexpExt` helpers vs raw pointers
- `into_r.rs`           Rust -> R conversions (scalars, vectors, strings)
- `from_r.rs`           R -> Rust conversions (scalars, slices, maps, sets)
- `strings.rs`          Encoding and string extraction variants
- `coerce.rs`           Coerce / TryCoerce cost and error paths
- `altrep.rs`           ALTREP callbacks and data access patterns
- `altrep_iter.rs`      Iterator-backed ALTREP performance
- `externalptr.rs`      ExternalPtr creation, access, tagging
- `trait_abi.rs`        mx_erased / trait vtable query and dispatch
- `unwind_protect.rs`   with_r_unwind_protect overhead (normal and error)
- `worker.rs`           worker-thread dispatch overhead vs direct calls
- `allocator.rs`        RAllocator vs System allocator (when applicable)
- `rayon.rs`            rayon_bridge parallel helpers (feature-gated)
- `connections.rs`      Custom connections (feature-gated)
- `wrappers.rs`         R wrapper call overhead (optional, via R eval)
- `list.rs`             list construction + named lookup + derives
- `rarray.rs`           RArray/RMatrix access patterns
- `factor.rs`           RFactor cached vs uncached levels
- `gc_protect.rs`       ProtectScope, OwnedProtect, builders
- `native_vs_coerce.rs` RNative memcpy vs element-wise coercion
- `refcount_protect.rs` RefCountedArena vs ProtectScope vs raw R_PreserveObject
- `translate.rs`        R_CHAR vs translateCharUTF8 string extraction

---------------------------------------------------------------------------
Shared harness expectations

- Use `miniextendr_bench::init()` once per process.
- Assert `miniextendr_bench::assert_on_init_thread()` for any R calls.
- Reuse fixtures where possible; avoid allocating per-iteration unless that
  is what is being measured.
- Use `divan` groups with clear parameter sets and labels.
- For allocation-heavy benchmarks, separate "allocation included" and
  "allocation excluded" cases.
- Keep NA density and size fixed within a benchmark to avoid noisy results.

---------------------------------------------------------------------------
Standard size matrix

The canonical size set is defined by `SIZES` in `lib.rs`:
  `[1, 16, 256, 4096, 65536]`

Named lists use a smaller set (`NAMED_LIST_SIZES`):
  `[16, 256, 4096]`

Standard NA densities (for logical/real/int/string where applicable):
- none (0%)
- sparse (~1%)
- moderate (~10%)
- heavy (~50%)

---------------------------------------------------------------------------
Fixtures to provide from the harness

- UTF-8 and Latin-1 CHARSXP and STRSXP fixtures (already in lib.rs).
- Pre-allocated vectors for each type/size matrix (INTSXP, REALSXP,
  LGLSXP, RAWSXP, STRSXP, VECSXP).
- Rust-side `Vec<T>` inputs mirroring the same sizes.
- Named list fixtures for map conversions.
- Matrix/array fixtures for `rarray` access benchmarks.
- ExternalPtr fixtures for tagging/protection tests.
- ALTREP class fixtures for each data type and iterator variant.

---------------------------------------------------------------------------
Module map (documentation only)

- `harness`: shared fixture and parameter design
- `ffi_calls`: raw R API calls, checked vs unchecked
- `sexp_ext`: `SexpExt` helpers vs raw pointer access
- `into_r`: conversion costs for IntoR
- `from_r`: conversion costs for TryFromSexp
- `strings`: encoding and string extraction costs
- `coerce`: Coerce / TryCoerce / Coerced
- `altrep`: ALTREP class access and callbacks
- `altrep_iter`: iterator-backed ALTREP
- `externalptr`: ExternalPtr creation/access/protection
- `trait_abi`: trait ABI dispatch (mx_erased query + vtable calls)
- `unwind_protect`: with_r_unwind_protect overhead
- `worker`: worker thread dispatch overhead
- `allocator`: RAllocator behavior
- `rayon`: parallel helpers
- `connections`: custom connections
- `wrappers`: generated R wrapper overhead
- `rffi_checked`: checked wrapper overhead
- `list`: list primitives and derives
- `rarray`: array/matrix access patterns
- `factor`: RFactor cached vs uncached levels
- `gc_protect`: ProtectScope, OwnedProtect, builders
- `native_vs_coerce`: RNative path vs element-wise coercion
- `refcount_protect`: RefCountedArena vs ProtectScope
- `translate`: R_CHAR vs translateCharUTF8

Each submodule contains a detailed plan for its bench cases.

### `bench_plan::allocator`

`pub mod allocator;`

RAllocator benchmarks.

Planned groups:
- `alloc_small` / `alloc_large` (bytes -> KB -> MB)
- `realloc_grow` / `realloc_shrink`
- `dealloc` cost
- compare to System allocator (baseline)

Parameters:
- size classes
- alignment classes

### `bench_plan::altrep`

`pub mod altrep;`

ALTREP benchmarks.

Focus on ALTREP class behavior and callback costs for each vector type.

Planned groups:
1) `elt_access`
   - ALTINTEGER / ALTREAL / ALTLOGICAL / ALTRAW / ALTSTRING / ALTLIST
   - Compare elt() vs get_region() vs dataptr() where applicable

2) `get_region`
   - varying region sizes: 1, 8, 64, 1024
   - contiguous vs random access patterns

3) `summary_methods`
   - sum/min/max (numeric), no_na, is_sorted
   - compare ALTREP overrides vs materialized vectors

4) `duplicate`
   - shallow vs deep duplicate cost

5) `coerce`
   - ALTREP Coerce method vs R's default coercion

6) `dataptr_or_null`
   - cost and behavior for lazy vs materialized data

Parameters:
- size matrix, NA density matrix
- materialized vs non-materialized ALTREP

### `bench_plan::altrep_iter`

`pub mod altrep_iter;`

Iterator-backed ALTREP benchmarks.

Planned groups:
- `sequential_access` (0..n)
- `random_access` (sparse indices)
- `materialize` (force full realization)
- `get_region` (bulk read)
- `coerce_variants` (IterIntCoerce, IterRealCoerce, bool->int)
- `option_iterators` (`Option<T>` -> NA)

Compare:
- iterator-backed vs pre-materialized `Vec<T>`
- ExactSizeIterator vs explicit length
- cache hit vs miss performance

### `bench_plan::coerce`

`pub mod coerce;`

Coerce / TryCoerce benchmarks.

Planned groups:
- `infallible_scalar` (i8/i16/u16/bool -> i32 or f64)
- `fallible_scalar` (f64 -> i32, u64 -> i32) with overflow/precision cases
- `slice_coerce` (`Vec<T>` -> `Vec<R>`) for large sizes
- `option_coerce` (`Option<T>` -> NA mapping)
- `coerced_wrapper` (`Coerced<T, R>` creation and access)

Track:
- success vs error path costs
- scaling with vector length

### `bench_plan::connections`

`pub mod connections;`

Connection framework benchmarks (feature = "connections").

Implemented groups:
- `open_close`: create/destroy connection
- `read_small` / `read_large`: 128 bytes / 4096 bytes
- `write_small` / `write_large`: 128 bytes / 4096 bytes

Remaining gaps:
- `seek` / `tell` benchmarks
- Buffering on/off variants
- Encoding variations

### `bench_plan::externalptr`

`pub mod externalptr;`

ExternalPtr benchmarks.

Planned groups:
- `create` (ExternalPtr::new) vs from_raw
- `access` (as_ref/as_mut) and pointer checks
- `tag_lookup` (tag, stored_type_id, type comparisons)
- `set_protected` (user-protected slot updates)
- `try_from_sexp` success/failure paths
- `into_raw` and reclaim cost

Parameters:
- small vs large payload types (e.g., i32 vs `Vec<i32>`)
- type-erased vs typed external pointers

### `bench_plan::factor`

`pub mod factor;`

RFactor enum ↔ R factor benchmarks.

Implemented groups:
- `single_value`: cached (OnceLock) vs uncached levels for single enum → factor
- `vector`: FactorVec of 256 elements, cached vs uncached

Key finding: ~4x speedup for single value conversions with cached levels.
Vector conversions show minimal difference since allocation dominates.

### `bench_plan::ffi_calls`

`pub mod ffi_calls;`

FFI call overhead benchmarks.

Measure the cost of calling R C-API functions through:
- checked wrappers (thread assertions)
- unchecked wrappers (no assertions)
- direct raw FFI functions (where available)

Planned benchmark groups:

1) `alloc_vector`
   - `Rf_allocVector` vs `Rf_allocVector_unchecked`
   - Types: INTSXP, REALSXP, LGLSXP, STRSXP
   - Sizes: tiny -> large

2) `scalar_creation`
   - `Rf_ScalarInteger`, `Rf_ScalarReal`, `Rf_ScalarLogical`
   - Checked vs unchecked

3) `data_access`
   - `INTEGER`, `REAL`, `LOGICAL`, `RAW`, `DATAPTR_RO`
   - Pointer acquisition only (no copy)

4) `protect_unprotect`
   - `Rf_protect` / `Rf_unprotect` vs ProtectPool (see protect_pool module)
   - Measure cost per protect/unprotect pair

Metrics:
- ns/op for each call
- throughput for large vector allocations (alloc/sec)

### `bench_plan::from_r`

`pub mod from_r;`

Benchmarks for R -> Rust conversions (TryFromSexp).

Planned groups:

1) `scalars`
   - i32, f64, bool, Rboolean
   - `Option<T>` (NA handling)

2) `slices`
   - &'static [i32], &'static [f64], &'static [u8]
   - Compare to manual pointer access + slice creation

3) `vectors`
   - `Vec<String>` (NA -> empty string)
   - `Vec<Option<String>>` (NA -> None)
   - `Vec<Option<i32/f64/bool>>`

4) `collections`
   - `HashSet<T>`, `BTreeSet<T>` for native types
   - `HashMap<String, V>` and `BTreeMap<String, V>` from named lists

5) `coerced`
   - `Coerced<T, R>` for numeric widening/narrowing
   - Error path measurement (overflow, precision loss)

Parameters:
- Size matrix and NA density matrix
- Named list sizes and key formats
- Encoding variants for strings

### `bench_plan::gc_protect`

`pub mod gc_protect;`

GC protection benchmarks.

Implemented groups:
- `protect_scope`: ProtectScope vs raw Rf_protect/Rf_unprotect
- `owned_protect`: OwnedProtect vs ProtectScope::protect
- `reprotect`: ReprotectSlot::set vs re-protect patterns
- `list_builders`: List::set_elt, set_elt_unchecked, ListBuilder, ListAccumulator, collect_list
- `strvec_builders`: StrVec::set_elt, StrVecBuilder, collect patterns
- `named_list`: ListBuilder with names vs manual allocation

Comprehensive coverage of all GC protection and builder APIs.

### `bench_plan::harness`

`pub mod harness;`

Benchmark harness plan.

Goal: ensure every benchmark runs with consistent fixtures, sizes, and
thread safety. This module describes the shared helpers that should exist
in `miniextendr-bench` (not implemented here).

---------------------------------------------------------------------------
Planned helpers

1) `init()`
   - Initialize embedded R once via `miniextendr_engine`.
   - Record the init thread and enforce single-threaded R access.

2) `assert_on_init_thread()`
   - Panic if any R API call is made from a non-init thread.

3) `Fixtures` struct
   - Pre-allocated R vectors for each type and size class.
   - Matching Rust `Vec<T>` inputs for IntoR benchmarks.
   - Named lists for map conversion benches.
   - ExternalPtr fixtures (typed and type-erased).
   - ALTREP classes for data and iterator-backed variants.
   - String fixtures: UTF-8, Latin-1, ASCII-only, empty, NA.

4) `Param` types
   - Size enum: tiny / small / medium / large.
   - NA density enum: none / sparse / moderate / heavy.
   - Encoding enum: ascii / utf8 / latin1 / bytes.
   - Flags: include_alloc (yes/no), include_gc (yes/no).

---------------------------------------------------------------------------
Execution guidelines

- Benchmarks should not allocate within the hot loop unless explicitly
  measuring allocation cost.
- Use divan parameterization and label all cases (type, size, NA density).
- Warm up R by running a small no-op .Call once per bench file.
- Keep all R objects protected or preserved for the full benchmark run.
- Prefer precomputed fixtures to avoid R GC noise.
- Use explicit “with allocation” and “without allocation” variants.

### `bench_plan::into_r`

`pub mod into_r;`

Benchmarks for Rust -> R conversions (IntoR).

Planned groups:

1) `scalars`
   - i32, f64, bool, Rboolean, RLogical
   - `Option<T>` (Some/None)

2) `vectors_native`
   - `Vec<i32>`, `Vec<f64>`, `Vec<u8>`, `Vec<RLogical>`
   - `&[]` and `&[T]` slices
   - Include NA densities for logical/real/int options

3) `vectors_option`
   - `Vec<Option<i32>>`, `Vec<Option<f64>>`, `Vec<Option<bool>>`
   - `Vec<Option<String>>`
   - NA density matrix

4) `strings`
   - `&[String]`, `Vec<String>`, `&[&str]`
   - ASCII vs UTF-8 vs Latin-1 payloads

5) `lists`
   - `Vec<SEXP>` (pre-allocated)
   - Nested lists (small depth)

Metrics:
- ns/op for scalar conversions
- MB/s for vector conversions
- allocations per conversion (if possible)

### `bench_plan::list`

`pub mod list;`

Bench plan: `benches/list.rs`

Groups:

1) `get_named` vs `get_index`
   - best-case (first element) vs worst-case (last element)
   - sizes from `NAMED_LIST_SIZES`

2) derive-driven conversions
   - `#[derive(IntoList)]` named vs tuple structs
   - `#[derive(TryFromList)]` named vs tuple structs
   - `#[into_list(ignore)]` field skipping impact (reads + bounds)

Notes:
- Keep list fixtures protected for the entire benchmark process.
- For `TryFromList`, avoid including list allocation cost unless explicitly measuring it
  (use protected fixture `SEXP`s or `divan::Bencher::with_inputs`).

### `bench_plan::native_vs_coerce`

`pub mod native_vs_coerce;`

RNative path vs Coercion path benchmarks.

Implemented groups (all parameterized by SIZES[0..5]):
- `integer_native`: &[i32] slice (zero-copy), `Vec<i32>` (memcpy)
- `integer_coerce`: `Vec<i64>` (widen), `Vec<u32>` (bounds-check)
- `real_native`: &[f64] slice, `Vec<f64>` (memcpy)
- `real_coerce`: `Vec<f32>` (narrow), `Vec<i64>` (truncate), `Vec<i32>` (truncate+narrow)

Shows the cost gradient from zero-copy slice → memcpy → element-wise coercion.

### `bench_plan::rarray`

`pub mod rarray;`

Bench plan: `benches/rarray.rs`

Focus:
- access patterns on `RMatrix<T>` and `RArray<T, NDIM>` wrappers.

Suggested benchmarks:
- `as_slice` full-buffer iteration baseline (column-major).
- `get_rc` nested loops to quantify per-element index overhead.
- `column(col)` + per-column iteration (contiguous slices).
- `to_vec` copy cost (main-thread copy-out for worker-friendly compute).

Parameters:
- matrix sizes from `MATRIX_DIMS` (e.g. 64x64, 256x256).
- optionally: add 3D arrays and “stride-heavy” index patterns.

### `bench_plan::rayon`

`pub mod rayon;`

Rayon integration benchmarks (feature = "rayon").

Planned groups:
- `Vec<T>` parallel collection via `par_iter().collect()`
- `with_r_vec<T>` zero-copy fill vs `Vec<T>` + IntoR
- `reduce::sum` / `reduce::mean` vs sequential reductions
- scaling across Rayon thread counts

Parameters:
- vector size matrix
- parallelism level (rayon thread count)

### `bench_plan::refcount_protect`

`pub mod refcount_protect;`

RefCountedArena vs ProtectScope benchmarks.

Implemented groups:
- `raw_preserve`: R_PreserveObject/R_ReleaseObject baseline (O(n) release)
- `protect_multi`: ProtectScope for N objects
- `refcount_arena`: RefCountedArena protect/release/refcount
- `threadlocal_arena`: ThreadLocalArena protect/release/refcount
- `arena_comparison`: head-to-head at scale (1k, 5k, 10k objects)
- `release_scaling`: O(n) raw release vs O(1) arena release at scale
- `mixed_workload`: interleaved protect/release/refcount patterns

Key finding: raw R_ReleaseObject is O(n) (scans precious list), making
protect+release cycles O(n²) at scale. RefCountedArena's BTreeMap
provides O(log n) lookup, dramatically faster than O(n) release at scale.

### `bench_plan::rffi_checked`

`pub mod rffi_checked;`

Thread-checked wrapper benchmarks.

Planned groups:
- `checked_vs_unchecked` for key FFI functions
- `panic_cost` when called from wrong thread (debug builds only)

Use simple primitives like Rf_ScalarInteger and DATAPTR_RO for comparison.

### `bench_plan::sexp_ext`

`pub mod sexp_ext;`

Benchmarks for SexpExt helpers vs raw access.

Planned cases:
- `is_integer` vs `type_of() == INTSXP`
- `len` vs `Rf_xlength` direct
- `as_slice` vs manual pointer + slice creation
- unchecked variants where available

Parameters:
- Types: int, real, logical, raw
- Sizes: tiny -> large
- Alignment: compare contiguous vectors vs ALTREP data (if possible)

### `bench_plan::strings`

`pub mod strings;`

String and encoding benchmarks.

Expand on the existing translate benchmark with a full matrix:

1) `charsxp_access`
   - R_CHAR (UTF-8/ASCII)
   - Rf_translateCharUTF8 (UTF-8, Latin-1, bytes)

2) `strsxp_to_string`
   - `TryFromSexp<String>` from STRSXP
   - `Vec<String>` from STRSXP

3) `option_strings`
   - `Option<String>` from NA vs empty
   - `Vec<Option<String>>` across NA densities

4) `roundtrip`
   - Rust String -> SEXP -> String
   - ASCII vs UTF-8 vs Latin-1 payloads

Measure:
- ns/op for scalar extraction
- throughput for vector conversions
- impact of encoding on translation path

### `bench_plan::trait_abi`

`pub mod trait_abi;`

Trait ABI (mx_erased) benchmarks.

Implemented groups:
- `query_vtable`: hit path + miss path
- `view_construct`: implicit in view_value_only, query_view_value
- `dispatch`: &self (value) vs &mut self (increment), repeated-hot (10x)
- `end_to_end`: query + view + call (query_view_value)
- `baseline`: direct concrete calls for comparison

Remaining gap:
- Multi-method trait variant (current trait has only 2 methods)

### `bench_plan::translate`

`pub mod translate;`

String extraction benchmarks: R_CHAR vs translateCharUTF8.

Implemented groups:
- `charsxp_direct`: R_CHAR → CStr → String (UTF-8 only, no translation)
- `charsxp_translate`: Rf_translateCharUTF8 → CStr → String (handles encodings)
- `strsxp_direct`: STRSXP → `Vec<String>` via R_CHAR path
- `strsxp_translate`: STRSXP → `Vec<String>` via translateCharUTF8 path

Compares the two strategies for extracting Rust strings from R CHARSXP
and STRSXP values. The translate path handles Latin-1 and native encodings
but has overhead; the direct path assumes UTF-8 only.

### `bench_plan::unwind_protect`

`pub mod unwind_protect;`

with_r_unwind_protect benchmarks.

Implemented groups:
- `baseline`: direct noop vs unwind_protect noop
- `r_call`: closure that calls a trivial R API inside unwind protection

Deferred (requires subprocess isolation):
- `panic_path`: closure that panics, converted to R error
- `r_error_path`: closure that triggers R error via longjmp

Error paths contaminate process state and cannot run in the same
bench process as normal benchmarks.

### `bench_plan::worker`

`pub mod worker;`

Worker-thread and dispatch benchmarks.

Implemented groups:
- `run_on_worker`: pure Rust closure overhead
- `with_r_thread`: round-trip latency (worker → main → worker)
- `channel_saturation`: 20 sequential worker round-trips
- `batching`: single worker hop with 10 batched R thread requests

Remaining gap:
- Payload size and batch count are hardcoded, not parameterized via divan

### `bench_plan::wrappers`

`pub mod wrappers;`

Generated R wrapper benchmarks.

Measures overhead of calling generated R wrappers vs direct `.Call`,
argument coercion costs, and class-system method dispatch.

Implemented groups:
- `wrapper_call_overhead`: noop and realvec wrapper vs direct .Call
- `direct_call_overhead`: direct .Call baseline (noop, realvec, eval_sum)
- `argument_coercion`: as.integer/as.double/as.character scalar + vec256
- `class_methods`: Env/R6/S3/S4/S7 dispatch + plain function baseline

### `pool_prototypes`

`pub mod pool_prototypes;`

VECSXP pool prototypes for benchmarking.

These are standalone implementations used to benchmark different pool
strategies head-to-head. The winner will be integrated into miniextendr-api.

#### Safety

All `unsafe` functions in this module require being called from R's main
thread with valid SEXP arguments. These are benchmark prototypes, not public API.

### `raw_ffi`

`pub mod raw_ffi;`

Raw FFI declarations for benchmarking.

These re-declare R C API functions that miniextendr-api has privatized.
Bench code needs direct access to measure overhead of safe wrappers.

---

## Structs

### `Fixtures`

```rust
pub struct Fixtures
```

Pre-allocated R values used by benchmark cases.

**Inherent associated items:**

#### `int_vec`

```rust
fn int_vec(self: Self, size_idx: usize) -> SEXP
```

Get pre-allocated INTSXP of given size index (0-4 maps to SIZES).

#### `latin1_charsxp`

```rust
fn latin1_charsxp(self: Self) -> SEXP
```

Returns a cached Latin-1 `CHARSXP`.

#### `latin1_strsxp`

```rust
fn latin1_strsxp(self: Self) -> SEXP
```

Returns a cached Latin-1 `STRSXP(1)`.

#### `lgl_vec`

```rust
fn lgl_vec(self: Self, size_idx: usize) -> SEXP
```

Get pre-allocated LGLSXP of given size index.

#### `named_list_i32`

```rust
fn named_list_i32(self: Self, size_idx: usize) -> SEXP
```

Get pre-allocated named list (VECSXP) of given size index (0-2 maps to NAMED_LIST_SIZES).

#### `raw_vec`

```rust
fn raw_vec(self: Self, size_idx: usize) -> SEXP
```

Get pre-allocated RAWSXP of given size index.

#### `real_matrix`

```rust
fn real_matrix(self: Self, size_idx: usize) -> SEXP
```

Get pre-allocated REAL matrix of given size index (0-1 maps to MATRIX_DIMS).

#### `real_vec`

```rust
fn real_vec(self: Self, size_idx: usize) -> SEXP
```

Get pre-allocated REALSXP of given size index.

#### `str_vec`

```rust
fn str_vec(self: Self, size_idx: usize) -> SEXP
```

Get pre-allocated STRSXP(1) with string of given size index.

#### `utf8_charsxp`

```rust
fn utf8_charsxp(self: Self) -> SEXP
```

Returns a cached UTF-8 `CHARSXP`.

#### `utf8_strsxp`

```rust
fn utf8_strsxp(self: Self) -> SEXP
```

Returns a cached UTF-8 `STRSXP(1)`.

### `pool_prototypes::BTreeMapPool`

```rust
pub struct BTreeMapPool
```

VECSXP pool with BTreeMap<String, usize> key management.

O(log n) insert/lookup/release, but ordered iteration and range operations.

**Inherent associated items:**

#### `get`

```rust
fn get(self: &Self, key: &str) -> Option<SEXP>
```

#### `insert`

```rust
unsafe fn insert(self: &mut Self, key: String, sexp: SEXP)
```

#### `new`

```rust
unsafe fn new(capacity: usize) -> Self
```

#### `release`

```rust
unsafe fn release(self: &mut Self, key: &str)
```

### `pool_prototypes::DequePool`

```rust
pub struct DequePool
```

VECSXP pool with `VecDeque<usize>` free list (FIFO slot reuse).

Released slots go to the back; allocations come from the front.
Delays reuse of recently-freed slots.

**Fields:**

- `backing`: `miniextendr_api::SEXP`
- `capacity`: `usize`
- `len`: `usize`

**Inherent associated items:**

#### `insert`

```rust
unsafe fn insert(self: &mut Self, sexp: SEXP) -> usize
```

#### `new`

```rust
unsafe fn new(capacity: usize) -> Self
```

#### `release`

```rust
unsafe fn release(self: &mut Self, slot: usize)
```

### `pool_prototypes::HashMapPool`

```rust
pub struct HashMapPool
```

VECSXP pool with HashMap<String, usize> key management.

O(1) insert/lookup/release by string key.

**Inherent associated items:**

#### `get`

```rust
fn get(self: &Self, key: &str) -> Option<SEXP>
```

#### `insert`

```rust
unsafe fn insert(self: &mut Self, key: String, sexp: SEXP)
```

#### `new`

```rust
unsafe fn new(capacity: usize) -> Self
```

#### `release`

```rust
unsafe fn release(self: &mut Self, key: &str)
```

### `pool_prototypes::IndexMapPool`

```rust
pub struct IndexMapPool
```

VECSXP pool with IndexMap<String, usize> key management.

O(1) insert/lookup/release by key, insertion-order iteration.

**Inherent associated items:**

#### `get`

```rust
fn get(self: &Self, key: &str) -> Option<SEXP>
```

#### `insert`

```rust
unsafe fn insert(self: &mut Self, key: String, sexp: SEXP)
```

#### `new`

```rust
unsafe fn new(capacity: usize) -> Self
```

#### `release`

```rust
unsafe fn release(self: &mut Self, key: &str)
```

### `pool_prototypes::ProtectKey`

```rust
pub struct ProtectKey
```

Generational key for VECSXP pool slots.

### `pool_prototypes::SlotmapPool`

```rust
pub struct SlotmapPool
```

VECSXP pool with slotmap generational index management.

Stale keys are safely detected via generation counter.

**Fields:**

- `backing`: `miniextendr_api::SEXP`
- `capacity`: `usize`

**Inherent associated items:**

#### `get`

```rust
fn get(self: &Self, key: ProtectKey) -> Option<SEXP>
```

#### `insert`

```rust
unsafe fn insert(self: &mut Self, sexp: SEXP) -> ProtectKey
```

#### `new`

```rust
unsafe fn new(capacity: usize) -> Self
```

#### `release`

```rust
unsafe fn release(self: &mut Self, key: ProtectKey)
```

### `pool_prototypes::VecPool`

```rust
pub struct VecPool
```

VECSXP pool with `Vec<usize>` free list (LIFO slot reuse).

Simplest possible pool. Stale handles are not detected.

**Fields:**

- `backing`: `miniextendr_api::SEXP`
- `capacity`: `usize`
- `len`: `usize`

**Inherent associated items:**

#### `get`

```rust
unsafe fn get(self: &Self, slot: usize) -> SEXP
```

#### `insert`

```rust
unsafe fn insert(self: &mut Self, sexp: SEXP) -> usize
```

#### `new`

```rust
unsafe fn new(capacity: usize) -> Self
```

#### `release`

```rust
unsafe fn release(self: &mut Self, slot: usize)
```

---

## Functions

### `assert_on_init_thread`

```rust
fn assert_on_init_thread()
```

Asserts the caller is running on the thread that initialized embedded R.

### `fixtures`

```rust
fn fixtures() -> Fixtures
```

Returns globally initialized benchmark fixtures.

### `init`

```rust
fn init()
```

Initialize the embedded R runtime and benchmark fixtures.

This must be called once, and all subsequent benchmark code should run on
the same thread.

### `raw_ffi::ALTREP`

```rust
unsafe extern "C-unwind" fn ALTREP(x: SEXP) -> i32
```

### `raw_ffi::COMPLEX`

```rust
unsafe extern "C-unwind" fn COMPLEX(x: SEXP) -> *mut Rcomplex
```

### `raw_ffi::DATAPTR_RO`

```rust
unsafe extern "C-unwind" fn DATAPTR_RO(x: SEXP) -> *const std::os::raw::c_void
```

### `raw_ffi::INTEGER`

```rust
unsafe extern "C-unwind" fn INTEGER(x: SEXP) -> *mut i32
```

### `raw_ffi::LENGTH`

```rust
unsafe extern "C-unwind" fn LENGTH(x: SEXP) -> i32
```

### `raw_ffi::LOGICAL`

```rust
unsafe extern "C-unwind" fn LOGICAL(x: SEXP) -> *mut i32
```

### `raw_ffi::RAW`

```rust
unsafe extern "C-unwind" fn RAW(x: SEXP) -> *mut u8
```

### `raw_ffi::REAL`

```rust
unsafe extern "C-unwind" fn REAL(x: SEXP) -> *mut f64
```

### `raw_ffi::R_CHAR`

```rust
unsafe extern "C-unwind" fn R_CHAR(x: SEXP) -> *const std::os::raw::c_char
```

### `raw_ffi::R_PreserveObject`

```rust
unsafe extern "C-unwind" fn R_PreserveObject(x: SEXP)
```

### `raw_ffi::R_ReleaseObject`

```rust
unsafe extern "C-unwind" fn R_ReleaseObject(x: SEXP)
```

### `raw_ffi::Rf_Scalar`

```rust
unsafe extern "C-unwind" fn Rf_Scalar(x: f64) -> SEXP
```

### `raw_ffi::Rf_ScalarComplex`

```rust
unsafe extern "C-unwind" fn Rf_ScalarComplex(x: Rcomplex) -> SEXP
```

### `raw_ffi::Rf_ScalarInteger`

```rust
unsafe extern "C-unwind" fn Rf_ScalarInteger(x: i32) -> SEXP
```

### `raw_ffi::Rf_ScalarLogical`

```rust
unsafe extern "C-unwind" fn Rf_ScalarLogical(x: i32) -> SEXP
```

### `raw_ffi::Rf_ScalarReal`

```rust
unsafe extern "C-unwind" fn Rf_ScalarReal(x: f64) -> SEXP
```

### `raw_ffi::Rf_ScalarString`

```rust
unsafe extern "C-unwind" fn Rf_ScalarString(x: SEXP) -> SEXP
```

### `raw_ffi::Rf_allocMatrix`

```rust
unsafe extern "C-unwind" fn Rf_allocMatrix(sexptype: SEXPTYPE, nrow: i32, ncol: i32) -> SEXP
```

### `raw_ffi::Rf_allocVector`

```rust
unsafe extern "C-unwind" fn Rf_allocVector(sexptype: SEXPTYPE, length: R_xlen_t) -> SEXP
```

### `raw_ffi::Rf_asInteger`

```rust
unsafe extern "C-unwind" fn Rf_asInteger(x: SEXP) -> i32
```

### `raw_ffi::Rf_duplicate`

```rust
unsafe extern "C-unwind" fn Rf_duplicate(x: SEXP) -> SEXP
```

### `raw_ffi::Rf_eval`

```rust
unsafe extern "C" fn Rf_eval(expr: SEXP, env: SEXP) -> SEXP
```

### `raw_ffi::Rf_getAttrib`

```rust
unsafe extern "C-unwind" fn Rf_getAttrib(vec: SEXP, name: SEXP) -> SEXP
```

### `raw_ffi::Rf_install`

```rust
unsafe extern "C" fn Rf_install(name: *const c_char) -> SEXP
```

### `raw_ffi::Rf_isNewList`

```rust
unsafe extern "C-unwind" fn Rf_isNewList(s: SEXP) -> Rboolean
```

### `raw_ffi::Rf_lang2`

```rust
unsafe extern "C-unwind" fn Rf_lang2(s: SEXP, t: SEXP) -> SEXP
```

### `raw_ffi::Rf_lang3`

```rust
unsafe extern "C-unwind" fn Rf_lang3(s: SEXP, t: SEXP, u: SEXP) -> SEXP
```

### `raw_ffi::Rf_mkCharLenCE`

```rust
unsafe extern "C-unwind" fn Rf_mkCharLenCE(s: *const std::os::raw::c_char, len: i32, encoding: miniextendr_api::cetype_t) -> SEXP
```

### `raw_ffi::Rf_mkString`

```rust
unsafe extern "C" fn Rf_mkString(name: *const c_char) -> SEXP
```

### `raw_ffi::Rf_protect`

```rust
unsafe extern "C-unwind" fn Rf_protect(x: SEXP) -> SEXP
```

### `raw_ffi::Rf_setAttrib`

```rust
unsafe extern "C-unwind" fn Rf_setAttrib(vec: SEXP, name: SEXP, val: SEXP) -> SEXP
```

### `raw_ffi::Rf_unprotect`

```rust
unsafe extern "C-unwind" fn Rf_unprotect(n: i32)
```

### `raw_ffi::Rf_unprotect_ptr`

```rust
unsafe extern "C-unwind" fn Rf_unprotect_ptr(s: SEXP)
```

### `raw_ffi::Rf_xlength`

```rust
unsafe extern "C-unwind" fn Rf_xlength(x: SEXP) -> R_xlen_t
```

### `raw_ffi::SET_STRING_ELT`

```rust
unsafe extern "C-unwind" fn SET_STRING_ELT(x: SEXP, i: R_xlen_t, v: SEXP)
```

### `raw_ffi::SET_VECTOR_ELT`

```rust
unsafe extern "C-unwind" fn SET_VECTOR_ELT(x: SEXP, i: R_xlen_t, v: SEXP) -> SEXP
```

### `raw_ffi::STRING_ELT`

```rust
unsafe extern "C-unwind" fn STRING_ELT(x: SEXP, i: R_xlen_t) -> SEXP
```

### `raw_ffi::VECTOR_ELT`

```rust
unsafe extern "C-unwind" fn VECTOR_ELT(x: SEXP, i: R_xlen_t) -> SEXP
```

---

## Constants

### `LARGE_SIZES`

```rust
pub const LARGE_SIZES: &[usize] = _;
```

Extended sizes for scaling benchmarks (catches GC pressure / cache effects).

### `MATRIX_DIMS`

```rust
pub const MATRIX_DIMS: &[(usize, usize)] = _;
```

Matrix dimensions (nrow, ncol) for matrix/rarray benchmarks.

### `MATRIX_DIM_LABELS`

```rust
pub const MATRIX_DIM_LABELS: &[&str] = _;
```

Labels for matrix dimensions.

### `NAMED_LIST_SIZES`

```rust
pub const NAMED_LIST_SIZES: &[usize] = _;
```

Sizes for named list fixtures (used for map/list benchmarks).

### `NAMED_LIST_SIZE_LABELS`

```rust
pub const NAMED_LIST_SIZE_LABELS: &[&str] = _;
```

Labels for named list sizes.

### `SIZES`

```rust
pub const SIZES: &[usize] = _;
```

Standard size constants for benchmark parameterization.

### `SIZE_LABELS`

```rust
pub const SIZE_LABELS: &[&str] = _;
```

Size labels for divan output.

---

## Statics

### `raw_ffi::R_BlankString`

```rust
pub unsafe static R_BlankString: SEXP;
```

### `raw_ffi::R_GlobalEnv`

```rust
pub unsafe static R_GlobalEnv: SEXP;
```

### `raw_ffi::R_NaString`

```rust
pub unsafe static R_NaString: SEXP;
```

### `raw_ffi::R_NamesSymbol`

```rust
pub unsafe static R_NamesSymbol: SEXP;
```

### `raw_ffi::R_NilValue`

```rust
pub unsafe static R_NilValue: SEXP;
```
