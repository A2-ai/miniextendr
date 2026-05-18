---
name: miniextendr-architecture
description: Use when the user asks about the miniextendr codebase structure, crate graph, build pipeline, registration system, the cdylib-to-staticlib double-link, distributed_slice tables, the install-mode latch, or "how does X get from Rust to R". Also use when the user is navigating the repo for the first time and needs orientation.
---

# miniextendr Architecture

miniextendr is a Rust-R interoperability framework structured as a multi-crate
workspace. Understanding how it fits together is prerequisite knowledge for
working on any subsystem — the build pipeline, the macro system, the type
conversions, and the ALTREP support all interlock at specific seams.

## When to use this skill

- "How does this codebase fit together?"
- "What is the relationship between miniextendr-api and miniextendr-macros?"
- "How does a Rust function end up callable from R?"
- "What is the cdylib to staticlib double-link?"
- "What are distributed_slice / linkme tables and why does miniextendr use them?"
- "What is the install-mode latch and why does it matter?"
- "Where does registration happen?"
- "Where do I look for the build pipeline?"
- "Which crate owns which concern?"

## Key concepts

### Workspace layout

The repository root is a Cargo workspace. The primary crates are:

- `miniextendr-api/` — the runtime library. All R API contact lives here:
  FFI bindings, type conversion traits (`IntoR`, `TryFromSexp`), ExternalPtr
  wrappers, ALTREP infrastructure, GC protection, the worker thread, and error
  transport.
- `miniextendr-macros/` — procedural macros. `#[miniextendr]` on a function or
  impl block generates C-callable wrappers and R wrapper fragments. Derives for
  `ExternalPtr`, `DataFrameRow`, `Vctrs`, ALTREP structs live here too.
- `miniextendr-engine/` — codegen engine. Provides the
  `miniextendr_write_wrappers` function consumed by the cdylib phase of the
  build. Reads the distributed_slice tables and emits
  `R/miniextendr-wrappers.R`.
- `miniextendr-lint/` — build-time static analysis. Runs via `build.rs` during
  `cargo check`. Enforces MXL-coded rules (MXL300, MXL301, MXL112, etc.).
- `miniextendr-bench/` — benchmarks (separate workspace member).
- `miniextendr-cli/` — CLI helpers.
- `cargo-revendor/` — standalone cargo subcommand, intentionally excluded from
  the main workspace.

Non-crate directories:

- `rpkg/` — the exemplar R package (`library(miniextendr)`). Its Rust crate
  lives at `rpkg/src/rust/` and depends on the workspace crates above.
- `minirextendr/` — pure-R scaffolding helper for end-user packages.
- `tests/cross-package/` — two R packages (`producer.pkg` / `consumer.pkg`)
  exercising cross-package trait dispatch over ExternalPtr.
- `docs/` — narrative documentation source; `scripts/docs-to-site.sh` syncs
  into `site/content/manual/`.

The dependency direction:

```
miniextendr-macros    miniextendr-engine
(proc macros)         (codegen: write_wrappers)
      |                       |
      +------+----------------+
             |
      miniextendr-api
      (runtime library)
             |
      rpkg/src/rust     (example consumer)
      end-user packages (any package using miniextendr)
```

Neither `miniextendr-macros` nor `miniextendr-engine` imports from
`miniextendr-api` at macro-expansion time. The generated code it emits
does reference `miniextendr_api::` paths, but the macro crate itself only
depends on proc-macro support crates (`syn`, `quote`, `proc-macro2`).

### Build pipeline

The double-link pipeline is the most unusual aspect of the build system and the
key to understanding why wrapper generation works without a separate codegen
tool:

```
Makevars.in  ->  configure  ->  Makevars
                                   |
                     cargo rustc --crate-type cdylib
                                   |
                              dyn.load in R
                                   |
                         miniextendr_write_wrappers()
                         (reads distributed_slice tables,
                          emits R/miniextendr-wrappers.R)
                                   |
                     cargo rustc --crate-type staticlib
                                   |
                               final .so
```

Step by step:

