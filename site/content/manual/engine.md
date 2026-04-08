+++
title = "miniextendr-engine: why it exists (and why it changed)"
weight = 50
description = "This file documents the rationale for the recent refactor that moved R linking and embedded-R initialization into miniextendr-engine, and made the benchmark crate (miniextendr-bench) depend on it."
+++

This file documents the rationale for the recent refactor that moved R linking
and embedded-R initialization into `miniextendr-engine`, and made the benchmark
crate (`miniextendr-bench`) depend on it.

## Why centralize R init + linking?

The benchmark work initially embedded R directly inside `miniextendr-bench`
(custom `extern "C"` declarations + a per-crate `build.rs` linking to `libR`).
That was intentionally “quick”, but it caused two practical problems:

1. **Duplication / drift**
   - Every new benchmark/binary would need to re-implement: “how do we find
     `R_HOME`?” and “how do we link `-lR`?”.
   - Bugs would get fixed in one place but not the others.

2. **A hard-to-debug crash (SIGSEGV)**
   - The first version called `Rf_initEmbeddedR(...)` and then *also* called
     `setup_Rmainloop()` manually.
   - In R’s own source (`Rembedded.c`), `Rf_initEmbeddedR()` already calls
     `setup_Rmainloop()` as part of initialization. Calling it twice is unsafe
     and was the likely cause of the crash.

Putting this logic in `miniextendr-engine` makes embedding behavior consistent
across all “Rust-only” tools (benchmarks, experiments, helper binaries), and
keeps crash fixes in one place.

## What changed in `miniextendr-engine`

### 1) `build.rs` now links to `libR`

`miniextendr-engine/build.rs` resolves `R_HOME` (from the environment or by
running `R RHOME`) and emits:

- `cargo:rustc-link-search=native=<R_HOME>/lib`
- `cargo:rustc-link-lib=R`

This is the exact `cargo:` directive needed so dependents get `-lR`
automatically (the benchmark requested `println!(\"cargo:rustc-link-lib=R\");`).

### 2) Initialization uses `Rf_initialize_R()` + a single `setup_Rmainloop()`

Instead of calling `Rf_initEmbeddedR()` (which performs `setup_Rmainloop()`
internally), we now do:

1. `ensure_r_home_env()` (so embedding can find R’s resources)
2. `Rf_initialize_R(argc, argv)`
3. `setup_Rmainloop()` exactly once

This preserves control over when the mainloop setup happens, and avoids the
double-setup crash.

### 3) `R_HOME` is set when missing

Embedding fails with “R home directory is not defined” unless `R_HOME` is set.
`miniextendr-engine` now ensures `R_HOME` exists before initializing R by
running `R RHOME` and setting the process environment.

Note: in current Rust toolchains, `std::env::set_var` is `unsafe` because the
process environment is global mutable state; we only call it during startup
before any multi-threaded work.

## What changed in `miniextendr-bench`

### 1) The bench crate depends on `miniextendr-engine`

The benchmark no longer contains its own embedded-R init logic. It calls the
engine builder and keeps the engine alive for the process lifetime.

### 2) “Common vendor strategy”

The benchmark is configured to use the repository’s shared `vendor/` directory
(`miniextendr-bench/.cargo/config.toml` points to `../vendor`), matching the
existing “workspace-level vendoring” approach (see `justfile` target `vendor`).

## Bench intent (translate vs non-translate)

The benchmarks in `miniextendr-bench/benches/translate.rs` measure:

- `R_CHAR(charsxp)` pointer retrieval (fast path for UTF-8/ASCII)
- `Rf_translateCharUTF8(charsxp)` pointer retrieval (translation path)
- End-to-end “pointer → `CStr` → Rust `String`” for both strategies
- End-to-end `TryFromSexp<String>` (STRSXP → CHARSXP → translate → `String`)

We include two fixtures:

- UTF-8 CHARSXP (`"hello"`, `CE_UTF8`)
- Latin-1 CHARSXP (`"café"` with `CE_LATIN1`)

The point is to quantify the “extra cost” of always calling
`Rf_translateCharUTF8` vs taking an encoding-aware fast path.

## How to run the benchmark

From repo root:

```sh
cd miniextendr-bench
cargo bench --bench translate
```

If you want to refresh vendored dependencies:

```sh
just vendor
```
