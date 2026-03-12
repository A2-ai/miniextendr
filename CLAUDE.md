# miniextendr

A Rust-R interoperability framework for building R packages with Rust backends.

## Principles

- **No backwards compatibility**: This is an unreleased project. Remove deprecated code, don't shim around it.
- **Simple over complex**: Avoid over-engineering. Only make changes directly requested or clearly necessary.
- **Trust the framework**: Don't add excessive error handling for scenarios that can't happen internally.
- **Edit `.in` templates, not generated files**: Many files in rpkg are generated from `.in` templates. Always edit the `.in` source file instead:
  - `rpkg/src/rust/.cargo/config.toml` → edit `rpkg/src/rust/cargo-config.toml.in`
  - `rpkg/src/Makevars` → edit `rpkg/src/Makevars.in`
  - `rpkg/src/miniextendr-win.def` → edit `rpkg/src/win.def.in`
  - `rpkg/configure` → edit `rpkg/configure.ac` (then run `autoconf`)
  - `rpkg/src/stub.c` — static file (no configure substitution), just a linker stub

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
just lint               # Run miniextendr-lint (source-side checks)
just fmt                # Format Rust code

# R package development (rpkg)
just configure          # REQUIRED before any R CMD operations (dev mode, no vendoring)
just vendor             # Vendor deps for CRAN release prep (creates inst/vendor.tar.xz)
just rcmdinstall        # Build and install `library(miniextendr)` package in `rpkg` directory
just devtools-test      # Run R tests
just devtools-document  # Run roxygen2 (NAMESPACE + man pages)

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
- **ExternalPtr**: Box-like owned pointer using R's EXTPTRSXP with type safety via R symbols
- **TypedExternal**: Trait for type-safe ExternalPtr identification across packages
- **ALTREP**: Lazy/compact vectors via proc-macro method traits
- **R_UnwindProtect**: Ensures Rust destructors run on R errors
- **GC Protection**: Use `OwnedProtect`/`ProtectScope` for RAII-based protect/unprotect
- **Dots (`...`)**: R's variadic args become `_dots: &Dots`. Use `name @ ...` for custom name. See `docs/DOTS_TYPED_LIST.md`
- **typed_list!**: Validate dots structure with `#[miniextendr(dots = typed_list!(...))]`. Creates `dots_typed` variable.

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

### Complete list of `_unchecked` variants

All **functions** below have a corresponding `*_unchecked` variant (e.g., `Rf_protect` → `Rf_protect_unchecked`). **Statics** (like `R_NilValue`, `R_NaString`) are passed through unchanged — they don't get `_unchecked` variants.

**Statics (no unchecked variant)** (`ffi.rs`):
`R_NilValue`, `R_NaString`, `R_BlankString`, `R_NamesSymbol`, `R_DimSymbol`, `R_DimNamesSymbol`, `R_ClassSymbol`, `R_RowNamesSymbol`, `R_LevelsSymbol`, `R_TspSymbol`, `R_GlobalEnv`, `R_BaseEnv`, `R_EmptyEnv`, `R_MissingArg`, `R_TrueValue`^nonapi^, `R_FalseValue`^nonapi^, `R_LogicalNAValue`^nonapi^

**String/character** (`ffi.rs`):
`Rf_mkChar`, `Rf_mkCharLen`, `Rf_mkCharLenCE`, `Rf_mkCharCE`, `Rf_xlength`, `Rf_translateCharUTF8`, `Rf_getCharCE`, `Rf_charIsASCII`, `Rf_charIsUTF8`, `Rf_charIsLatin1`, `R_nchar`, `Rf_type2char`, `R_CHAR`

**Unwind protect** (`ffi.rs`):
`R_MakeUnwindCont`, `R_ContinueUnwind`, `R_UnwindProtect`, `R_UnwindProtect_C_unwind`

**External pointers** (`ffi.rs`):
`R_MakeExternalPtr`, `R_ExternalPtrAddr`, `R_ExternalPtrTag`, `R_ExternalPtrProtected`, `R_ClearExternalPtr`, `R_SetExternalPtrAddr`, `R_SetExternalPtrTag`, `R_SetExternalPtrProtected`, `R_MakeExternalPtrFn`, `R_ExternalPtrAddrFn`, `R_RegisterFinalizer`, `R_RegisterCFinalizer`, `R_RegisterFinalizerEx`, `R_RegisterCFinalizerEx`

**C-callable interface** (`ffi.rs`):
`R_RegisterCCallable`, `R_GetCCallable`

**GC protection** (`ffi.rs`):
`R_PreserveObject`, `R_ReleaseObject`, `Rf_protect`, `Rf_unprotect`, `Rf_unprotect_ptr`, `R_ProtectWithIndex`, `R_Reprotect`

**Vector allocation** (`ffi.rs`):
`Rf_allocVector`, `Rf_allocMatrix`, `Rf_allocArray`, `Rf_alloc3DArray`, `Rf_allocList`, `Rf_allocLang`, `Rf_allocS4Object`, `Rf_allocSExp`

