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

## SEXP as the unit of memory

Rust types in miniextendr are, wherever possible, thin wrappers around `SEXP` (R's tagged pointer) rather than independent Rust-owned allocations. The goal is that the Rust-side representation of an R object *is* the R object: same pointer, same memory, same GC root. Concretely, types like `ExternalPtr<T>`, `Altrep<T>`, owned vector views, and the class-system wrappers (`R6<T>`, `S7<T>`, etc.) are all `#[repr(transparent)]` over `SEXP`, which lets us `transmute` between a typed handle and its raw `SEXP` without a conversion step and without any copy.

That alignment with the R API matters for three reasons:

1. **No parallel heap.** There is exactly one live copy of the data: the R heap. The Rust side doesn't duplicate it into `Vec<T>` and then write back, so the GC never sees "shadow" values that could go stale. Conversions that *do* copy (e.g. `Vec<i32>` to `integer()`) are explicit and localized to `TryFromSexp` / `IntoR`.
2. **FFI is free.** Passing a miniextendr wrapper across the `extern "C"` boundary is just passing a pointer. There's no boxing, no adapter struct, no `into_raw` dance. The transmute-equivalence means a `fn foo() -> MyExternalPtr` compiles to the same ABI as a `fn foo() -> SEXP`.
3. **Trait dispatch travels with the pointer.** Because the pointer carries its R class/altrep/extptr tag, type recovery is `Any::downcast` or a class-symbol check, not a lookup into a Rust-side registry that another package can't see. Cross-package dispatch works without a shared Rust type.

Implication for extenders: if you find yourself writing `struct MyThing { inner: Vec<Foo> }` and then converting back to `SEXP` on every call, prefer keeping the canonical storage on the R side and letting your Rust type be a typed view over it. ALTREP (see below) is the tool for keeping R semantics while materializing lazily on demand.

## Performance considerations

Performance was a concrete design input, not a post-hoc measurement. The architectural choices above (main-thread default, SEXP-as-memory, ALTREP-first) were picked because they eliminate the two costs that dominate an R/Rust boundary: (1) cross-thread handoff of pointers that can't move off the main thread, and (2) data copies between the R heap and a parallel Rust heap.

The maintainer-only [`miniextendr-bench/`](https://github.com/A2-ai/miniextendr/tree/main/miniextendr-bench) crate exercises each subsystem (FFI dispatch, conversions, ALTREP materialization, panic/unwind overhead, class-system dispatch) under divan, and the results feed back into architectural review. The full methodology, recipe list, and the current reference baseline are documented in [Benchmarks](./manual/benchmarks/). Design changes that regress any headline number block the PR.

A concrete example: the `.Call()`-registered C entry point is the same mechanism [cpp11](https://cpp11.r-lib.org/) uses to expose C++ to R. miniextendr leans on the fact that R's DLL / `.Call()` dispatch is already a well-optimized, zero-marshalling path (R passes a `SEXP` array, the callee returns a `SEXP`); any interop framework that keeps SEXPs *as* SEXPs on the Rust side pays only the pointer-copy cost on the boundary. That is what miniextendr-bench's FFI-dispatch numbers are measuring against.

## Crate Architecture

```text
miniextendr-macros        miniextendr-engine       miniextendr-lint
(proc macros)             (wrapper codegen)        (build-time checks)
      \                         |                        /
       \                        |                       /
        +-----------------------+----------------------+
                                v
                        miniextendr-api
                        (runtime library)
                                |
                                v
                example package / user packages
                (R package with Rust backend)
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
cargo-config.toml.in ----------------------------------------> .cargo/config.toml
```

All entry points are generated in Rust via `miniextendr_init!`. A minimal `stub.c` exists solely to satisfy R's build system requirement for at least one C file.

### Vendor system

For CRAN compatibility, all dependencies must be vendored:

1. **Workspace crates** (miniextendr-api, miniextendr-macros, miniextendr-lint): Synced to `vendor/`
2. **crates.io dependencies** (proc-macro2, syn, quote): Vendored by `cargo vendor`

### Cross-package dispatch

ExternalPtr objects can be passed between R packages. The `TypedExternal` trait uses R symbols for type identification, enabling trait dispatch across package boundaries without shared Rust types.

```text
producer.pkg:                 consumer.pkg:
  Counter { value: i32 }       uses CounterView (trait object)
  impl Counter trait            impl Counter trait for DoubleCounter
  exports as ExternalPtr        calls trait methods via vtable lookup
```
