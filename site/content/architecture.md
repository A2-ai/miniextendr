+++
title = "Architecture"
weight = 2
description = "Crate layout, call flow, and build system overview"
+++

## Why miniextendr exists

miniextendr differs from extendr in several key design decisions:

- **Main thread with unwind protection**: By default, Rust code runs inline on R's main thread inside `R_UnwindProtect`, which catches both panics and R longjmps. The `worker-thread` feature supplies dedicated-worker infrastructure; `#[miniextendr(worker)]` or `worker-default` selects it.
- **Configure-based builds**: Uses autoconf/configure rather than build scripts, integrating with R's standard package build system.
- **ALTREP first-class**: Proc-macro-driven ALTREP support for lazy/zero-copy vectors.
- **Vendored for CRAN**: All dependencies are vendored for offline CRAN builds.

## SEXP at the boundary

miniextendr keeps the R boundary pointer-oriented, but ownership depends on the
type. R-backed views can hold a `SEXP` directly and avoid a copy while the
caller or a protection handle keeps the object rooted. Ordinary conversions
such as `Vec<i32> -> integer()` explicitly copy Rust data into R-owned memory.

Rust-owned values use a different path. `ExternalPtr<T>` is a `#[repr(C)]`
handle containing the R `EXTPTRSXP`, a cached Rust pointer, and protection
state. The pointee is stored on the Rust heap as `Box<Box<dyn Any>>`; R's GC
decides when its finalizer runs and the Rust value is dropped. ALTREP can use an
external pointer similarly to keep a Rust backing store lazy and expose it as
an R vector.

Three rules follow:

1. **R-backed views need roots.** A bare view does not become safe merely
   because it contains a `SEXP`; use the call frame, `ProtectScope`, or an owned
   protection handle for the required lifetime.
2. **Rust-owned data remains Rust-owned.** `ExternalPtr<T>` avoids copying the
   pointee into R's heap, but it is not representation-equivalent to `SEXP`.
3. **Type and trait checks are distinct.** Concrete external-pointer access is
   checked with `Any::downcast`. Cross-package trait dispatch queries a stable
   trait tag and calls through a registered C-compatible vtable; R symbols on
   `TypedExternal` are display and diagnostic metadata.

Choose an R-backed view for existing R memory, `ExternalPtr<T>` for a Rust
value owned from R, and ALTREP when R should see vector semantics backed by
lazy or external storage.

## Performance considerations

Performance was a concrete design input, not a post-hoc measurement. The architectural choices above (main-thread default, SEXP-as-memory, ALTREP-first) were picked because they eliminate the two costs that dominate an R/Rust boundary: (1) cross-thread handoff of pointers that can't move off the main thread, and (2) data copies between the R heap and a parallel Rust heap.

The maintainer-only [`miniextendr-bench/`](https://github.com/A2-ai/miniextendr/tree/main/miniextendr-bench) crate exercises each subsystem (FFI dispatch, conversions, ALTREP materialization, panic/unwind overhead, class-system dispatch) under divan, and the results feed back into architectural review. The full methodology, recipe list, and the current reference baseline are documented in [Benchmarks](./manual/benchmarks/). Design changes that regress any headline number block the PR.

A concrete example: the `.Call()`-registered C entry point is the same mechanism [cpp11](https://cpp11.r-lib.org/) uses to expose C++ to R. miniextendr leans on the fact that R's DLL / `.Call()` dispatch is already a well-optimized, zero-marshalling path (R passes a `SEXP` array, the callee returns a `SEXP`); any interop framework that keeps SEXPs *as* SEXPs on the Rust side pays only the pointer-copy cost on the boundary. That is what miniextendr-bench's FFI-dispatch numbers are measuring against.

## Crate Architecture

```text
miniextendr-macros --re-exported by--> miniextendr-api --> R packages
   (proc macros)                        (runtime + registry)

miniextendr-lint --build-time source checks--------------> R packages

miniextendr-engine
(standalone R embedding for Rust binaries and tests; independent of package codegen)
```

Supporting crates outside the main dependency chain:

- `miniextendr-bench/` — divan-based performance suite (maintainer-only, separate workspace member).
- `miniextendr-cli/` — Rust-side workflow commands.
- `cargo-revendor/` — standalone `cargo revendor` subcommand for offline/CRAN vendoring (excluded from the main workspace).
- `minirextendr/` — R scaffolding helper for end users.

### miniextendr-api

The runtime library. Provides:

- **FFI types**: `SEXP`, `Rboolean`, protect/unprotect wrappers
- **Type conversions**: `IntoR`, `TryFromSexp`, `IntoRAs` traits
- **ExternalPtr**: Box-like `EXTPTRSXP` ownership for Rust values, with
  authoritative `Any::downcast` type checks
- **ALTREP**: Proc-macro method traits for lazy/compact vectors
- **Thread identification**: `is_r_main_thread()`, `Sendable<T>` for thread-safe dispatch
- **GC protection**: `OwnedProtect`, `ProtectScope` for RAII-based protect/unprotect
- **Package registry**: routine registration plus the host-side wrapper and
  wasm registry writers

### miniextendr-macros

Proc macros that generate the glue code:

- `#[miniextendr]` on functions: generates C-callable wrapper + R wrapper code
- `#[miniextendr]` on impl blocks: generates method dispatch (env/R6/S3/S4/S7 class systems)
- Registration is automatic via linkme distributed slices
- `#[derive(ExternalPtr)]`, `#[derive(DataFrameRow)]`, `#[derive(Vctrs)]`, etc.

### miniextendr-engine

Standalone R embedding for Rust binaries, tests, and benchmarks. It finds and
links `libR` and initializes an embedded R runtime. Normal R package builds do
not use it; their wrapper generation lives in `miniextendr-api::registry` and
runs from the freshly linked package shared library.

### miniextendr-lint

Build-time static analysis. Checks `#[miniextendr]` source-level attributes for consistency. Runs automatically during `cargo check` via `build.rs`.

## How a Function Call Flows

When R calls a miniextendr function:

```text
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

```text
.in templates --[autoconf]--> configure script --[./configure]--> generated files

Makevars.in -------------------------------------------------> Makevars
configure.ac (cargo-config command) -------------------------> .cargo/config.toml
```

All entry points are generated in Rust via `miniextendr_init!`. A minimal `stub.c` exists solely to satisfy R's build system requirement for at least one C file.

### Vendor system

For CRAN compatibility, all dependencies must be vendored:

1. **Workspace crates** (miniextendr-api, miniextendr-macros, miniextendr-lint)
2. **crates.io and git dependencies**

`cargo revendor` resolves and writes both groups into the package's `vendor/`
tree; release preparation compresses that tree into `inst/vendor.tar.xz`.

### Cross-package dispatch

Concrete `ExternalPtr<T>` conversion uses `Any::downcast`. Cross-package trait
dispatch is a separate ABI: the consumer queries the object's stable trait tag
and invokes methods through its registered C-compatible vtable. `TypedExternal`
symbols are used for display and diagnostics, not as the authoritative type
gate.

```text
producer.pkg:                 consumer.pkg:
  Counter { value: i32 }       uses CounterView (trait object)
  impl Counter trait            impl Counter trait for DoubleCounter
  exports as ExternalPtr        calls trait methods via vtable lookup
```

## Full reference

This page is a curated entry point. See the [user manual](/manual/architecture/) for the exhaustive treatment, edge cases, and every feature switch.
