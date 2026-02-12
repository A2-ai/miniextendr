# miniextendr

A Rust-R interoperability framework for building R packages with Rust backends.

## Principles

- **No backwards compatibility**: This is an unreleased project. Remove deprecated code, don't shim around it.
- **Simple over complex**: Avoid over-engineering. Only make changes directly requested or clearly necessary.
- **Trust the framework**: Don't add excessive error handling for scenarios that can't happen internally.
- **Edit `.in` templates, not generated files**: Many files in rpkg are generated from `.in` templates. Always edit the `.in` source file instead:
  - `rpkg/src/rust/.cargo/config.toml` → edit `rpkg/src/rust/cargo-config.toml.in`
  - `rpkg/src/rust/document.rs` → edit `rpkg/src/rust/document.rs.in`
  - `rpkg/src/Makevars` → edit `rpkg/src/Makevars.in`
  - `rpkg/src/entrypoint.c` → edit `rpkg/src/entrypoint.c.in`
  - `rpkg/src/mx_abi.c` → edit `rpkg/src/mx_abi.c.in`
  - `rpkg/src/miniextendr-win.def` → edit `rpkg/src/win.def.in`
  - `rpkg/configure` → edit `rpkg/configure.ac` (then run `autoconf`)

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
├── miniextendr-macros/   # Proc macros (#[miniextendr], miniextendr_module!)
├── miniextendr-macros-core/ # Shared parser types (used by macros + lint)
├── miniextendr-bench/    # Benchmarks (separate workspace member)
├── miniextendr-lint/     # Static analysis tool
├── miniextendr-engine/   # Code generation engine
├── rpkg/                 # Example R package demonstrating all features (named `miniextendr`)
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
just clippy             # Run clippy lints
just lint               # Run miniextendr-lint (checks macro/module consistency)
just fmt                # Format Rust code

# R package development (rpkg)
just configure          # REQUIRED before any R CMD operations (dev mode, no vendoring)
just vendor             # Vendor deps for CRAN release prep (creates inst/vendor.tar.xz)
just rcmdinstall        # Build and install `library(miniextendr)` package in `rpkg` directory
just devtools-test      # Run R tests
just devtools-document  # Regenerate R wrappers

# Full R CMD check workflow
just configure          # 1. Configure (generates Makevars, etc.)
just r-cmd-build        # 2. Build tarball (R CMD build)
just r-cmd-check        # 3. Check the built tarball (R CMD check)

# CRAN release prep (vendors deps into tarball)
just vendor             # Package workspace crates + vendor external deps
just configure-cran     # Configure with PREPARE_CRAN=true (unpacks vendor.tar.xz)
# IMPORTANT: Always check the built tarball, not the source directory.
# R CMD check on a source directory skips steps like Authors@R -> Author/Maintainer conversion.

# devtools::check (preserves output for debugging)
just devtools-check     # Runs devtools::check with output saved to rpkg-check-output/
# When checks fail, explore rpkg-check-output/miniextendr.Rcheck/ for logs:
#   - 00check.log: Main check log
#   - 00install.out: Installation/compilation output
#   - tests/: Test output files

# Cross-package tests
just cross-install      # Build + install producer.pkg and consumer.pkg
just cross-test         # Run cross-package tests

# minirextendr (scaffolding helper)
just minirextendr-install   # Install the helper package
just minirextendr-test      # Run tests
```

## Critical: Configure Before R CMD Operations

**ALWAYS run `./configure` (or `just configure`) before any R CMD operation.**

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
cd rpkg && ./configure   # or: just configure
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
cd rpkg && NOT_CRAN=true ./configure    # dev-monorepo (explicit)
cd rpkg && ./configure                  # dev-monorepo (auto-detected in monorepo)
cd rpkg && PREPARE_CRAN=true ./configure # prepare-cran
```

This builds and checks a package called `miniextendr`,  i.e. you load it with
`library(miniextendr)`, not `library(rpkg)`.

## Development Workflow

### Making Changes to Rust Code

For changes to fully propagate (especially macro changes):

```bash
just configure          # 1. Configure build (generates Makevars, etc.)
just rcmdinstall        # 2. Build and install (compiles Rust)
just devtools-document  # 3. Regenerate R wrappers
just rcmdinstall        # 4. Rebuild with updated R wrappers
```

**Why this order matters:**

- `just configure` generates build config files
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

## Adding New Rust Functions

When adding new `#[miniextendr]` functions to rpkg:

### Requirements for R export

1. **Function must be `pub`** - only `pub` functions get `@export` in R wrappers
2. **Function must be in `miniextendr_module!`** - list it in the module declaration
3. **Sub-modules must be `use`d** - if functions are in a sub-module, add `use module_name;` to the parent's `miniextendr_module!`

### Workflow for new functions