1. `configure` runs from `rpkg/configure.ac`, generating `src/Makevars` from
   `src/Makevars.in` and `.cargo/config.toml` from
   `src/rust/cargo-config.toml.in`.
2. R's build system invokes `Makevars`. The first cargo invocation compiles the
   Rust crate as a cdylib.
3. `Makevars` then calls into R to `dyn.load` the cdylib and invoke
   `miniextendr_write_wrappers()`. This function walks the `MX_R_WRAPPERS`
   distributed_slice and writes `R/miniextendr-wrappers.R`.
4. A second cargo invocation compiles the crate as a staticlib for the final
   installed shared object.
5. `stub.c` provides the minimal C translation unit R's build system requires.
   It declares `extern const char miniextendr_force_link`, which references a
   symbol emitted by `miniextendr_init!()`. With `codegen-units = 1`, this
   pulls the entire user crate out of the staticlib archive — no
   `-force_load` or `--whole-archive` needed.

The cdylib-to-staticlib double-link is what makes wrapper generation possible:
the cdylib boots far enough into R to call back into Rust to emit the R-side
wrappers, then the staticlib relinks for the final installed object.

### Registration via distributed_slice

`miniextendr` uses the `linkme` crate to collect self-registration entries at
link time. Every `#[miniextendr]` item emits entries into one or more of these
static slices, declared in `miniextendr-api/src/registry.rs`:

- `MX_CALL_DEFS` — `R_CallMethodDef` entries. These are the C-callable function
  pointers registered with R during `R_init_*`.
- `MX_R_WRAPPERS` — R wrapper code fragments with priority ordering. Consumed
  by `miniextendr_write_wrappers()` during the cdylib phase to produce
  `R/miniextendr-wrappers.R`.
- `MX_ALTREP_REGISTRATIONS` — ALTREP class registration functions, called once
  at package init.
- `MX_MATCH_ARG_CHOICES` — placeholder-to-choices map for R formal defaults.
- `MX_MATCH_ARG_PARAM_DOCS` — placeholder-to-@param docs map.
- `MX_CLASS_NAMES` — cross-type R class name resolution.
- `MX_S7_SIDECAR_PROPS` — S7 property sidecar entries.
- `MX_TRAIT_DISPATCH` — trait dispatch entries for universal query.

`RWrapperPriority` controls evaluation order in the generated R file: Sidecar
before Class before Function before TraitImpl before Vctrs. S7 classes are
further topologically sorted (parent before child) inside `collect_r_wrappers()`
in the engine crate.

The initialization sequence is consolidated in `miniextendr-api/src/init.rs`.
`package_init()` is called from the `R_init_<pkg>` entry point generated by
`miniextendr_init!(pkg)`. It installs the panic hook, records the main thread
ID, asserts UTF-8 locale, sets the ALTREP package name, registers mx_abi
C-callables, and finally registers all `#[miniextendr]` routines and ALTREP
classes.

### The install-mode latch

`rpkg/inst/vendor.tar.xz` is the single signal that flips `configure` into
tarball (offline CRAN) mode.

| Mode | Triggered when | What configure does |
|------|---------------|---------------------|
| Source | tarball absent | No vendoring. In monorepo: writes `[patch."git+url"]` → workspace siblings. |
| Tarball | tarball present | Unpacks tarball, writes `[source]` replacement → `vendored-sources`, enables offline build. |

The tarball is gitignored (since 2026-04-18). CI regenerates it per-build via
`just vendor`. Locally, `just r-cmd-build` / `just r-cmd-check` trap-clean the
tarball on exit.

Three layered triggers all converge on this signal:
1. Maintainer's explicit `just vendor` / `miniextendr_vendor()`.
2. `bootstrap.R` (run by pkgbuild during `devtools::build()` / `rcmdcheck`).
3. End-user install of a tarball that arrived without the vendored
   dependencies already bundled.

A leaked tarball causes monorepo workspace-crate edits to be silently ignored
(no `[patch."git+url"]` in effect). Fix: `just clean-vendor-leak`. Detection:
`minirextendr_doctor()`.

