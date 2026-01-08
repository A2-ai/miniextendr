# `miniextendr`

Rust <-> R interoperability workspace. This repo contains the core Rust crates,
an example R package, and build tooling for embedding R, exporting Rust
functions into R, ALTREP support, and safe threading patterns.

## Workspace layout

### Rust crates

- `miniextendr-api/` - Core runtime: R FFI bindings, conversions, worker-thread
  pattern, ALTREP traits, and macro re-exports.
- `miniextendr-macros/` - Procedural macros (`#[miniextendr]`,
  `miniextendr_module!`, `#[r_ffi_checked]`, `ExternalPtr`, `RNativeType`).
- `miniextendr-engine/` - Standalone R embedding engine for Rust-only
  binaries/tests; uses non-API R internals.
- `miniextendr-bench/` - Benchmarks for conversions and R interop.
- `miniextendr-lint/` - Internal build-time linter (proc-macro helper); not
  intended for external consumption.

### R packages and tooling

- `rpkg/` – Example R package that exercises the Rust crates. Includes the
  autoconf build, vendoring, and wrapper generation flow.
- `minirextendr/` – Helper R package for scaffolding, autoconf/configure wiring,
  vendoring helpers, and cargo wrappers for R package workflows.
- `tests/cross-package/` – Producer/consumer R packages used to validate the
  cross-package trait ABI dispatch flow.
- `rpkg_0.0.0.9000.tar.gz` – Example built tarball (when present).

## Crate details

### `miniextendr-api`

Core runtime crate for R interop.

Highlights:

- Rust <-> R conversions for common types.
- Worker-thread pattern for panic isolation and Drop safety under R longjmp.
- ALTREP traits and utilities.
- Adapter traits (`RDebug`, `RDisplay`, `RHash`, `ROrd`, `RPartialOrd`) with
  blanket impls for std traits.
- Re-exports macros from `miniextendr-macros` so downstream crates can depend
  on a single crate.

Feature flags (core):

| Feature | Description |
|---------|-------------|
| `nonapi` | Non-API R symbols (stack controls, mutable `DATAPTR`). May break with R updates. |
| `rayon` | Parallel iterators via Rayon. Adds `RParallelIterator`, `RParallelExtend`. |
| `connections` | Experimental R connection framework. **Unstable R API.** |
| `indicatif` | Progress bars via R console. Requires `nonapi`. |
| `vctrs` | Access to vctrs C API (`obj_is_vector`, `short_vec_size`, `short_vec_recycle`). |

Feature flags (type conversions):

| Feature | Rust Type | R Type |
|---------|-----------|--------|
| `either` | `Either<L, R>` | Tries L then R |
| `uuid` | `Uuid`, `Vec<Uuid>` | `character` |
| `regex` | `Regex` | `character(1)` |
| `url` | `Url`, `Vec<Url>` | `character` |
| `time` | `OffsetDateTime`, `Date` | `POSIXct`, `Date` |
| `ordered-float` | `OrderedFloat<f64>` | `numeric` |
| `num-bigint` | `BigInt`, `BigUint` | `character` |
| `rust_decimal` | `Decimal` | `character` |
| `num-complex` | `Complex<f64>` | `complex` |
| `indexmap` | `IndexMap<String, T>` | named `list` |
| `bitflags` | `RFlags<T>` | `integer` |
| `bitvec` | `RBitVec` | `logical` |
| `ndarray` | `Array1`–`Array6`, views | R vectors/matrices |
| `nalgebra` | `DVector`, `DMatrix` | R vectors/matrices |
| `rand` / `rand_distr` | `RRng`, `RDistributions` | R's RNG with rand traits |
| `serde` | `RSerialize`, `RDeserialize` | JSON via serde_json |
| `serde_r` | `RSerializeNative`, `RDeserializeNative` | Direct Rust ↔ R |
| `num-traits` | `RNum`, `RSigned`, `RFloat` | Generic numeric ops |
| `bytes` | `RBuf`, `RBufMut` | Byte buffer ops |
| `aho-corasick` | `AhoCorasick` | Multi-pattern search |
| `toml` | `TomlValue` | TOML parsing |
| `tabled` | `table_to_string` | Table formatting |
| `sha2` | `sha256_str`, `sha512_bytes` | Cryptographic hashing |
| `raw_conversions` | `Raw<T>`, `RawSlice<T>` | POD ↔ raw vectors |

### `miniextendr-macros`

Proc-macro crate that generates the R-facing glue.

Main macros:

- `#[miniextendr]` – exports Rust functions to R, generates wrappers and
  conversion glue.
- `miniextendr_module!` – registers exported functions and ALTREP types.
- `#[r_ffi_checked]` – routes R FFI calls to the main thread when needed.
- Derives: `ExternalPtr`, `RNativeType`.

### `miniextendr-engine`

Standalone embedding crate for initializing R in Rust-only binaries/tests.

Notes:

- Uses `Rembedded.h`/`Rinterface.h` (non-API), so it is **not** intended for
  use inside R packages.