```bash
# 1. Add your #[miniextendr] function(s) to a .rs file
# 2. Add fn declarations to miniextendr_module! in that file
# 3. If new module, add `use module_name;` to lib.rs miniextendr_module!
# 4. Rebuild and regenerate R wrappers:
NOT_CRAN=true just configure
NOT_CRAN=true just rcmdinstall
NOT_CRAN=true just devtools-document   # Regenerates R/miniextendr_wrappers.R and NAMESPACE
NOT_CRAN=true just rcmdinstall         # Rebuild with new wrappers

# If permission issues, use local library path:
R_LIBS=/tmp/claude/R_lib NOT_CRAN=true R CMD INSTALL rpkg
```

### Feature-gated modules

For modules that only exist when a feature is enabled (like `rayon`):

```rust
// In lib.rs - use #[path] for conditional module inclusion
#[cfg(feature = "my_feature")]
#[path = "my_module.rs"]
mod my_module;

#[cfg(not(feature = "my_feature"))]
#[path = "my_module_disabled.rs"]
mod my_module;

miniextendr_module! {
    mod rpkg;
    use my_module;  // Works for both enabled and disabled
}
```

Create a stub module for when feature is disabled:

```rust
// my_module_disabled.rs
use miniextendr_api::miniextendr_module;

miniextendr_module! {
    mod my_module;
    // Empty - no functions when feature disabled
}
```

### What `just devtools-document` does

- Runs the `document` binary which executes proc macros to generate R code
- Regenerates `rpkg/R/miniextendr_wrappers.R` with R wrapper functions
- Runs roxygen2 to regenerate `rpkg/NAMESPACE` with exports

### Verifying your changes

```bash
just lint                        # Check #[miniextendr] ↔ miniextendr_module! consistency
NOT_CRAN=true just devtools-test # Run R tests
```

## Key Concepts

- **Worker thread pattern**: Rust code runs on worker thread for proper panic handling
- **ExternalPtr**: Box-like owned pointer using R's EXTPTRSXP with type safety via R symbols
- **TypedExternal**: Trait for type-safe ExternalPtr identification across packages
- **ALTREP**: Lazy/compact vectors via proc-macro method traits
- **R_UnwindProtect**: Ensures Rust destructors run on R errors
- **GC Protection**: Use `OwnedProtect`/`ProtectScope` for RAII-based protect/unprotect
- **Dots (`...`)**: R's variadic args become `_dots: &Dots`. Use `name @ ...` for custom name. See `docs/dots_typed_list.md`
- **typed_list!**: Validate dots structure with `#[miniextendr(dots = typed_list!(...))]`. Creates `dots_typed` variable.

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

The `miniextendr-lint` crate is a build-time static analysis tool that checks consistency between `#[miniextendr]` attributes and `miniextendr_module!` declarations.

### What it checks

- **Missing module entries**: `#[miniextendr]` items not listed in `miniextendr_module!`
- **Missing attributes**: Items in `miniextendr_module!` without `#[miniextendr]` attribute
- **Multiple impl blocks**: When a type has 2+ impl blocks, all must have distinct labels
- **Class system compatibility**: Trait impls must be compatible with inherent impl class systems

### What it does NOT check

- **Missing `use submodule;`** in parent module - if a sub-module has its own `miniextendr_module!` with all functions listed, the lint passes even if the parent forgets to `use` it. You'll only discover this at runtime when R can't find the functions.

### Running the lint

```bash
just lint               # Run lint on rpkg
```

The lint runs automatically during `cargo build`/`cargo check` via `build.rs`. Output appears as cargo warnings. To disable temporarily:

```bash
MINIEXTENDR_LINT=0 cargo check --manifest-path=rpkg/src/rust/Cargo.toml
```

### Fixing lint errors

1. **"#[miniextendr] fn X not listed in miniextendr_module!"**
   - Add `fn X;` to the appropriate `miniextendr_module!` block

2. **"fn X listed in miniextendr_module! but has no #[miniextendr] attribute"**
   - Add `#[miniextendr]` to the function definition, or remove from module

3. **"type T has N impl blocks but some are missing labels"**
   - Add `#[miniextendr(label = "...")]` with unique labels to each impl block

## Common Issues

### R tests fail with "could not find function"

Functions exist in Rust but aren't callable from R. Check:

1. **Function is `pub`** - non-pub functions don't get `@export`
2. **Function is in `miniextendr_module!`** - check the module declaration
3. **Sub-module is `use`d** - check parent module's `miniextendr_module!` has `use submodule;`
4. **NAMESPACE is stale** - run `just devtools-document` to regenerate

Quick fix:

```bash
NOT_CRAN=true just devtools-document && NOT_CRAN=true just rcmdinstall
# Or with local library path if permission issues:
R_LIBS=/tmp/claude/R_lib NOT_CRAN=true just devtools-document
R_LIBS=/tmp/claude/R_lib NOT_CRAN=true R CMD INSTALL rpkg
```

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