**Pairlist construction** (`ffi.rs`):
`Rf_cons`, `Rf_lcons`

**Attribute manipulation** (`ffi.rs`):
`Rf_setAttrib`, `Rf_getAttrib`, `Rf_namesgets`, `Rf_dimgets`, `Rf_classgets`, `Rf_dimnamesgets`, `Rf_GetRowNames`, `Rf_GetColNames`, `SET_ATTRIB`, `ATTRIB`

**Scalar constructors** (`ffi.rs`):
`Rf_ScalarComplex`, `Rf_ScalarInteger`, `Rf_ScalarLogical`, `Rf_ScalarRaw`, `Rf_ScalarReal`, `Rf_ScalarString`

**Data pointers** (`ffi.rs`):
`DATAPTR`^nonapi^, `DATAPTR_RO`, `DATAPTR_OR_NULL`, `LOGICAL`, `INTEGER`, `REAL`, `COMPLEX`, `RAW`

**Cons cell (pairlist) accessors** (`ffi.rs`):
`CAR`, `CDR`, `CAAR`, `CDAR`, `CADR`, `CDDR`, `CADDR`, `CADDDR`, `CAD4R`, `TAG`, `SET_TAG`, `SETCAR`, `SETCDR`, `SETCADR`, `SETCADDR`, `SETCADDDR`, `SETCAD4R`

**Nullable data pointers** (`ffi.rs`):
`LOGICAL_OR_NULL`, `INTEGER_OR_NULL`, `REAL_OR_NULL`, `COMPLEX_OR_NULL`, `RAW_OR_NULL`

**Element-wise accessors (ALTREP-aware)** (`ffi.rs`):
`INTEGER_ELT`, `REAL_ELT`, `LOGICAL_ELT`, `COMPLEX_ELT`, `RAW_ELT`, `VECTOR_ELT`, `STRING_ELT`, `SET_STRING_ELT`, `SET_LOGICAL_ELT`, `SET_INTEGER_ELT`, `SET_REAL_ELT`, `SET_COMPLEX_ELT`, `SET_RAW_ELT`, `SET_VECTOR_ELT`

**SEXP metadata** (`ffi.rs`):
`LENGTH`, `XLENGTH`, `TRUELENGTH`, `OBJECT`, `SET_OBJECT`, `LEVELS`, `SETLEVELS`, `TYPEOF`, `ALTREP`

**ALTREP support** (`ffi.rs`):
`ALTREP_CLASS`, `R_altrep_data1`, `R_altrep_data2`, `R_set_altrep_data1`, `R_set_altrep_data2`

**Symbol and duplication** (`ffi.rs`):
`Rf_install`, `Rf_installChar`, `PRINTNAME`, `Rf_duplicate`, `Rf_shallow_duplicate`, `Rf_copyMostAttrib`, `Rf_any_duplicated`, `R_compute_identical`, `Rf_PrintValue`

**Type coercion** (`ffi.rs`):
`Rf_asLogical`, `Rf_asInteger`, `Rf_asReal`, `Rf_asChar`, `Rf_coerceVector`

**Matrix utilities** (`ffi.rs`):
`Rf_nrows`, `Rf_ncols`

**Type checking predicates** (`ffi.rs`):
`Rf_inherits`, `Rf_isNull`, `Rf_isSymbol`, `Rf_isLogical`, `Rf_isReal`, `Rf_isComplex`, `Rf_isExpression`, `Rf_isEnvironment`, `Rf_isString`, `Rf_isArray`, `Rf_isMatrix`, `Rf_isList`, `Rf_isNewList`, `Rf_isPairList`, `Rf_isFunction`, `Rf_isPrimitive`, `Rf_isLanguage`, `Rf_isDataFrame`, `Rf_isFactor`, `Rf_isInteger`, `Rf_isObject`, `Rf_isOrdered`, `Rf_isUnordered`, `Rf_isUnsorted`

**Pairlist utilities** (`ffi.rs`):
`Rf_elt`, `Rf_lastElt`, `Rf_nthcdr`, `Rf_listAppend`, `Rf_PairToVectorList`, `Rf_VectorToPairList`

**Environment operations** (`ffi.rs`):
`Rf_findVar`, `Rf_findVarInFrame`, `Rf_findVarInFrame3`, `Rf_defineVar`, `Rf_setVar`, `Rf_findFun`, `R_NewEnv`, `R_existsVarInFrame`, `R_removeVarFromFrame`, `Rf_topenv`

**Evaluation** (`ffi.rs`):
`Rf_eval`, `Rf_applyClosure`, `R_tryEval`, `R_tryEvalSilent`, `R_forceAndCall`

**Matching and factors** (`ffi.rs`):
`Rf_match`, `Rf_asS4`, `Rf_S3Class`, `Rf_substitute`, `Rf_lengthgets`, `Rf_xlengthgets`

**Options** (`ffi.rs`):
`Rf_GetOption1`, `Rf_GetOptionDigits`, `Rf_GetOptionWidth`

