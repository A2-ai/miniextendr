+++
title = "Architecture"
weight = 2
description = "Crate layout, call flow, and build system overview"
+++

## Why miniextendr exists

miniextendr differs from extendr in several key design decisions:

- **Main thread with unwind protection**: By default, Rust code runs inline on R's main thread inside `R_UnwindProtect`, which catches both panics and R longjmps. An optional `worker-thread` feature enables a dedicated worker thread.
- **Configure-based builds**: Uses autoconf/configure rather than build scripts, integrating with R's standard package build system.
- **ALTREP first-class**: Proc-macro-driven ALTREP support for lazy/zero-copy vectors.
- **Vendored for CRAN**: All dependencies are vendored for offline CRAN builds.

## Crate Architecture

```
miniextendr-macros        miniextendr-engine
(proc macros)             (code generation)
      |                         |
      +----------+--------------+
      v          v
miniextendr-api           miniextendr-macros-core
(runtime library)         (shared parser types)
      |                         |
      v                    used by macros + lint
rpkg / user packages
(R package with Rust backend)
```

### miniextendr-api

The runtime library. Provides:

- **FFI types**: `SEXP`, `Rboolean`, protect/unprotect wrappers
- **Type conversions**: `IntoR`, `TryFromSexp`, `IntoRAs` traits
- **ExternalPtr**: Type-safe `EXTPTRSXP` wrappers with `TypedExternal` for cross-package dispatch
- **ALTREP**: Proc-macro method traits for lazy/compact vectors
- **Thread identification**: `is_r_main_thread()`, `Sendable<T>` for thread-safe dispatch
- **GC protection**: `OwnedProtect`, `ProtectScope` for RAII-based protect/unprotect

### miniextendr-macros

Proc macros that generate the glue code:

- `#[miniextendr]` on functions: generates C-callable wrapper + R wrapper code
- `#[miniextendr]` on impl blocks: generates method dispatch (env/R6/S3/S4/S7 class systems)
- Registration is automatic via linkme distributed slices
- `#[derive(ExternalPtr)]`, `#[derive(DataFrameRow)]`, `#[derive(Vctrs)]`, etc.

### miniextendr-engine

Code generation engine. Provides the `miniextendr_write_wrappers` function that reads linkme distributed slices and generates `miniextendr-wrappers.R`. Called via a temporary cdylib loaded into R.

### miniextendr-lint

Build-time static analysis. Checks `#[miniextendr]` source-level attributes for consistency. Runs automatically during `cargo check` via `build.rs`.

## How a Function Call Flows

When R calls a miniextendr function:

```
R: my_function(x, y)
  |
  v
Rust: C_my_function(x_sexp, y_sexp)     [registered via linkme]
  |
  v
Rust: with_r_unwind_protect(|| {         [main thread, unwind-protected]
    let x = i32::try_from_sexp(x_sexp); [convert R -> Rust]
    let y = i32::try_from_sexp(y_sexp);
    let result = my_function(x, y);      [call user's Rust function]
    result.into_sexp()                   [convert Rust -> R]
  })
  |
  v
R: receives result SEXP
```

Key safety properties:
- Panics in Rust are caught via `catch_unwind` and converted to R errors
- `R_UnwindProtect` ensures Rust destructors run even when R longjmps
- GC protection keeps SEXPs alive while Rust holds references

## Build System

### Template / configure flow

```
.in templates --[autoconf]--> configure script --[./configure]--> generated files

Makevars.in -------------------------------------------------> Makevars
cargo-config.toml.in ----------------------------------------> .cargo/config.toml
```

All entry points are generated in Rust via `miniextendr_init!`. A minimal `stub.c` exists solely to satisfy R's build system requirement for at least one C file.

### Vendor system

For CRAN compatibility, all dependencies must be vendored:

1. **Workspace crates** (miniextendr-api, miniextendr-macros, miniextendr-lint): Synced to `vendor/`
2. **crates.io dependencies** (proc-macro2, syn, quote): Vendored by `cargo vendor`

### Cross-package dispatch

ExternalPtr objects can be passed between R packages. The `TypedExternal` trait uses R symbols for type identification, enabling trait dispatch across package boundaries without shared Rust types.

```
producer.pkg:                 consumer.pkg:
  Counter { value: i32 }       uses CounterView (trait object)
  impl Counter trait            impl Counter trait for DoubleCounter
  exports as ExternalPtr        calls trait methods via vtable lookup
```
