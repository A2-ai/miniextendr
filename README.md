# `miniextendr`

Rust <-> R interoperability workspace. This repository contains the runtime
crates, proc macros, CLI/tooling, an example R package, and the helper package
used to scaffold standalone packages and monorepos.

## Workspace layout

### Core crates and tooling

- `miniextendr-api/` - Runtime crate: R FFI bindings, conversions, worker-thread
  routing, ALTREP, class systems, trait ABI, and macro re-exports.
- `miniextendr-macros/` - Proc macros such as `#[miniextendr]`,
  `#[r_ffi_checked]`, `ExternalPtr`, and `RNativeType`.
- `miniextendr-engine/` - Standalone R embedding engine for Rust-only binaries
  and integration tests.
- `miniextendr-cli/` - CLI for scaffolding, workflow, vendoring, and cargo
  operations.
- `miniextendr-bench/` - Benchmarks for conversions and feature-gated paths.
- `miniextendr-lint/` - Internal linter used during builds.
- `cargo-revendor/` - Standalone vendoring tool for hermetic/offline builds.
  It is intentionally excluded from the main Cargo workspace.

### R packages and test fixtures

- `rpkg/` - Example R package that exercises the Rust crates and the
  autoconf/configure flow.
- `minirextendr/` - R helper package for scaffolding, configure wiring,
  vendoring helpers, and cargo wrappers.
- `tests/cross-package/` - Producer/consumer R packages used to validate the
  cross-package trait ABI dispatch flow.

## What `miniextendr-api` provides

- FFI bindings to R's C API plus safer `SEXP`-oriented helpers.
- Rust <-> R conversions for common scalars, vectors, lists, matrices, and
  user-defined types.
- Main-thread execution inside `R_UnwindProtect` by default, with an optional
  worker-thread dispatch model for functions that opt in.
- ALTREP traits, registration helpers, and iterator-backed ALTREP data types.
- Env, S3, S4, S7, and R6 class generation from Rust impl blocks.
- Type-erased cross-package trait dispatch via tags and vtables.
- Adapter traits such as `RDebug`, `RDisplay`, `RHash`, `ROrd`, and
  `RPartialOrd`.
- Re-exports of the `miniextendr-macros` proc macros so most downstream crates
  only need one dependency.

Feature families in the runtime crate include:

- Runtime/build: `nonapi`, `rayon`, `connections`, `indicatif`, `vctrs`,
  `worker-thread`, `worker-default`, `fast-default`, `log`
- Serialization/data: `serde` (native R), `serde_json` (JSON), `borsh`,
  `arrow`, `datafusion`
- Ecosystem conversions: `either`, `uuid`, `regex`, `url`, `time`, `jiff`,
  `ordered-float`, `num-bigint`, `rust_decimal`, `num-complex`, `indexmap`,
  `bitflags`, `bitvec`, `ndarray`, `nalgebra`, `tinyvec`, `bytes`,
  `raw_conversions`, `aho-corasick`, `globset`, `toml`, `tabled`, `rand`,
  `rand_distr`, `num-traits`, `sha2`, `blake3`, `md5`, `zstd`
- Proc-macro defaults and diagnostics: `strict-default`, `coerce-default`,
  `r6-default`, `s7-default`, `doc-lint`, `growth-debug`,
  `macro-coverage`

See `miniextendr-api/README.md` and `docs/FEATURES.md` for the current feature
matrix.

## Example R package flow (`rpkg`)

`rpkg/` is a real R package that vendors Rust crates and builds a shared
library via cargo during `R CMD INSTALL`.

The high-level flow is:

1. `bootstrap.R` runs `configure`.
2. `configure` generates `src/Makevars` and writes
   `src/rust/.cargo/config.toml` for the detected source or tarball mode.
   `src/rust/Cargo.toml` remains an ordinary tracked source file.
3. `Makevars` builds the Rust static library, links the package shared library,
   then loads that library to write `R/miniextendr-wrappers.R` and the wasm32
   registration snapshot.

Generated artifacts have two different tracking policies:

- Track `rpkg/configure`, `rpkg/config.guess`, `rpkg/config.sub`,
  `rpkg/NAMESPACE`, and `rpkg/man/*.Rd`.
- Do not track `rpkg/R/miniextendr-wrappers.R` or
  `rpkg/src/rust/wasm_registry.rs`; they are regenerated during development
  installs but must be present on disk when the release tarball is built.

Common workflows from the repo root:

```sh
just configure        # generate Makevars + .cargo/config.toml (auto-detects mode)
just devtools-load    # compile Rust, regenerate wrappers, load rpkg
just devtools-test    # run rpkg tests
just vendor           # CRAN release prep: build rpkg/inst/vendor.tar.xz
just r-cmd-build      # build tarball (depends on `just vendor`)
just r-cmd-check      # build + check tarball
```

Install mode (source vs tarball) is auto-detected from
`inst/vendor.tar.xz` presence — no env var to set. See
[docs/CRAN_COMPATIBILITY.md](docs/CRAN_COMPATIBILITY.md).

## Development setup

Requirements:

- Rust toolchain (edition 2024)
- R with headers and `R` on `PATH`
- `autoconf` for regenerating `configure`
- `just` for the repo task wrappers

Common tasks:

```sh
just --list
just check
just check-features
just test
just clippy
just devtools-test
just cross-test
```

## Documentation

- `docs/README.md` - docs index for architecture, build system, features, and
  troubleshooting
- `miniextendr-cli/README.md` - CLI surface and command groups
- `miniextendr-api/README.md` - runtime crate overview and feature summary
- `minirextendr/README.md` - scaffolding and R-side workflow helpers
- `tests/cross-package/README.md` - end-to-end trait ABI example
- `cargo-revendor/README.md` - standalone vendoring tool for offline builds

## Publishing to CRAN

- Do not embed R in the CRAN-facing package shared library.
  `miniextendr-engine` is for standalone binaries/tests, not for R packages.
- Keep `nonapi` disabled unless you are prepared for CRAN checks to flag
  non-API symbol usage.
- Vendor Rust dependencies into the package tarball and include
  `inst/vendor.tar.xz` when building offline/CRAN releases.
- Keep `configure`, `config.guess`, `config.sub`, `NAMESPACE`, and `man/*.Rd`
  committed. Regenerate the gitignored wrapper and wasm registry before
  building the tarball so both are shipped in it.
- Run `R CMD check` on the release tarball before submission.

## Maintainer

- Regenerate `rpkg/configure` whenever `rpkg/configure.ac` changes.
- Update `config.guess` and `config.sub` from GNU config when needed.
- Keep wrapper generation aligned with macro output.
- Run both Rust and R validation paths before cutting releases.

## License

MIT (see `LICENSE-MIT`).