- Centralizes `R_HOME` discovery/linking and avoids double-calling
  `setup_Rmainloop()` while keeping initialization order consistent.
- Intentionally does not call `Rf_endEmbeddedR` on drop (non-reentrant cleanup).

### `miniextendr-bench`

Benchmarks for translation and conversion costs. Uses `divan` and depends on
`miniextendr-engine` to embed R.

Run:

```sh
cd miniextendr-bench
cargo bench --bench translate
```

### `miniextendr-lint`

Internal proc-macro tooling used during builds. It is part of the workspace
but not intended as a public crate.

## R package (`rpkg`) build flow

`rpkg/` is a real R package that vendors the Rust crates and builds a shared
library via `cargo` during `R CMD INSTALL`.

Key files that must be committed:

- `rpkg/configure` (generated by autoconf from `configure.ac`).
- `rpkg/R/miniextendr_wrappers.R` (generated by the `document` binary).

Developer workflow (from repo root):

```sh
just configure       # vendor deps + run rpkg/configure for monorepo dev
just devtools-load   # build Rust + generate wrappers + load R package
just r-cmd-check     # run R CMD check
```

Standalone R package notes:

- `bootstrap.R` runs before build and only invokes `rpkg/configure` to support
  monorepos where the R package is a subdirectory of the Rust workspace.
- `configure` generates `src/Makevars` and Rust config for `cargo build`.

## Development setup

Requirements:

- Rust toolchain (edition 2024).
- R (with headers and `R` on PATH).
- `autoconf` (for `rpkg/configure` regeneration).
- `just` (recommended for repo tasks).

Common tasks:

```sh
just --list
just check
just test
just clippy
just devtools-test
```

More tasks (minirextendr helpers, cross-package tests, templates, vendor/lint
sync) are listed in `justfile` and `tests/cross-package/justfile`.

## Additional docs

### Core concepts
- `SAFETY.md` – Thread safety invariants, FFI safety, and memory model.
- `THREADS.md` – Worker-thread model and safety notes.
- `RAYON.md` – Rayon integration guide with patterns and performance tips.
- `ENTRYPOINT.md` – R_init_* requirements and initialization order.

### Type system and conversions
- `COERCE.md` – Coercion rules and conversion behavior.
- `COERCE_AND_INTO_R_REVIEW.md` – Review notes on coercion and `IntoR` behavior.
- `TRAIT_AS_R.md` – Trait ABI and cross-package dispatch.
- `ADAPTER_TRAITS.md` – Exporting external traits to R via adapter pattern.
- `ADAPTER_COOKBOOK.md` – Practical recipes: iterators, serde, IO, comparison, hashing.

### Build and linking
- `LINKING.md` – Linking strategy for R packages and standalone binaries.
- `VENDOR.md` – Vendoring and sync checks.
- `ENGINE.md` – Embedding engine notes.

### Reference
- `docs.md` – Architecture overview and command reference.
- `NONAPI.md` – Non-API R symbols and policy.
- `TRACK_CALLER.md` – `#[track_caller]` usage and error reporting details.
- `altrep.md` – ALTREP notes and design context.
- `MAINTAINER.md` – Maintenance and release checklist.

## Threading model (two modes)

1) **Default worker-thread pattern**
   - `#[miniextendr]` runs Rust code on a worker thread so panics and Drops are
     safe even if R errors via longjmp. R API calls are marshalled back to the
     main thread.

2) **Opt-in non-main-thread R calls (unsafe)**
   - `miniextendr_api::thread` under feature `nonapi` disables stack checking.
   - You must still serialize R access; R is not thread-safe.
   - Non-API usage is feature gated (e.g., `R_CStack*`, mutable `DATAPTR`).

## Publishing to CRAN

This workspace supports CRAN-compatible packages through `rpkg/`, but only
under the following constraints:

- **Do not embed R** in a CRAN-facing package. `miniextendr-engine` is for
  Rust-only binaries and tests, not for package shared libraries.
- **Avoid non-API symbols** unless you are prepared for CRAN checks to flag
  them. Keep `nonapi` disabled by default.
- **Vendor Rust dependencies** into `rpkg/src/vendor/` and include them in the
  source tarball.
- **Commit generated artifacts** required by CRAN: `rpkg/configure`,
  `rpkg/config.guess`, `rpkg/config.sub`, and `rpkg/R/miniextendr_wrappers.R`.
- **Run `R CMD check`** on the release tarball before submission.

See `rpkg/README.md` for the complete CRAN workflow.

## Maintainer

- Keep non-API usage feature-gated and documented in code.
- Regenerate `rpkg/configure` whenever `rpkg/configure.ac` changes.
- Update `rpkg/config.guess` and `rpkg/config.sub` from GNU config when needed
  (`https://cgit.git.savannah.gnu.org/cgit/config.git/tree/`).
- Ensure wrapper generation remains in sync with macro behavior.
- Run CI/local checks across Rust and R tooling before releases.

## Notes on release builds

The workspace keeps `debug-assertions = true` in the release profile (see
`Cargo.toml`) so debug-only safety checks remain enabled.

## License

MIT (see `LICENSE-MIT`).
