# miniextendr-api

Core runtime crate for Rust <-> R interop.

This crate provides:

- FFI bindings to R's C API.
- Conversions between Rust and R types.
- The worker-thread pattern for panic isolation and Drop safety.
- ALTREP traits, registration helpers, and iterator-backed ALTREP data types.
- Env, S3, S4, S7, and R6 class generation from Rust impl blocks.
- Cross-package trait ABI support via tags and vtables.
- Re-exports of `miniextendr-macros` for ergonomic downstream use.

Most users should depend on this crate directly.

## Quick start

```rust
use miniextendr_api::miniextendr;

#[miniextendr]
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

Exports are automatically registered via linkme; no manual module declarations
are needed.

## Feature highlights

- **ALTREP** - Build custom ALTREP vectors with user-friendly data traits and
  opt-in low-level callbacks.
- **Iterators as ALTREP** - Built-in iterator-backed ALTREP data types with
  caching and optional coercion.
- **Connections** - Experimental R connection framework (feature-gated).
- **Class systems** - S3, S4, S7, and R6 impl-block methods plus env impl-block
  dispatch.
- **Trait ABI** - Type-erased, cross-package trait dispatch via tags + vtables.
- **Coerce** - Infallible and fallible numeric coercion with clear errors.
- **Generated R wrappers** - R functions and class methods are generated from
  Rust signatures and doc comments/roxygen tags.

## R wrapper generation

`#[miniextendr]` generates:

- C ABI wrappers (`C_<name>` symbols)
- R functions that call `.Call(...)` with the original argument names
- Class constructors and methods for impl-block types

R wrappers are generated from Rust doc comments (roxygen tags are extracted)
when the package calls the `miniextendr_write_wrappers` entrypoint during
build. The example package in this repo commits them to
`R/miniextendr-wrappers.R` so CRAN builds do not require code generation.

## Class systems and impl blocks

miniextendr supports multiple class systems from Rust impl blocks:

- **Env** - Environment-style `$` / `[[` dispatch for methods on an object.
- **S3** - Constructors use `structure(..., class = "Class")`, methods are
  `generic.class` with optional generic creation.
- **S4** - Uses `methods::setClass` and `methods::setMethod` with an external
  pointer slot for the Rust struct.
- **S7** - Uses `S7::new_class`, `S7::new_generic`, and `S7::method`.
- **R6** - Uses `R6::R6Class` with `$new()` and `$method()` entries.

Per-method attributes control behavior (constructor, finalizer, private/active
bindings for R6, method name overrides, and more).

## Trait ABI (cross-package dispatch)

`#[miniextendr]` can generate trait ABI metadata to allow type-erased dispatch
across package boundaries.

- Apply `#[miniextendr]` to the trait definition to generate tags + vtable
  types.
- Apply `#[miniextendr]` to `impl Trait for Type` to build vtables and wrapper
  metadata.

See `tests/cross-package/README.md` for an end-to-end example.

## Adapter traits

Built-in adapter traits provide blanket implementations for common std traits:

- `RDebug` - Debug string output (`debug_str()`, `debug_str_pretty()`)
- `RDisplay` - Display string output (`as_r_string()`)
- `RHash` - Hash computation (`r_hash() -> i64`)
- `ROrd` - Total ordering comparison (`r_cmp() -> -1/0/1`)
- `RPartialOrd` - Partial ordering (`r_partial_cmp() -> Option<i32>`)

Any type implementing the corresponding std trait automatically gets these
methods:

```rust
#[derive(Debug, Hash, Ord, PartialOrd, Eq, PartialEq, ExternalPtr)]
struct Version(u32, u32, u32);

#[miniextendr]
impl RDebug for Version {}

#[miniextendr]
impl ROrd for Version {}
```

See `ADAPTER_TRAITS.md` and `ADAPTER_COOKBOOK.md` for patterns and recipes.

## ALTREP support

ALTREP support is built around a two-layer trait model:

- **Data traits** (`Alt*Data`) expose ergonomic `&self` methods like `len()`
  and `elt()` with optional fast paths such as `get_region`, `as_slice`, and
  `sum`.
