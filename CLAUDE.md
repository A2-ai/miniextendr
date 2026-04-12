# miniextendr

A Rust-R interoperability framework for building R packages with Rust backends.

## Principles

- **No backwards compatibility**: This is an unreleased project. Remove deprecated code, don't shim around it.
- **Simple over complex**: Avoid over-engineering. Only make changes directly requested or clearly necessary.
- **Trust the framework**: Don't add excessive error handling for scenarios that can't happen internally.
- **No pre-existing warnings**: If you encounter a warning, lint issue, or test failure — fix it, even if it's pre-existing and unrelated to the current task. There is no such thing as a "known issue" that can be ignored. Every warning is a bug to be fixed. Always leave the codebase cleaner than you found it.
- **just is for maintainers, not end users**: `just`/`justfile` is a convenience tool for miniextendr authors/maintainers only. End-user R packages built with miniextendr must NOT require `just`. Any build logic that end users need (vendoring, configure, etc.) must work through `configure.ac`, `tools/*.R` scripts, or standard R package mechanisms. Mitigate any reliance on `just` in generated/scaffolded packages.
- **Edit `.in` templates, not generated files**: Many files in rpkg are generated from `.in` templates. Always edit the `.in` source file instead:
  - `rpkg/src/rust/.cargo/config.toml` → edit `rpkg/src/rust/cargo-config.toml.in`
  - `rpkg/src/Makevars` → edit `rpkg/src/Makevars.in`
  - `rpkg/src/miniextendr-win.def` → edit `rpkg/src/win.def.in`
  - `rpkg/configure` → edit `rpkg/configure.ac` (then run `autoconf`)
  - `rpkg/src/stub.c` — static file (no configure substitution), just a linker stub