**Weak references** (`ffi.rs`):
`R_MakeWeakRef`, `R_MakeWeakRefC`, `R_WeakRefKey`, `R_WeakRefValue`, `R_RunPendingFinalizers`

**User interrupt** (`ffi.rs`):
`R_CheckUserInterrupt`, `R_FlushConsole`

**Connections API** (`ffi.rs`, `#[cfg(feature = "connections")]`):
`R_new_custom_connection`, `R_ReadConnection`, `R_WriteConnection`, `R_GetConnection`

**Routine registration** (`ffi.rs`):
`R_registerRoutines`, `R_useDynamicSymbols`, `R_forceSymbols`

**Legacy C ABI** (`ffi.rs`, `legacy_c` module):
`R_RegisterCFinalizer_C`, `R_RegisterCFinalizerEx_C`, `R_MakeExternalPtrFn_C`, `R_ExternalPtrAddrFn_C`, `R_registerRoutines_C`

**Non-API encoding** (`ffi.rs`, `#[cfg(feature = "nonapi")]`):
`R_nativeEncoding` (function → `_unchecked`); `utf8locale`, `latin1locale`, `known_to_be_utf8` (statics, no unchecked)

**RNG** (`ffi.rs`):
`GetRNGstate`, `PutRNGstate`, `unif_rand`, `norm_rand`, `exp_rand`, `R_unif_index`, `R_sample_kind`

**Memory allocation** (`ffi.rs`):
`vmaxget`, `vmaxset`, `R_gc`, `R_gc_running`, `R_alloc`, `R_allocLD`, `S_alloc`, `S_realloc`, `R_malloc_gc`, `R_calloc_gc`, `R_realloc_gc`

**Sorting and utility** (`ffi.rs`):
`R_isort`, `R_rsort`, `R_csort`, `revsort`, `rsort_with_index`, `iPsort`, `rPsort`, `cPsort`, `R_qsort`, `R_qsort_I`, `R_qsort_int`, `R_qsort_int_I`, `R_ExpandFileName`, `R_atof`, `R_strtod`, `R_tmpnam`, `R_tmpnam2`, `R_free_tmpnam`, `R_CheckStack`, `R_CheckStack2`, `findInterval`, `findInterval2`, `R_max_col`, `StringFalse`, `StringTrue`, `isBlankString`

**ALTREP class construction and method registration** (`ffi/altrep.rs`):
`R_new_altrep`, `R_make_altstring_class`, `R_make_altinteger_class`, `R_make_altreal_class`, `R_make_altlogical_class`, `R_make_altraw_class`, `R_make_altcomplex_class`, `R_make_altlist_class`, `R_altrep_inherits`, `R_set_altrep_UnserializeEX_method`, `R_set_altrep_Unserialize_method`, `R_set_altrep_Serialized_state_method`, `R_set_altrep_DuplicateEX_method`, `R_set_altrep_Duplicate_method`, `R_set_altrep_Coerce_method`, `R_set_altrep_Inspect_method`, `R_set_altrep_Length_method`, `R_set_altvec_Dataptr_method`, `R_set_altvec_Dataptr_or_null_method`, `R_set_altvec_Extract_subset_method`, `R_set_altinteger_Elt_method`, `R_set_altinteger_Get_region_method`, `R_set_altinteger_Is_sorted_method`, `R_set_altinteger_No_NA_method`, `R_set_altinteger_Sum_method`, `R_set_altinteger_Min_method`, `R_set_altinteger_Max_method`, `R_set_altreal_Elt_method`, `R_set_altreal_Get_region_method`, `R_set_altreal_Is_sorted_method`, `R_set_altreal_No_NA_method`, `R_set_altreal_Sum_method`, `R_set_altreal_Min_method`, `R_set_altreal_Max_method`, `R_set_altlogical_Elt_method`, `R_set_altlogical_Get_region_method`, `R_set_altlogical_Is_sorted_method`, `R_set_altlogical_No_NA_method`, `R_set_altlogical_Sum_method`, `R_set_altraw_Elt_method`, `R_set_altraw_Get_region_method`, `R_set_altcomplex_Elt_method`, `R_set_altcomplex_Get_region_method`, `R_set_altstring_Elt_method`, `R_set_altstring_Set_elt_method`, `R_set_altstring_Is_sorted_method`, `R_set_altstring_No_NA_method`, `R_set_altlist_Elt_method`, `R_set_altlist_Set_elt_method`

^nonapi^ = requires `#[cfg(feature = "nonapi")]`

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

## Code Style

### Module Structure

**Never use the `mod.rs` pattern** (`foo/mod.rs`) — always use the `foo.rs` alongside `foo/` directory pattern instead.

- Example: `builtins.rs` + `builtins/math.rs` + `builtins/strings.rs`
- If you find existing `mod.rs` files, refactor them to the `foo.rs` pattern when touching that code

### Type Conversions

**Prefer `From`/`TryFrom` over `as` casts** — use `TryFrom` and `From` trait conversions instead of `as`-casts. Propagate the error rather than silently truncating or wrapping. When you encounter `as` casts during development, flag them for replacement.

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
