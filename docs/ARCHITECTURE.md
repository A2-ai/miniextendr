# Architecture Overview

This document provides a high-level overview of miniextendr for evaluators, contributors, and users comparing approaches for Rust-R interop.

## Why miniextendr exists

miniextendr differs from extendr in several key design decisions:

- **Worker thread**: All Rust code runs on a dedicated worker thread, providing proper panic isolation from R's main thread. Panics in Rust don't crash R.
- **Configure-based builds**: Uses autoconf/configure rather than build scripts, integrating with R's standard package build system.
- **ALTREP first-class**: Proc-macro-driven ALTREP support for lazy/zero-copy vectors.
- **Vendored for CRAN**: All dependencies are vendored for offline CRAN builds.

## Crate architecture

```
miniextendr-macros        miniextendr-engine
(proc macros)             (code generation)
      │                         │
      ├─────────┬───────────────┘
      ▼         ▼
miniextendr-api
(runtime library: FFI, ExternalPtr, ALTREP, worker thread)
      │
      ▼
rpkg / user packages
(R package with Rust backend)
```

### miniextendr-api

The runtime library. Provides:

- **FFI types**: `SEXP`, `Rboolean`, protect/unprotect wrappers
- **Type conversions**: `IntoR`, `TryFromSexp`, `IntoRAs` traits
- **ExternalPtr**: Type-safe `EXTPTRSXP` wrappers with `TypedExternal` for cross-package dispatch
- **ALTREP**: Proc-macro method traits for lazy/compact vectors
- **Worker thread**: `worker_thread::run()` for panic-safe Rust execution
- **GC protection**: `OwnedProtect`, `ProtectScope` for RAII-based protect/unprotect

### miniextendr-macros

Proc macros that generate the glue code:

- `#[miniextendr]` on functions: generates C-callable wrapper + R wrapper code
- `#[miniextendr]` on impl blocks: generates method dispatch (env/R6/S3/S4/S7 class systems)
- `miniextendr_module!`: declares which items to register with R
- `#[derive(ExternalPtr)]`, `#[derive(DataFrameRow)]`, `#[derive(Vctrs)]`, etc.

### miniextendr-engine

Code generation engine used by the `document` binary. Reads proc-macro output and generates `miniextendr_wrappers.R` (the R-side wrapper functions).

### miniextendr-lint

Build-time static analysis. Checks consistency between `#[miniextendr]` attributes and `miniextendr_module!` declarations. Runs automatically during `cargo check` via `build.rs`.

## How a function call flows

When R calls a miniextendr function, the path is:

```
R: my_function(x, y)
  │
  ▼
C: C_my_function(x_sexp, y_sexp)        [generated entrypoint in entrypoint.c]
  │
  ▼
Rust: worker_thread::run(|| {            [spawns/reuses worker thread]
    let x = i32::try_from_sexp(x_sexp); [convert R -> Rust]
    let y = i32::try_from_sexp(y_sexp);
    let result = my_function(x, y);      [call user's Rust function]
    result.into_sexp()                   [convert Rust -> R]
  })
  │
  ▼
R: receives result SEXP
```

Key safety properties:
- Panics in Rust are caught on the worker thread and converted to R errors
- `R_UnwindProtect` ensures Rust destructors run even when R longjmps
- GC protection keeps SEXPs alive while Rust holds references

## Build system

### Template / configure flow

```
.in templates ──[autoconf]──> configure script ──[./configure]──> generated files

Cargo.toml.in ──────────────────────────────────> Cargo.toml
Makevars.in ────────────────────────────────────> Makevars
entrypoint.c.in ────────────────────────────────> entrypoint.c
document.rs.in ─────────────────────────────────> document.rs
```

### Vendor system

For CRAN compatibility, all dependencies must be vendored:

1. **Workspace crates** (miniextendr-api, miniextendr-macros, miniextendr-lint): Synced by `./configure` via rsync to `rpkg/src/vendor/`
2. **crates.io dependencies** (proc-macro2, syn, quote): Vendored by `cargo vendor` during configure

### Cross-package dispatch

ExternalPtr objects can be passed between R packages. The `TypedExternal` trait uses R symbols for type identification, enabling trait dispatch across package boundaries without shared Rust types.

```
producer.pkg:                 consumer.pkg:
  Counter { value: i32 }       uses CounterView (trait object)
  impl Counter trait            impl Counter trait for DoubleCounter
  exports as ExternalPtr        calls trait methods via vtable lookup
```

## Project layout

See the [crate README](../CLAUDE.md) for the complete directory structure and build commands.