## How it works

### A function call from R to Rust

When R calls a miniextendr function, the path is:

1. R evaluates `my_function(x, y)`.
2. The R wrapper (from `R/miniextendr-wrappers.R`) calls `.Call(C_my_function, x, y, .call = match.call())`.
3. The C wrapper — generated by `#[miniextendr]` in the macros crate — receives
   `(x_sexp: SEXP, y_sexp: SEXP)` on R's main thread inside
   `with_r_unwind_protect`.
4. `TryFromSexp` converts the incoming SEXPs to Rust types.
5. The user's Rust function runs.
6. `IntoR` converts the return value back to an SEXP.
7. If the function panics, `catch_unwind` intercepts and converts to an R error
   via the tagged-SEXP transport in `miniextendr-api/src/error_value.rs`.

Key safety properties: panics become R errors, `R_UnwindProtect` ensures Rust
destructors run even when R longjmps, and GC protection keeps SEXPs alive while
Rust holds references.

### Which files own which concern

- `#[miniextendr]` attribute processing:
  `miniextendr-macros/src/miniextendr_fn.rs` (standalone functions),
  `miniextendr-macros/src/miniextendr_impl.rs` (impl blocks).
- C wrapper code generation:
  `miniextendr-macros/src/c_wrapper_builder.rs` (`CWrapperContext`).
- R wrapper code generation:
  `miniextendr-macros/src/r_wrapper_builder.rs` (`DotCallBuilder`).
- Return type analysis (strict mode, `-> impl Trait`):
  `miniextendr-macros/src/return_type_analysis.rs`.
- Class system generators (one each for R6/S3/S4/S7/Env/Vctrs):
  `miniextendr-macros/src/r_class_formatter.rs` (shared `MethodContext`).
- Distributed_slice declarations + cdylib entry:
  `miniextendr-api/src/registry.rs`.
- Init sequence:
  `miniextendr-api/src/init.rs`.
- Cross-package ABI (`mx_wrap`/`mx_get`/`mx_query`/`mx_abi_register`):
  `miniextendr-api/src/mx_abi.rs`.
- Build configuration (Makevars template):
  `rpkg/src/Makevars.in`.
- Configure script source:
  `rpkg/configure.ac`.

## Decision trees

### Source mode vs tarball mode

Ask: is `rpkg/inst/vendor.tar.xz` present?

- Yes → tarball mode. `configure` writes
  `[source] ... replace-with = "vendored-sources"` in `.cargo/config.toml`.
  Cargo resolves all dependencies from the unpacked tarball. No network access.
  This is the path CRAN's offline build farm uses.
- No → source mode. In the miniextendr monorepo, `configure` writes
  `[patch."git+url"]` entries so the workspace siblings (miniextendr-api, etc.)
  are resolved from local paths instead of from git. Outside the monorepo
  (end-user package under development), the config is minimal and cargo
  resolves from git or crates.io.

If the tarball is unexpectedly present during local development (latch leak),
monorepo path overrides are missing and edits to workspace crates appear to
have no effect. Always run `just clean-vendor-leak` before iterating on
framework crates.

### Which crate do I change?

- Changing the Rust-side runtime behavior of a conversion, ExternalPtr,
  ALTREP, or error handling? That is `miniextendr-api`.
- Changing how `#[miniextendr]` parses attributes, generates C wrappers, or
  generates R wrapper fragments? That is `miniextendr-macros`.
- Changing the ordering or output format of `R/miniextendr-wrappers.R`? That
  may be `miniextendr-engine` (the `write_wrappers` entry point) or
  `miniextendr-macros` (the fragments emitted into `MX_R_WRAPPERS`).
- Adding a build-time source-level lint? That is `miniextendr-lint`.
- Changing how end-user packages are scaffolded? That is `minirextendr`.
- Changing the install-mode detection or configure flags? That is
  `rpkg/configure.ac` and `rpkg/src/Makevars.in`.

## Key files