- **FFI traits** (`Alt*`) expose raw `SEXP` callbacks. Only methods that are
  explicitly enabled are registered with R, so defaults remain conservative.

Registration is handled via `#[miniextendr]` on a one-field wrapper type.
Classes are automatically registered at load time.

### Iterators as ALTREP

Iterator-backed ALTREP data types are provided for common vector kinds:

- Integer, real, logical, raw, string, complex, and list vectors
- Caching iterators for repeatable reads
- Explicit or inferred length (`ExactSizeIterator`)
- Coercing variants for integer/real iterators
- `Option<T>` iterator support that maps `None` to the appropriate missing
  value

## Conversions and coercion

This crate exposes conversion traits for Rust <-> R data:

- `IntoR` for Rust -> R conversion
- `TryFromSexp` for R -> Rust conversion
- `Coerce<R>` for infallible, widening conversions
- `TryCoerce<R>` for fallible conversions with explicit errors such as
  `Overflow`, `PrecisionLoss`, and `NaN`

`#[miniextendr(coerce)]` enables automatic coercion on function parameters,
including `Vec<T>`. Overflow and precision failures surface as R errors.

## Threading and safety

R uses `longjmp` for errors, which can bypass Rust destructors. The default
pattern is:

- Run Rust logic on a worker thread where `catch_unwind` is reliable.
- Marshal R API calls back to the main R thread via `with_r_thread`.

Most FFI wrappers are main-thread routed via `#[r_ffi_checked]`. Use
`*_unchecked` variants only when you have explicitly arranged safe context.

### Calling R from non-main threads (unsafe)

With feature `nonapi`, miniextendr can disable R's stack checking to allow
calls from other threads. Utilities include:

- `spawn_with_r` / `scope_with_r` / `RThreadBuilder`
- `StackCheckGuard` or `with_stack_checking_disabled`

R is still not thread-safe; you must serialize all R API access.

## Rayon integration (`rayon` feature)

Rayon helpers allow parallel Rust computation with R-safe boundaries:

- `with_r_vec<T>` pre-allocates and fills R vectors
- `Vec<T>` parallel collection followed by `IntoR`
- `reduce::*` helpers for sum/min/max/mean returning R scalars
- `perf::*` helpers for Rayon pool info
- `rayon_bridge::rayon` re-export to avoid version mismatches

R API calls must stay outside parallel closures. Use `with_r_thread` before or
after parallel work when you need to touch R.

## Connections (`connections` feature)

An experimental framework for defining custom R connections. This API is
unstable in R itself; use it only when you control the runtime environment.

## Feature flags

Default features:

- `doc-lint` - Lints exported docs/roxygen during macro expansion.
- `refcount-fast-hash` - Uses `ahash` for refcount-protect arenas.

### Runtime and build behavior

| Feature | Description |
|---------|-------------|
| `nonapi` | Enables non-API R symbols such as stack controls and mutable `DATAPTR`. May break with R updates or trigger CRAN warnings. |
| `rayon` | Parallel iterators and Rayon-aware helpers. |
| `connections` | Experimental custom R connection framework. |
| `indicatif` | Progress bars routed through the R console. Requires `nonapi`. |
| `vctrs` | Access to the vctrs C API and the `#[derive(Vctrs)]` proc macro. |
| `worker-thread` | Enables the worker-thread infrastructure without changing proc-macro defaults by itself. |
| `default-worker` | Makes worker-thread execution the proc-macro default and enables `worker-thread`. |
| `log` | Routes Rust `log` output to the R console during package init. |

### Proc-macro defaults

| Feature | Description |
|---------|-------------|
| `default-strict` | Turns on strict numeric conversion defaults in the macros. |
| `default-coerce` | Makes `#[miniextendr(coerce)]` the default macro behavior. |
| `default-r6` | Makes generated classes default to R6. |
| `default-s7` | Makes generated classes default to S7. |

### Serialization and columnar data

