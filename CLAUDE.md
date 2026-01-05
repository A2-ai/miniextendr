# miniextendr

A Rust-R interoperability framework for building R packages with Rust backends.

## Principles

- **No backwards compatibility**: This is an unreleased project. Remove deprecated code, don't shim around it.
- **Simple over complex**: Avoid over-engineering. Only make changes directly requested or clearly necessary.
- **Trust the framework**: Don't add excessive error handling for scenarios that can't happen internally.
- **Edit `.in` templates, not generated files**: Files like `Cargo.toml`, `Makevars`, `configure` are generated from `.in` templates. Always edit the `.in` source file instead:
  - `rpkg/src/rust/Cargo.toml` → edit `rpkg/src/rust/Cargo.toml.in`
  - `rpkg/src/Makevars` → edit `rpkg/src/Makevars.in`
  - `rpkg/configure` → edit `rpkg/configure.ac` (then run `autoconf`)

## Project Structure

```
miniextendr/
├── miniextendr-api/      # Runtime library (FFI, ExternalPtr, ALTREP, worker thread)
├── miniextendr-macros/   # Proc macros (#[miniextendr], miniextendr_module!)
├── miniextendr-bench/    # Benchmarks (separate workspace member)
├── miniextendr-lint/     # Static analysis tool (copy of macros parser)
├── miniextendr-engine/   # Code generation engine
├── rpkg/                 # Example R package demonstrating all features
├── minirextendr/         # Helper R package for scaffolding new projects
├── tests/cross-package/  # Cross-package trait ABI tests
│   ├── producer.pkg/     # Exports types with TypedExternal
│   └── consumer.pkg/     # Imports and uses those types
└── background/           # Reference docs (gitignored, R Internals, etc.)
```

## Build Commands

### Quick Reference

```bash
# Rust development
just check              # Run cargo check on all crates
just test               # Run cargo tests
just clippy             # Run lints
just fmt                # Format Rust code

# R package development (rpkg)
just configure          # REQUIRED before any R CMD operations
just rcmdinstall        # Build and install rpkg
just devtools-test      # Run R tests
just devtools-document  # Regenerate R wrappers

# Full R CMD check workflow
just configure          # 1. Configure (generates vendor/, Makevars, etc.)
just r-cmd-build        # 2. Build tarball (R CMD build)
just r-cmd-check        # 3. Check (R CMD check)

# Cross-package tests
just cross-install      # Build + install producer.pkg and consumer.pkg
just cross-test         # Run cross-package tests

# minirextendr (scaffolding helper)
just minirextendr-install   # Install the helper package
just minirextendr-test      # Run tests
```

## Critical: Configure Before R CMD Operations

**ALWAYS run `./configure` (or `just configure`) before any R CMD operation.**

The configure script:
1. Syncs `miniextendr-api/`, `miniextendr-macros/`, `miniextendr-lint/`, `miniextendr-engine/` to `rpkg/src/vendor/`
2. Vendors crates.io dependencies (proc-macro2, quote, syn, unicode-ident)
3. Generates `Makevars` from `Makevars.in`

```bash
# WRONG - will fail or use stale code
R CMD build rpkg
R CMD check rpkg

# CORRECT
cd rpkg && ./configure   # or: just configure
R CMD build rpkg
R CMD check rpkg
```

### NOT_CRAN Environment Variable

Set `NOT_CRAN=true` for development mode:
```bash
cd rpkg && NOT_CRAN=true ./configure
NOT_CRAN=true R CMD INSTALL rpkg
```

**What NOT_CRAN does:**
- Preserves the `src/vendor/` directory during `R CMD build` (CRAN strips it)
- Enables symlinks for faster iteration (CRAN requires copies)
- Skips certain checks that only apply to CRAN submissions
- Should ALWAYS be set for local development/testing

## Development Workflow

### Making Changes to Rust Code

For changes to fully propagate (especially macro changes):

```bash
just configure          # 1. Sync crates to rpkg/src/vendor/
just rcmdinstall        # 2. Build and install (compiles Rust)
just devtools-document  # 3. Regenerate R wrappers
just rcmdinstall        # 4. Rebuild with updated R wrappers
```

**Why this order matters:**
- `just configure` syncs workspace crates to vendored copies
- First build compiles the new macros
- `devtools-document` runs the macros to regenerate `rpkg/R/miniextendr_wrappers.R`
- Second build incorporates the regenerated R code

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

## Key Concepts

- **Worker thread pattern**: Rust code runs on worker thread for proper panic handling
- **ExternalPtr**: Box-like owned pointer using R's EXTPTRSXP with type safety via R symbols
- **TypedExternal**: Trait for type-safe ExternalPtr identification across packages
- **ALTREP**: Lazy/compact vectors via proc-macro method traits
- **R_UnwindProtect**: Ensures Rust destructors run on R errors
- **GC Protection**: Use `OwnedProtect`/`ProtectScope` for RAII-based protect/unprotect

## Reference Documentation

The `background/` folder (gitignored) contains reference documentation.

### Official R Documentation

| File | Use For |
|------|---------|
| `R Internals.html` | R's internal structures, SEXP types, memory management |
| `Writing R Extensions.html` | R package development, .Call interface, ALTREP |
| `ALTREP_ Alternative Representations for R Objects.html` | ALTREP system deep dive |
| `Autoconf.html` | configure.ac script syntax |
| `GNU make.html` | Makefile syntax |

### R Source Code

| Directory | Use For |
|-----------|---------|
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
|---------|---------|
| `Rpkg-mutable-master/` | Mutable R objects via ALTREP |
| `Rpkg-simplemmap-master/` | Memory-mapped files via ALTREP |
| `vectorwindow-main/` | Vector windowing/views via ALTREP |

**Always check `background/` for R API details before guessing.**

## Sync Checks

### Vendor Sync

After modifying workspace crates, ensure vendored copies are updated:

```bash
just vendor-sync-check  # Verify vendored crates match workspace
just configure          # Refresh if drift detected
```

### Template Sync

Templates in `minirextendr/inst/templates/` should match rpkg sources:

```bash
just templates-check    # Verify templates haven't drifted
just templates-approve  # Accept current delta as approved
```

### Lint Sync

The lint crate parser should track the macros parser:

```bash
just lint-sync-check    # Check for significant drift
just lint-sync-diff     # Show differences
```

## Common Issues

### "configure: command not found"

Run autoconf first:
```bash
cd rpkg && autoconf && ./configure
```

### Stale R wrappers after macro changes

Run the full workflow:
```bash
just configure && just rcmdinstall && just devtools-document && just rcmdinstall
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