- **configure.ac must not depend on minirextendr**: Template `configure.ac` files should not call `minirextendr::*` functions. Instead, put helper R scripts in `tools/` (included in templates) and invoke them via `Rscript tools/my-helper.R` from configure.ac.
- **`cargo package` for workspace resolution**: When vendoring workspace crates, use `cargo package` to produce resolved Cargo.toml files (workspace inheritance already expanded). Do not hard-code workspace dependency replacements — the vendoring strategy must work with any workspace-monorepo project.
- **configure.ac must not modify source files**: Never rewrite Cargo.toml, Cargo.lock, or .rs files during `./configure`. This dirties the VCS tree. Use `cargo revendor --freeze` at vendor time instead.
- **m4 in AC_CONFIG_COMMANDS**: `$1` becomes empty (use `$0` or avoid `sh -c`). `[` and `]` in sed/grep patterns must use `@<:@` and `@:>@`.
- **cargo-revendor**: Standalone workspace (excluded from miniextendr workspace). Build with `just revendor-build`, test with `just revendor-test`. `--freeze` rewrites Cargo.toml to resolve from vendor/ only.
- **Windows paths in TOML**: Use forward slashes. `canonicalize()` adds `\\?\` prefix on Windows — strip it with `strip_prefix(r"\\?\")` before writing to TOML/config files.
- **macOS tar xattrs**: Set `COPYFILE_DISABLE=1` when creating tarballs on macOS to prevent Apple xattr metadata that causes warnings on Linux/Windows GNU tar.
- **Pointer provenance**: When caching a `*mut T` for later reads AND writes, derive it from a mutable path (`&mut T`, `Box::into_raw`, `downcast_mut`, `ptr::from_mut`). Never derive a write-capable `cached_ptr` from `&T` / `downcast_ref` / `as_ref` — shared-reference provenance makes later writes UB under Stacked Borrows.

## Adding a New Conversion Type (e.g., `Box<[T]>`)

To add a new container type to the R ↔ Rust conversion system, modify these files:

1. **`miniextendr-api/src/from_r.rs`** — `TryFromSexp` impls (R → Rust):
   - Native types: `Box<[i32]>`, `Box<[f64]>`, `Box<[u8]>`, `Box<[RLogical]>`, `Box<[Rcomplex]>`
   - NA-aware: `Box<[Option<i32>]>`, `Box<[Option<f64>]>`
   - Bool: `Box<[bool]>`, `Box<[Option<bool>]>`
   - String: `Box<[String]>`, `Box<[Option<String>]>`
2. **`miniextendr-api/src/into_r.rs`** — `IntoR` impls (Rust → R):
   - Native blanket: `Box<[T: RNativeType]>` (delegates to `&[T]`)
   - Bool: `Box<[bool]>` (non-RNativeType, needs explicit impl)
   - String: `Box<[String]>`
3. **`miniextendr-api/src/coerce.rs`** — `Coerce<Box<[R]>> for Box<[T]>`
4. **Serde docs** — `src/serde.rs`, `src/serde/de.rs`, `src/serde/traits.rs` (update type tables)
5. **rpkg test fixtures** — `rpkg/src/rust/<type>_tests.rs` + `rpkg/tests/testthat/test-<type>.R`
6. **Vendor sync** — run `vendor_miniextendr(path = "rpkg", local_path = ".")` to update rpkg's vendor/

Note: `bool` is NOT `RNativeType` (R uses `i32` for logicals), so it needs separate impls.
Note: The proc-macro (`#[miniextendr]`) handles `Box<[T]>` generically via `TryFromSexp`/`IntoR` — no macro changes needed.

## Capturing Command Output

**Always redirect long-running R/Cargo command output to a log file**, then read the log. This ensures you see the full output (no truncation from `tail`) and can re-read sections as needed.

```bash
# Pattern: redirect to file, then read with Read tool
just devtools-document 2>&1 > /tmp/devtools-doc.log
just rcmdinstall 2>&1 > /tmp/rcmdinstall.log
just r-cmd-check 2>&1 > /tmp/rcmdcheck.log
just devtools-test 2>&1 > /tmp/devtools-test.log
just vendor 2>&1 > /tmp/vendor.log
just devtools-check 2>&1 > /tmp/devtools-check.log
```

After the command finishes, use the **Read tool** to read the log file. Start by reading the whole file (or the tail end for very long logs), and if you need more context, read earlier sections. **Do NOT use `tail` or `head`** — use the Read tool so you see the complete output and can go back for more.

## Sandbox Restrictions

**IMPORTANT**: The Claude Code sandbox blocks compilation. Commands that compile code (like `just devtools-document`, `R CMD INSTALL`, `cargo build`) will fail with "Could not find tools necessary to compile a package" or similar errors.

For any command that needs to compile, use `dangerouslyDisableSandbox: true`:

```bash
# These commands require sandbox disabled:
just devtools-document    # Compiles via devtools::document()
just rcmdinstall          # R CMD INSTALL compiles
just devtools-test        # May need to recompile
R CMD INSTALL rpkg        # Direct R installation
R CMD check rpkg          # Checks include compilation
```

## Project Structure

```sh
miniextendr/
├── miniextendr-api/      # Runtime library (FFI, ExternalPtr, ALTREP, worker thread)
├── miniextendr-macros/   # Proc macros (#[miniextendr], derives)
├── miniextendr-macros-core/ # Shared naming helpers (used by macros)
├── miniextendr-bench/    # Benchmarks (separate workspace member)
├── miniextendr-lint/     # Static analysis tool
├── miniextendr-engine/   # Code generation engine
├── cargo-revendor/      # Standalone cargo subcommand for vendoring (not in workspace)
├── rpkg/                 # Example R package demonstrating all features (named `miniextendr`)
├── minirextendr/         # Helper R package for scaffolding new projects
├── tests/cross-package/  # Cross-package trait ABI tests
│   ├── producer.pkg/     # Exports types with TypedExternal
│   └── consumer.pkg/     # Imports and uses those types
├── site/                 # Zola documentation website (deployed to GitHub Pages)
└── background/           # Reference docs (gitignored, R Internals, etc.)
```

## Build Commands

### Quick Reference

```bash
# Rust development
just check              # Run cargo check on all crates
just test               # Run cargo tests
just clippy             # Run clippy lints
just lint               # Run miniextendr-lint (source-side checks)
just fmt                # Format Rust code

# R package development (rpkg)
just configure          # REQUIRED before any R CMD operations (dev mode, no vendoring)
just vendor             # Vendor deps for CRAN release prep (creates inst/vendor.tar.xz)
just rcmdinstall        # Build and install `library(miniextendr)` package in `rpkg` directory
just devtools-test      # Run R tests
just devtools-document  # Run roxygen2 (NAMESPACE + man pages)

# Documentation site
just site-build         # Build Zola site (output in site/public/)
just site-serve         # Local preview at http://127.0.0.1:1111

# Full R CMD check workflow
just configure          # 1. Configure (generates Makevars, etc.)
just r-cmd-build        # 2. Build tarball (R CMD build)
just r-cmd-check        # 3. Check the built tarball (R CMD check)

# CRAN release prep (vendors deps into tarball)
just vendor             # Uses cargo-revendor: vendor, strip, freeze, compress
just configure-cran     # Configure with PREPARE_CRAN=true (unpacks vendor.tar.xz)
# IMPORTANT: Always check the built tarball, not the source directory.
# R CMD check on a source directory skips steps like Authors@R -> Author/Maintainer conversion.

# devtools::check (preserves output for debugging)
just devtools-check     # Runs devtools::check with output saved to rpkg-check-output/
# When checks fail, explore rpkg-check-output/miniextendr.Rcheck/ for logs:
#   - 00check.log: Main check log
#   - 00install.out: Installation/compilation output
#   - tests/: Test output files

# IMPORTANT: When running `just r-cmd-check`, save output to a file and read it:
#   just r-cmd-check 2>&1 | tee /tmp/rcmdcheck.log
#   # Then read /tmp/rcmdcheck.log — do NOT tail the output, as warnings
#   # and notes appear throughout and will be missed.

# Cross-package tests
just cross-install      # Build + install producer.pkg and consumer.pkg
just cross-test         # Run cross-package tests

# minirextendr (scaffolding helper)
just minirextendr-install   # Install the helper package
just minirextendr-test      # Run tests
```

## Critical: Configure Before R CMD Operations

**ALWAYS run `./configure` (or `just configure`) before any R CMD operation.**

**IMPORTANT: Always invoke configure with `bash`** (e.g., `bash ./configure`, not `./configure`).
The autoconf-generated shebang is `#!/bin/sh`, which on some systems causes spurious
"command not found" errors in AC_CONFIG_COMMANDS variable passthrough. The justfiles
already use `bash ./configure`; follow the same pattern when running manually.

The configure script (dev mode):

1. Generates `Makevars` from `Makevars.in` and other build config files
2. Cleans up stale vendor artifacts (`vendor/`, `inst/vendor.tar.xz`)
3. Does NOT vendor — cargo resolves deps via `[patch]` in `Cargo.toml`

For CRAN release prep, use `just vendor` to create the vendor tarball.

```bash
# WRONG - will fail or use stale code
R CMD build rpkg
R CMD check rpkg

# CORRECT
cd rpkg && bash ./configure   # or: just configure
R CMD build rpkg
R CMD check rpkg
```

### Build Context Model

The configure script resolves one of four build contexts:

| Context | When | Behavior |
|---|---|---|
| `dev-monorepo` | Monorepo detected (default for `just configure`) | Uses `[patch]` paths, no vendoring |
| `dev-detached` | No monorepo, no vendor artifacts | Uses git/network deps directly |
| `vendored-install` | `NOT_CRAN=false` or vendor artifacts present | Offline build from vendored sources |
| `prepare-cran` | `PREPARE_CRAN=true` | Explicit CRAN release prep mode |

**Environment variables:**

- `NOT_CRAN=true` — dev mode (legacy, still supported)
- `PREPARE_CRAN=true` — explicit CRAN release prep (highest precedence)
- Neither set — auto-detects from monorepo/vendor presence

```bash
cd rpkg && NOT_CRAN=true bash ./configure    # dev-monorepo (explicit)
cd rpkg && bash ./configure                  # dev-monorepo (auto-detected in monorepo)
cd rpkg && PREPARE_CRAN=true bash ./configure # prepare-cran
```

This builds and checks a package called `miniextendr`,  i.e. you load it with
`library(miniextendr)`, not `library(rpkg)`.

## Development Workflow

### Making Changes to Rust Code

For changes to fully propagate (especially macro changes):

```bash
just configure          # 1. Configure build (generates Makevars, etc.)
just rcmdinstall        # 2. Build and install (compiles Rust + auto-generates R wrappers)
just devtools-document  # 3. Run roxygen2 (regenerate NAMESPACE + man pages)
```

**Why this order matters:**

- `just configure` generates build config files
- Build compiles Rust, then auto-generates `rpkg/R/miniextendr-wrappers.R` via cdylib
- `devtools-document` runs roxygen2 on the generated wrappers to update NAMESPACE

**Always run `just devtools-document`** after any change that affects R wrapper output — this includes changes to roxygen generation in proc macros (`r_class_formatter.rs`, `r_wrapper_builder.rs`, `roxygen.rs`), R wrapper codegen (`r_wrappers.rs`, class system generators), or adding/removing `#[miniextendr]` functions/impls in rpkg. The generated files (`rpkg/R/miniextendr-wrappers.R`, `rpkg/NAMESPACE`, `rpkg/man/*.Rd`) must be committed in sync with the Rust changes that produced them.

### Testing Changes

```bash
just test               # Rust unit tests (fast, no R required)
just devtools-test      # R package tests (requires rpkg installed)
just cross-test         # Cross-package ABI tests
```

### Before Committing

```bash
just fmt                # Format all Rust code
just clippy             # Check for warnings
just check              # Verify compilation
```

## R Packages in This Repo

### rpkg (Example Package)

The main example R package demonstrating all miniextendr features.

```bash
just configure          # Configure before any operation
just rcmdinstall        # Install for development
just devtools-test      # Run tests
just r-cmd-check        # Full R CMD check
```

### minirextendr (Scaffolding Helper)

Pure R package (no Rust) that helps scaffold new miniextendr projects.

```bash
just minirextendr-install   # Install
just minirextendr-test      # Test
just minirextendr-check     # R CMD check
```

### Cross-Package Tests (tests/cross-package/)

Tests for TypedExternal trait dispatch across package boundaries.

- `producer.pkg`: Exports Rust types wrapped in ExternalPtr
- `consumer.pkg`: Imports and uses those types

```bash
just cross-install      # Build and install both
just cross-test         # Run cross-package tests
just cross-check        # R CMD check both packages
```

## Adding New Rust Functions

When adding new `#[miniextendr]` functions to rpkg:

### Requirements for R export

1. **Function must have `#[miniextendr]`** — this attribute handles automatic registration via linkme's `#[distributed_slice]`
2. **Function must be `pub`** — only `pub` functions get `@export` in R wrappers
3. **No module declaration needed** — functions self-register automatically; there is no `miniextendr_module!`

### Workflow for new functions

```bash
# 1. Add your #[miniextendr] function(s) to a .rs file
# 2. Make sure the file is reachable via `mod` declarations from lib.rs
# 3. Rebuild (R wrappers are auto-generated during build):
just configure
just rcmdinstall
just devtools-document   # Regenerates NAMESPACE via roxygen2

# If permission issues, use local library path:
R_LIBS=/tmp/claude/R_lib NOT_CRAN=true R CMD INSTALL rpkg
```

### Feature-gated modules

For modules that only exist when a feature is enabled (like `rayon`):

```rust
// In lib.rs - use #[cfg] on the mod declaration
#[cfg(feature = "my_feature")]
mod my_module;
```

The `#[cfg]` on the `mod` declaration is sufficient — functions inside the module
self-register via linkme when the feature is enabled.

### What happens during build (R CMD INSTALL)

1. Makevars builds a **cdylib** via `cargo rustc --crate-type cdylib`
2. Loads the cdylib via `dyn.load()` and calls `miniextendr_write_wrappers`
3. Auto-generates `R/miniextendr-wrappers.R` with all R wrapper functions
4. Builds the **staticlib** (linked into the final `.so`)

### What `just devtools-document` does

- Runs roxygen2 on all R files (including the auto-generated wrappers)
- Regenerates `rpkg/NAMESPACE` with exports, S3method registrations, etc.

### Verifying your changes

```bash
just lint                        # Run source-side checks
NOT_CRAN=true just devtools-test # Run R tests
```

## Key Concepts

- **Worker thread pattern**: Rust code runs on worker thread for proper panic handling
- **ExternalPtr**: Box-like owned pointer using R's EXTPTRSXP. Stores `Box<Box<dyn Any>>` — thin pointer (in `R_ExternalPtrAddr`) → fat pointer (heap, carries `Any` vtable). Type safety via `Any::downcast`, not R symbols. Non-generic finalizer (`release_any`). `cached_ptr` must have mutable provenance (`downcast_mut` / `ptr::from_mut`, never `downcast_ref`).
- **TypedExternal**: Trait providing R-visible type name (`TYPE_NAME_CSTR` for display tag, `TYPE_ID_CSTR` for error messages). No longer used for type safety — `Any::downcast` is authoritative.
- **ALTREP**: Lazy/compact vectors. Single-struct pattern — no wrapper struct. Two paths:
  - **Field-based derive**: `#[derive(AltrepInteger)]` with `#[altrep(len = "field", elt = "field", class = "Name")]` generates everything (AltrepLen, AltIntegerData, low-level traits, TypedExternal, RegisterAltrep, IntoR, linkme entry, Ref/Mut)
  - **Manual traits + registration**: `#[derive(Altrep)]` with `#[altrep(class = "Name")]` generates registration only; user implements `AltrepLen`, `Alt*Data`, and calls `impl_alt*_from_data!()` manually
  - **`AltrepExtract` trait**: abstracts data extraction from ALTREP SEXP. Blanket impl for `TypedExternal` (ExternalPtr). Override for custom storage.
  - **`#[miniextendr]` on 1-field structs is removed** — use derives instead
- **R_UnwindProtect**: Ensures Rust destructors run on R errors
- **GC Protection**: Use `OwnedProtect`/`ProtectScope` for RAII-based protect/unprotect
- **Dots (`...`)**: R's variadic args become `_dots: &Dots`. Use `name @ ...` for custom name. See `docs/DOTS_TYPED_LIST.md`
- **typed_list!**: Validate dots structure with `#[miniextendr(dots = typed_list!(...))]`. Creates `dots_typed` variable.
- **`impl Trait` support**: Return position only (`-> impl IntoR`). The C wrapper is a caller that only sees the opaque trait bound, so it needs `IntoR` to call `into_sexp()`. Argument position is NOT supported — Rust's type inference cannot resolve the concrete type from `TryFromSexp + Trait` across a `let` binding (E0283), even when only one type satisfies both bounds.
- **S4 helpers**: `slot()`/`slot<-()` live in the `methods` package — evaluate via `getNamespace("methods")`, not `R_BaseEnv`.

## FFI Thread Checking (`#[r_ffi_checked]`)

The `#[r_ffi_checked]` proc-macro attribute on `unsafe extern` blocks generates **both** checked and `*_unchecked` variants of every declared function and static. Checked wrappers route through `with_r_thread()` (debug thread assertion); `*_unchecked` variants bypass thread checking entirely.

### How it works

```rust
// In ffi.rs:
#[r_ffi_checked]
unsafe extern "C-unwind" {
    pub fn Rf_allocVector(sexptype: SEXPTYPE, length: R_xlen_t) -> SEXP;
}

// Generates:
// pub unsafe fn Rf_allocVector(...)       → debug thread-assert, then calls real symbol
// pub unsafe fn Rf_allocVector_unchecked(...) → calls real symbol directly
```

Use `_unchecked` variants when you are **certain** you are on the R main thread (e.g., inside ALTREP callbacks, `with_r_unwind_protect` closures, or `with_r_thread` blocks) and want to eliminate the debug-mode assertion overhead.

### `_unchecked` variants

All functions in `ffi.rs` under `#[r_ffi_checked]` have `*_unchecked` variants (e.g., `Rf_protect` → `Rf_protect_unchecked`). Statics (`R_NilValue`, `R_NaString`, etc.) pass through unchanged. Grep `ffi.rs` and `ffi/altrep.rs` for the complete list. `^nonapi^` variants require `#[cfg(feature = "nonapi")]`.

## Reference Documentation

The `background/` folder (gitignored) contains reference documentation.

### Official R Documentation

| File | Use For |
|<------|---------|
| `R Internals.html` | R's internal structures, SEXP types, memory management |
| `Writing R Extensions.html` | R package development, .Call interface, ALTREP |
| `ALTREP_ Alternative Representations for R Objects.html` | ALTREP system deep dive |
| `Autoconf.html` | configure.ac script syntax |
| `GNU make.html` | Makefile syntax |

### R Source Code

| Directory | Use For |
|<-----------|---------|
| `r-source-tags-R-4-5-2/` | R 4.5.2 source with tags - lookup exact API behavior |

Key paths in R source:

- `src/include/Rinternals.h` - SEXP types, macros
- `src/include/R_ext/Altrep.h` - ALTREP C API
- `src/main/altclasses.c` - ALTREP implementations
- `src/main/memory.c` - GC, protect stack

### Reference R Packages

| Package | Use For |
|---------|---------|
| `S7-main/` | Next-gen OOP system - class system patterns for #[miniextendr] |
| `R6-main/` | Reference classes - class generation patterns |
| `vctrs-main/` | Vector helpers - type coercion, recycling patterns |
| `roxygen2-main/` | R documentation system - tag parsing, R wrapper generation |
| `mirai-main/` | Async framework - parallel execution patterns, clean environment model |

### ALTREP Reference Implementations

| Package | Use For |
|<---------|---------|
| `Rpkg-mutable-master/` | Mutable R objects via ALTREP |
| `Rpkg-simplemmap-master/` | Memory-mapped files via ALTREP |
| `vectorwindow-main/` | Vector windowing/views via ALTREP |

**Always check `background/` for R API details before guessing.**

## Documentation Site

The `site/` directory contains a [Zola](https://www.getzola.org/) static site deployed to GitHub Pages at `https://cgmossa.github.io/miniextendr/`.

### Structure

```
site/
├── config.toml          # Zola config (base_url, title, search, anchor links)
├── templates/           # Tera templates (base, index, page with sidebar, section)
├── static/css/style.css # Tokyonight dark theme
├── static/js/search.js  # Elasticlunr search integration
└── content/             # Markdown pages with TOML frontmatter (+++ ... +++)
```

### Adding/editing content

Content pages use Zola's TOML frontmatter:

```markdown
+++
title = "Page Title"
weight = 5
description = "One-line summary for cards and meta tags"
+++

Markdown body here.
```

`weight` controls sort order on the homepage card grid and sidebar. Lower weights appear first.

### Local preview

```bash
just site-serve   # http://127.0.0.1:1111 with live reload
just site-build   # One-shot build to site/public/
```

### Deployment

GitHub Actions (`.github/workflows/pages.yml`) auto-deploys on push to `main` when `site/**` or `*/src/**` changes. The workflow:

1. Builds **rustdoc** (`cargo doc --no-deps --workspace`)
2. Builds the **Zola site**
3. Copies rustdoc into `site/public/rustdoc/`
4. Deploys the combined output to GitHub Pages

Rustdoc API reference is at `https://cgmossa.github.io/miniextendr/rustdoc/miniextendr_api/`.

Manual deploy via `workflow_dispatch` is also available.

### Content vs docs/

`site/content/` pages are user-facing documentation derived from `docs/`. They are curated summaries -- not 1:1 copies. When updating `docs/`, check if the corresponding `site/content/` page needs updating too.

## Sync Checks

### Vendor Sync

After `just vendor`, verify vendored copies match workspace sources:

```bash
just vendor-sync-check  # Verify vendored crates match workspace
just vendor             # Refresh if drift detected
```

### Template Sync

Templates in `minirextendr/inst/templates/` are **derived from rpkg** (the master source).

**Workflow for changes:**
1. First ensure changes work in `rpkg/` (the master)
2. Apply appropriate changes to templates in `minirextendr/inst/templates/`
3. Run `just templates-approve` to lock in the delta

**Key points:**
- **Source direction**: `rpkg/` → templates (not the other way around)
- **Approved delta**: Templates may have extra logic for standalone projects (e.g., checking if miniextendr-api exists before using path overrides, running cargo vendor for transitive deps)
- **Patch file**: `patches/templates.patch` records the approved differences between rpkg and templates

```bash
just templates-check    # Verify templates haven't drifted unexpectedly
just templates-approve  # Accept current delta as approved (after intentional changes)
```

## miniextendr-lint

The `miniextendr-lint` crate is a build-time static analysis tool that checks source-level correctness of `#[miniextendr]` usage.

### What it checks

- **MXL008**: Trait impl class system compatibility with inherent impl
- **MXL009**: Multiple impl blocks for a type without distinct labels
- **MXL010**: Duplicate labels on impl blocks
- **MXL106**: Non-`pub` function that would get `@export`
- **MXL203**: Redundant `internal` + `noexport` on the same item
- **MXL300**: Direct `Rf_error`/`Rf_errorcall` usage (should use `panic!()` instead)
- **MXL301**: `_unchecked` FFI calls outside of known-safe contexts

### Running the lint

```bash
just lint               # Run lint on rpkg
```

The lint runs automatically during `cargo build`/`cargo check` via `build.rs`. Output appears as cargo warnings. To disable temporarily:

```bash
MINIEXTENDR_LINT=0 cargo check --manifest-path=rpkg/src/rust/Cargo.toml
```

### Fixing lint errors

1. **"type T has N impl blocks but some are missing labels"** (MXL009)
   - Add `#[miniextendr(label = "...")]` with unique labels to each impl block

2. **"non-pub function will not be exported"** (MXL106)
   - Make the function `pub`, or add `#[miniextendr(noexport)]`

3. **"direct Rf_error usage"** (MXL300)
   - Replace `Rf_error()` with `panic!()` — the framework converts panics to R errors safely

## Common Issues

### R tests fail with "could not find function"

Functions exist in Rust but aren't callable from R. Check:

1. **Function has `#[miniextendr]`** — required for automatic registration
2. **Function is `pub`** — non-pub functions don't get `@export`
3. **Module is reachable** — the `.rs` file must be reachable via `mod` declarations from `lib.rs`
4. **NAMESPACE is stale** — run `just devtools-document` to regenerate

Quick fix:

```bash
just configure && just rcmdinstall && just devtools-document
# Or with local library path if permission issues:
R_LIBS=/tmp/claude/R_lib NOT_CRAN=true R CMD INSTALL rpkg
NOT_CRAN=true just devtools-document
```

### "configure: command not found"

Run autoconf first:

```bash
cd rpkg && autoconf && bash ./configure
```

### Stale R wrappers after macro changes

R wrappers are auto-generated during build, so just rebuild:

```bash
just configure && just rcmdinstall && just devtools-document
```

### Tests fail with "package not found"

Install rpkg first:

```bash
just rcmdinstall
```

### Cross-package tests fail

Rebuild both packages:

```bash
just cross-install
```

### Stale insta snapshots (`.snap.new` files)

A `.snap.new` file means an insta snapshot test produced different output. Diff it against the `.snap` file — if the change is expected (e.g., reflects a codegen fix), `mv` the `.snap.new` over the `.snap` to accept it. Run `just test` to confirm.

### R package installation permission errors

If you get "ERROR: no permission to install to directory", use a local library path:

```bash
R_LIBS=/tmp/claude/R_lib R CMD INSTALL rpkg
# or
R_LIBS=/tmp/claude/R_lib just rcmdinstall
```

The `/tmp/claude/` directory is writable in sandboxed environments.

Alternatively, use `devtools::install()` which handles library paths:

```bash
just devtools-install
```

### Debugging segfaults

Use `R -d lldb` to run R under the LLDB debugger:

```bash
R -d lldb -e 'library(miniextendr); lazy_int_seq(0L, -1L, 1L)'
# At the (lldb) prompt: run
# After crash: bt (backtrace), frame select N, p variable
```

For segfaults during `R CMD check` or `devtools::test()`:

```bash
R -d lldb -e 'testthat::test_file("rpkg/tests/testthat/test-altrep.R")'
```

## Code Style

### Module Structure

**Never use the `mod.rs` pattern** (`foo/mod.rs`) — always use the `foo.rs` alongside `foo/` directory pattern instead.

- Example: `builtins.rs` + `builtins/math.rs` + `builtins/strings.rs`
- If you find existing `mod.rs` files, refactor them to the `foo.rs` pattern when touching that code

### Section Comments

**Use `// region:` / `// endregion`** for logical sections within a file. These are IDE-foldable in VS Code and RustRover.

```rust
// region: Scalar implementations
...
// endregion
```

When touching files that use other section patterns (`// =====` banners, `// ──` box drawing, `// ---`), migrate them to `// region:` / `// endregion`.

### Type Conversions

**Prefer `From`/`TryFrom` over `as` casts** — use `TryFrom` and `From` trait conversions instead of `as`-casts. Propagate the error rather than silently truncating or wrapping. When you encounter `as` casts during development, flag them for replacement.

## Error Handling in Vectorized Operations

**Collect all errors, not just the first** — in operations that can fail at multiple points (e.g. vectorized TryFromSexp, column validation, bulk conversions), collect all errors and report them together rather than bailing on the first one. This gives users actionable diagnostics instead of fix-one-rerun loops.

## File Deletion Safety

- **Never use `rm` or any permanent deletion command** in automated/agent workflows.
- Use a safe mechanism that moves files to system trash (`trash`, `gio trash`, or platform equivalent).
- If no trash utility is available, **stop and ask** instead of deleting.
- Permanent deletion is irreversible — the trash provides a recovery path.

## Agent Worktrees

- Agents should run in **worktrees** (`isolation: "worktree"`) so they don't collide with each other or main.
- **Merge workflow**: rebase the worktree branch onto current main, then merge into main. The rebase must happen immediately before the merge — not when the agent finishes.
- **Sequential merging**: When multiple worktrees need merging, do them one at a time: rebase worktree-1 onto main, merge it, then rebase worktree-2 onto the now-updated main, merge it, and so on. Each rebase must see the previous merge's commits on main, otherwise the merge silently overwrites them.
- **Never copy entire files** from a worktree to main — rebase + merge is the correct workflow. Copying whole files overwrites unrelated changes made on main since the worktree branched.
- If the agent didn't commit, commit its work in the worktree first, then rebase + merge.
- Never delete a worktree until its changes have been verified as merged into main.

## Plans

- **Flat structure, no phases** — list what needs to be done in a flat, prioritized order. Don't organize plans into "Phase 1", "Phase 2", etc. — just say what's needed and in what order.

## Reviews

When things go wrong during development (test failures, runtime errors, CI breakage, unexpected behavior), write a short markdown file in `reviews/` describing:

1. **What was attempted** — the change or operation
2. **What went wrong** — the error, failure mode, or unexpected behavior
3. **Root cause** — why it happened
4. **Fix** — what resolved it (or what's still open)

Name files descriptively, e.g. `reviews/cdylib-vctrs-init-crash.md`, `reviews/filter-repo-pr-destruction.md`.

These accumulate institutional knowledge about non-obvious failure modes that aren't captured by tests or docs.

## Vendor Audit

When dependencies change (new crates added, `just vendor` run), audit `vendor/` for crates that could be useful in miniextendr. Write a `plans/` file for any vendored crate worth integrating (e.g. a crate that provides R-relevant functionality like better error types, serialization, or data structures).

## Using Codex for Reviews

OpenAI's Codex CLI can be used for non-interactive code reviews and plan generation.

### Invocation

Use `codex exec` for non-interactive mode (the bare `codex` command requires a TTY):

```bash
# Non-interactive execution (no TTY needed)
codex exec -m gpt-5.3-codex "your prompt here"

# Full-auto mode (no confirmation prompts)
codex exec -m gpt-5.3-codex --full-auto "your prompt here"

# Review mode
codex exec -m gpt-5.3-codex review "review these changes"
```

**Important**: The bare `codex` command (without `exec`) requires a terminal/TTY and will fail with "stdin is not a terminal" when called from non-interactive contexts like Claude Code's Bash tool. Always use `codex exec` instead.

### Common patterns

```bash
# Code review of recent changes
codex exec -m gpt-5.3-codex --full-auto "Review the changes in the last commit for bugs and design issues"

# Generate implementation plan
codex exec -m gpt-5.3-codex --full-auto "Read file X and produce a plan to fix issues Y and Z"
```