| Feature | Surface | Description |
|---------|---------|-------------|
| `serde` | `RSerializeNative`, `RDeserializeNative` | Native Rust <-> R serialization with no JSON intermediate. |
| `serde_json` | `RSerialize`, `RDeserialize` | JSON-based serialization helpers built on top of serde. |
| `borsh` | `Borsh<T>` | Binary serialization to and from raw vectors. |
| `arrow` | Arrow arrays, buffers, schemas, record batches | Arrow integration for zero-copy and columnar workflows. |
| `datafusion` | `RSessionContext` | SQL querying over Arrow-backed data; depends on `arrow` and Tokio. |

### Type ecosystem integrations

| Feature | Types / APIs | Notes |
|---------|---------------|-------|
| `either` | `Either<L, R>` | Tries `L` then `R`. |
| `uuid` | `Uuid`, `Vec<Uuid>` | `character` conversion. |
| `regex` | `Regex` | Compile regexes from R strings. |
| `url` | `Url`, `Vec<Url>` | Strict URL validation. |
| `time` | `OffsetDateTime`, `Date` | `POSIXct` / `Date` conversions. |
| `ordered-float` | `OrderedFloat<f64>` | Total ordering for floats. |
| `num-bigint` | `BigInt`, `BigUint` | Arbitrary-precision integer conversion via strings. |
| `rust_decimal` | `Decimal` | Fixed-point decimal conversion. |
| `num-complex` | `Complex<f64>` | Native R complex support. |
| `indexmap` | `IndexMap<String, T>` | Named list conversion with stable insertion order. |
| `bitflags` | `RFlags<T>` | Bitflags <-> integer helpers. |
| `bitvec` | `RBitVec` | Bit vectors <-> logical vectors. |
| `ndarray` | `Array*`, `ArrayView*` | R arrays/matrices <-> ndarray. |
| `nalgebra` | `DVector`, `DMatrix` | R vectors/matrices <-> nalgebra. |
| `tinyvec` | `TinyVec`, `ArrayVec` | Compact vector conversions. |
| `num-traits` | `RNum`, `RSigned`, `RFloat` | Numeric adapter traits. |
| `bytes` | `RBuf`, `RBufMut` | Byte-buffer helpers. |
| `rand` | `RRng`, `RDistributions` | Wraps R's RNG with rand traits. |
| `rand_distr` | Re-export of `rand_distr` | Extra distributions; requires `rand`. |
| `aho-corasick` | `AhoCorasick`, helpers | Multi-pattern search. |
| `toml` | `TomlValue`, helpers | TOML parsing/serialization. |
| `tabled` | `table_to_string` | Table formatting helpers. |
| `raw_conversions` | `Raw<T>`, `RawSlice<T>` | POD <-> raw vectors via bytemuck. |
| `sha2` | Hash helpers | SHA-256 / SHA-512 helpers. |

### Diagnostics and developer support

| Feature | Description |
|---------|-------------|
| `macro-coverage` | Emits extra expansion artifacts for `cargo expand` auditing. |
| `debug-preserve` | Enables preserve-count diagnostics for tests and benchmarks. |
| `growth-debug` | Tracks unexpected collection growth events. |

See `Cargo.toml` and `docs/FEATURES.md` for the definitive feature list.

## Publishing to CRAN

`miniextendr-api` is CRAN-compatible when used correctly:

- Do not enable `nonapi` unless you are prepared for CRAN to flag non-API
  symbol usage.
- Ensure all Rust dependencies are vendored in your R package tarball.
- Commit generated wrappers (for this repo: `R/miniextendr-wrappers.R`) before
  release.
- Run `R CMD check` on the release tarball.

For embedding R in standalone binaries or integration tests, use
`miniextendr-engine` instead of embedding inside an R package.

## Maintainer

- Keep FFI bindings aligned with current R headers.
- Update conversion behavior tests when R semantics change.
- Ensure roxygen/doc extraction remains in sync with macro behavior.
- Track non-API symbols behind feature gates.
- Verify thread checks and worker-thread behavior across R versions.
