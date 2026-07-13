# Architecture Overview

This document provides a high-level overview of miniextendr for evaluators, contributors, and users comparing approaches for Rust-R interop.

## Why miniextendr exists

miniextendr differs from extendr in several key design decisions:

- **Main thread with unwind protection**: By default, Rust code runs inline on R's main thread inside `R_UnwindProtect`, which catches both panics and R longjmps. The `worker-thread` feature supplies dedicated-worker infrastructure; `#[miniextendr(worker)]` or `worker-default` selects it.
- **Configure-based builds**: Uses autoconf/configure rather than build scripts, integrating with R's standard package build system.
- **ALTREP first-class**: Proc-macro-driven ALTREP support for lazy/zero-copy vectors.
- **Vendored for CRAN**: All dependencies are vendored for offline CRAN builds.

## Crate architecture

```text
miniextendr-macros ──re-exported by──▶ miniextendr-api ──▶ R packages
   (proc macros)                         (runtime + registry)

miniextendr-lint ──build-time source checks──────────────▶ R packages

miniextendr-engine
(standalone R embedding for Rust binaries and tests; independent of package codegen)
```

### miniextendr-api

The runtime library. Provides:

- **FFI types**: `SEXP`, `Rboolean`, protect/unprotect wrappers
- **Type conversions**: `IntoR`, `TryFromSexp`, `IntoRAs` traits
- **ExternalPtr**: Box-like `EXTPTRSXP` ownership for Rust values, with
  authoritative `Any::downcast` type checks
- **ALTREP**: Proc-macro method traits for lazy/compact vectors
- **Thread identification**: `is_r_main_thread()`, `Sendable<T>` for thread-safe dispatch
- **Worker thread** (`worker-thread` infrastructure, selected per export or by `worker-default`): `run_on_worker()` for dedicated thread dispatch
- **GC protection**: `OwnedProtect`, `ProtectScope` for RAII-based protect/unprotect
- **Package registry**: routine registration plus the host-only
  `miniextendr_write_wrappers` and `miniextendr_write_wasm_registry` writers

### miniextendr-macros

Proc macros that generate the glue code:

- `#[miniextendr]` on functions: generates C-callable wrapper + R wrapper code
- `#[miniextendr]` on impl blocks: generates method dispatch (env/R6/S3/S4/S7 class systems)
- Registration is automatic via linkme distributed slices
- `#[derive(ExternalPtr)]`, `#[derive(DataFrameRow)]`, `#[derive(Vctrs)]`, etc.

### miniextendr-engine

Standalone R embedding engine for Rust binaries, integration tests, and
benchmarks. It finds and links `libR`, initializes an embedded R runtime, and is
not used by normal R package builds. Package wrapper generation lives in
`miniextendr-api::registry` and runs from the freshly linked package shared
library.

### miniextendr-lint

Build-time static analysis. Checks `#[miniextendr]` source-level attributes for consistency. Runs automatically during `cargo check` via `build.rs`.

## How a function call flows

When R calls a miniextendr function, the path is:

```text
R: my_function(x, y)
  │
  ▼
Rust: C_mypkg_my_function(x_sexp, y_sexp) [registered via linkme + miniextendr_init!]
  │
  ▼
Rust: with_r_unwind_protect(|| {         [main thread, unwind-protected]
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
- Panics in Rust are caught via `catch_unwind` and converted to R errors
- `R_UnwindProtect` ensures Rust destructors run even when R longjmps
- GC protection keeps SEXPs alive while Rust holds references

## Build system

### Template / configure flow

```text
.in templates ──[autoconf]──> configure script ──[./configure]──> generated files

Makevars.in ────────────────────────────────────> Makevars
configure.ac (cargo-config command) ────────────> .cargo/config.toml
```

Note: `entrypoint.c.in` and `mx_abi.c.in` have been eliminated. All entry
points are now generated in Rust via `miniextendr_init!`. A minimal `stub.c`
exists solely to satisfy R's build system requirement for at least one C file.

### Vendor system

For CRAN compatibility, all dependencies must be vendored:

1. **Workspace crates** (miniextendr-api, miniextendr-macros, miniextendr-lint) and **crates.io dependencies** (proc-macro2, syn, quote): Both are populated by `cargo revendor` into `rpkg/vendor/`. See [CRAN_COMPATIBILITY.md](CRAN_COMPATIBILITY.md) for details.
2. **Local crates** use flat paths (`vendor/miniextendr-api/`); **transitive registry crates** use versioned paths (`vendor/serde-1.0.210/`).

### Cross-package dispatch

Concrete `ExternalPtr<T>` access is checked authoritatively with
`Any::downcast`; its R symbols are display and diagnostic metadata. The
separate trait ABI makes cross-package dispatch possible by querying the
object's stable trait tag and calling through its registered C-compatible
vtable, without requiring the consumer to know the concrete Rust type.

```text
producer.pkg:                 consumer.pkg:
  Counter { value: i32 }       uses CounterView (trait object)
  impl Counter trait            impl Counter trait for DoubleCounter
  exports as ExternalPtr        calls trait methods via vtable lookup
```

## Project layout

See the [repository README](../README.md) for the complete directory structure
and build commands.
