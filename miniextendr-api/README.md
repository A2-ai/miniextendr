# miniextendr-api

Core runtime crate for Rust <-> R interop.

This crate provides:

- FFI bindings to R’s C API.
- Conversions between Rust and R types.
- The worker‑thread pattern for panic isolation and Drop safety.
- ALTREP traits, registration helpers, and iterator‑backed ALTREP data types.
- Re-exports of `miniextendr-macros` for ergonomic use.

Most users should depend on this crate directly.

## Quick start

```rust
use miniextendr_api::miniextendr;

#[miniextendr]
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

Register exports in your package/module:

```rust
use miniextendr_api::miniextendr_module;

miniextendr_module! {
    mod mypkg;
    fn add;
}
```

## Feature highlights

- **ALTREP** – build custom ALTREP vectors with user‑friendly data traits and
  opt‑in low‑level callbacks.
- **Iterators as ALTREP** – built‑in iterator‑backed ALTREP data types with
  caching and optional coercion.
- **Connections** – experimental R connection framework (feature‑gated).
- **Class systems** – S3, S4, S7, and R6 impl‑block methods plus an env
  impl‑block for `$`/`[[` dispatch.
- **Trait ABI** – type‑erased, cross‑package trait dispatch via tags + vtables.
- **Coerce** – infallible and fallible numeric coercion with clear errors.
- **Generated R wrappers** – R functions and class methods are generated from
  Rust signatures and doc comments/roxygen tags.

## R wrapper generation

`#[miniextendr]` and `miniextendr_module!` generate:

- C‑ABI wrappers (`C_<name>` symbols)
- R functions that call `.Call(...)` with the original argument names
- Class constructors and methods for impl‑block types

R wrappers are generated from Rust doc comments (roxygen tags are extracted)
by the `document` binary during package build. The generated output is
committed to `R/miniextendr_wrappers.R` so CRAN builds do not require codegen.

## Class systems and impl blocks

miniextendr supports multiple class systems from Rust impl blocks:

- **Env** – environment‑style `$`/`[[` dispatch for methods on an object.
- **S3** – constructors use `structure(..., class = "Class")`, methods are
  `generic.class` with optional generic creation.
- **S4** – uses `methods::setClass` and `methods::setMethod` with an external
  pointer slot for the Rust struct.
- **S7** – uses `S7::new_class`, `S7::new_generic`, and `S7::method`.
- **R6** – uses `R6::R6Class` with `$new()` and `$method()` entries.

Per‑method attributes control behavior (constructor, finalizer, private/active
bindings for R6, method name overrides, etc.).

## Trait ABI (cross-package dispatch)

`#[miniextendr]` can generate trait ABI metadata to allow type‑erased dispatch
across package boundaries.

- Apply `#[miniextendr]` to the trait definition to generate tags + vtable types.
- Apply `#[miniextendr]` to `impl Trait for Type` to build vtables and wrappers.
- Register `impl Trait for Type;` entries in `miniextendr_module!`.

See `tests/cross-package/README.md` for an end‑to‑end example.

## Adapter traits

Built-in adapter traits provide blanket implementations for common std traits:

- `RDebug` – Debug string output (`debug_str()`, `debug_str_pretty()`)
- `RDisplay` – Display string output (`as_r_string()`)
- `RHash` – Hash computation (`r_hash() -> i64`)
- `ROrd` – Total ordering comparison (`r_cmp() -> -1/0/1`)
- `RPartialOrd` – Partial ordering (`r_partial_cmp() -> Option<i32>`)

Any type implementing the corresponding std trait automatically gets these methods:

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

ALTREP support is built around a two‑layer trait model:

- **Data traits** (`Alt*Data`) expose ergonomic `&self` methods like `len()` and
  `elt()` with optional fast‑paths (e.g., `get_region`, `as_slice`, `sum`).
- **FFI traits** (`Alt*`) expose raw `SEXP` callbacks. Only methods that are
  explicitly enabled are registered with R, so defaults remain safe.

Registration is handled via `#[miniextendr]` on a one‑field wrapper type and
`miniextendr_module!` to register the class at load time.

### Iterators as ALTREP

Iterator‑backed ALTREP data types are provided for common vector kinds:

- Integer, real, logical, raw, string, complex, and list vectors.
- Iterators are cached as elements are accessed to support repeatable reads.
- Length is explicit or inferred from `ExactSizeIterator`.
- Coercing variants exist for integer/real (including `bool → i32`).
- `Option<T>` iterators map `None` to NA values where appropriate.

## Conversions and coercion

This crate exposes conversion traits for Rust <-> R data:

- `IntoR` / `FromR` for standard conversions.
- `Coerce<R>` for infallible, widening conversions.
- `TryCoerce<R>` for fallible conversions with explicit errors
  (`Overflow`, `PrecisionLoss`, `NaN`).

`#[miniextendr(coerce)]` enables automatic coercion on function parameters
(including `Vec<T>`). Overflow/precision failures surface as R errors.

## Threading and safety

R uses `longjmp` for errors, which can bypass Rust destructors. The default
pattern is:

- Run Rust logic on a **worker thread** where `catch_unwind` is reliable.
- Marshal R API calls back to the **main R thread** via `with_r_thread`.

Most FFI wrappers are **main-thread routed** via `#[r_ffi_checked]`. Use
`*_unchecked` variants only when you have explicitly arranged safe context.

### Calling R from non‑main threads (unsafe)

With feature `nonapi`, miniextendr can disable R’s stack checking to allow
calls from other threads. Utilities include:

- `spawn_with_r` / `scope_with_r` / `RThreadBuilder` for configured threads
- `StackCheckGuard` or `with_stack_checking_disabled` for manual control

R is still **not thread‑safe**; you must serialize all R API access.

## Rayon integration (`rayon` feature)

Rayon helpers allow parallel Rust computation with R‑safe boundaries:

- `with_r_vec<T>` pre‑allocates and fills R vectors (zero‑copy).
- `Vec<T>` collects in parallel (Rayon `FromParallelIterator`) and converts via `IntoR`.
- `reduce::*` provides parallel sum/min/max/mean helpers that return R scalars.
- `perf::*` exposes Rayon pool info (thread count, thread index).
- `rayon_bridge::rayon` re‑exports Rayon to avoid version mismatches.

R API calls must be outside parallel closures. Use `with_r_thread` before/after
parallel work when you need to touch R.

## Connections (`connections` feature)

An experimental framework for defining custom R connections. This API is
unstable in R itself; use only when you control the runtime environment.

## Feature Flags

### Core Features

| Feature | Description |
|---------|-------------|
| `nonapi` | Non-API R symbols (stack controls, mutable `DATAPTR`). May break with R updates. |
| `rayon` | Parallel iterators via Rayon. Adds `RParallelIterator`, `RParallelExtend`. |
| `connections` | Experimental R connection framework. **Unstable R API.** |
| `indicatif` | Progress bars via R console. Requires `nonapi`. |
| `vctrs` | Access to vctrs C API (`obj_is_vector`, `short_vec_size`, `short_vec_recycle`). |

### Type Conversions (Scalars & Vectors)

| Feature | Rust Type | R Type | Notes |
|---------|-----------|--------|-------|
| `either` | `Either<L, R>` | Tries L then R | Union-like dispatch |
| `uuid` | `Uuid`, `Vec<Uuid>` | `character` | UUID ↔ string |
| `regex` | `Regex` | `character(1)` | Compiles pattern from R |
| `url` | `Url`, `Vec<Url>` | `character` | Validated URLs |
| `time` | `OffsetDateTime`, `Date` | `POSIXct`, `Date` | Date/time conversions |
| `ordered-float` | `OrderedFloat<f64>` | `numeric` | NaN-orderable floats |
| `num-bigint` | `BigInt`, `BigUint` | `character` | Arbitrary precision via strings |
| `rust_decimal` | `Decimal` | `character` | Fixed-point decimals |
| `num-complex` | `Complex<f64>` | `complex` | Native R complex support |
| `indexmap` | `IndexMap<String, T>` | named `list` | Preserves insertion order |
| `bitflags` | `RFlags<T>` | `integer` | Bitflags ↔ integer |
| `bitvec` | `RBitVec` | `logical` | Bit vectors ↔ logical |

### Matrix & Array Libraries

| Feature | Types | Conversions |
|---------|-------|-------------|
| `ndarray` | `Array1`–`Array6`, `ArrayD`, views | R vectors/matrices ↔ ndarray |
| `nalgebra` | `DVector`, `DMatrix` | R vectors/matrices ↔ nalgebra |

### Serialization

| Feature | Traits/Modules | Description |
|---------|----------------|-------------|
| `serde` | `RSerialize`, `RDeserialize` | JSON serialization via serde_json |
| `serde_r` | `RSerializeNative`, `RDeserializeNative` | Direct Rust ↔ R (no JSON) |
| `serde_full` | Both above | Enables `serde` + `serde_r` |

### Adapter Traits (Generic Operations)

| Feature | Traits | Use Case |
|---------|--------|----------|
| `num-traits` | `RNum`, `RSigned`, `RFloat` | Generic numeric operations |
| `bytes` | `RBuf`, `RBufMut` | Byte buffer operations |

### Text & Data Processing

| Feature | Types/Functions | Description |
|---------|-----------------|-------------|
| `aho-corasick` | `AhoCorasick`, `aho_compile` | Fast multi-pattern string search |
| `toml` | `TomlValue`, `toml_from_str` | TOML parsing and serialization |
| `tabled` | `table_to_string` | ASCII/Unicode table formatting |
| `sha2` | `sha256_str`, `sha512_bytes` | Cryptographic hashing |

### Random Number Generation

| Feature | Types | Description |
|---------|-------|-------------|
| `rand` | `RRng`, `RDistributions` | Wraps R's RNG with `rand` traits |
| `rand_distr` | Re-exports `rand_distr` | Additional distributions (Normal, Exp, etc.) |

### Binary Data

| Feature | Types | Description |
|---------|-------|-------------|
| `raw_conversions` | `Raw<T>`, `RawSlice<T>` | POD types ↔ raw vectors via bytemuck |

## Publishing to CRAN

`miniextendr-api` is **CRAN‑compatible** when used correctly:

- Do **not** enable `nonapi` unless you are prepared for CRAN checks to flag
  non‑API symbol usage.
- Ensure all Rust dependencies are vendored in your R package tarball.
- Commit generated wrappers (`R/miniextendr_wrappers.R`) before release.
- Run `R CMD check` on the release tarball.

For embedding R in standalone binaries or integration tests, use
`miniextendr-engine` instead of embedding inside an R package.

## Maintainer

- Keep FFI bindings aligned with current R headers.
- Update conversion behavior tests when R semantics change.
- Ensure roxygen/doc extraction remains in sync with macro behavior.
- Track any non‑API symbols in a feature‑gated manner.
- Verify thread checks and worker‑thread behavior across R versions.