- `miniextendr-api/src/registry.rs` — distributed_slice declarations and the
  cdylib entry point.
- `miniextendr-api/src/init.rs` — `package_init()` that consolidates all
  `R_init_*` steps.
- `miniextendr-api/src/mx_abi.rs` — Rust reimplementation of cross-package ABI.
- `miniextendr-macros/src/miniextendr_fn.rs` — `#[miniextendr]` on standalone
  functions.
- `miniextendr-macros/src/miniextendr_impl.rs` — `#[miniextendr]` on impl
  blocks.
- `miniextendr-macros/src/c_wrapper_builder.rs` — `CWrapperContext` for C
  wrapper codegen.
- `miniextendr-macros/src/r_wrapper_builder.rs` — `DotCallBuilder` for R
  wrapper codegen.
- `miniextendr-macros/src/r_class_formatter.rs` — shared `MethodContext` for
  all six class-system generators.
- `miniextendr-macros/src/return_type_analysis.rs` — return type analysis for
  strict mode.
- `miniextendr-engine/src/lib.rs` — `miniextendr_write_wrappers` cdylib entry.
- `rpkg/src/Makevars.in` — build pipeline template (the double-link lives here).
- `rpkg/configure.ac` — autoconf source for the install-mode latch logic.
- `rpkg/src/stub.c` — minimal C translation unit that pins all distributed_slice
  entries via `miniextendr_force_link`.
- `docs/ARCHITECTURE.md` — high-level overview with crate graph and call-flow
  diagram.
- `docs/CRAN_COMPATIBILITY.md` — vendoring and offline build details.

## Common pitfalls

- **`bash ./configure`, not `./configure`**: the script uses `#!/bin/sh` as its
  shebang, but on macOS `/bin/sh` is `dash` and `AC_CONFIG_COMMANDS` passthrough
  produces spurious errors. Always invoke as `bash ./configure` or via the
  `just configure` recipe.

- **Latch leak causes silent stale edits**: if `rpkg/inst/vendor.tar.xz` is
  present during local development, configure writes `[source]` vendored mode
  instead of `[patch."git+url"]` monorepo mode. Edits to `miniextendr-api` or
  `miniextendr-macros` appear to have no effect because cargo resolves those
  crates from the tarball, not the working tree. Run `just clean-vendor-leak`
  and `just configure` to repair.

- **`core.hooksPath` does not propagate to worktrees**: git worktrees inherit
  `.git/hooks/` from the common git dir, not the `core.hooksPath` setting from
  the main worktree's config. After creating a worktree, always run
  `git config core.hooksPath .githooks` in the worktree before making commits.

- **Never edit generated files directly**: `rpkg/src/Makevars`,
  `rpkg/src/rust/.cargo/config.toml`, and `rpkg/src/miniextendr-win.def` are
  all generated from `.in` templates. Edit the `.in` template and re-run
  `just configure` (or `autoconf && bash ./configure` if the script itself
  changed).

- **`R CMD build --debug` flag is invalid**: R silently ignores it with a
  warning. The `r-cmd-build` justfile recipe passes it; this is a pre-existing
  harmless quirk, not an intentional debug mode.

- **`configure.ac` must not mutate sources**: rewriting `Cargo.toml`,
  `Cargo.lock`, or `.rs` files during `./configure` dirties the VCS working
  tree. Vendoring belongs in `just vendor`, not in configure.

- **`--all-features` fails on this workspace**: the `default-r6` and
  `default-s7` features are mutually exclusive. CI's `clippy_all` job uses a
  curated feature list maintained in `.github/workflows/ci.yml`. Read it from
  there before reproducing locally.

## Related skills

- `miniextendr-macros` — deep dive into `#[miniextendr]` attribute parsing and
  codegen.
- `miniextendr-build` — configure.ac, Makevars.in, and the vendor pipeline in
  detail.
- `miniextendr-ffi` — `#[r_ffi_checked]`, `_unchecked` variants, and the
  MXL300/MXL301 lint rules.
- `miniextendr-getting-started` — walkthrough for new users starting from an
  empty R package.
