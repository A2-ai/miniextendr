# miniextendr

A Rust-R interoperability framework for building R packages with Rust backends.

## Build Commands

```bash
just check          # Run cargo check
just rcmdinstall    # Build and install R package
cargo fmt           # Format Rust code
cargo clippy        # Run lints
```

## Reference Documentation

The `background/` folder (gitignored) contains reference documentation:

| File | Use For |
|------|---------|
| `R Internals.html` | R's internal structures, SEXP types, memory management |
| `Writing R Extensions.html` | R package development, .Call interface, ALTREP |
| `Autoconf.html` | configure.ac script syntax |
| `GNU make.html` | Makefile syntax |
| `r-source-tags-R-4-5-2/` | R 4.5.2 source with tags - lookup exact API behavior |

**Always check `background/` for R API details before guessing.**

## Architecture

- `miniextendr-api/` - Runtime library (FFI, ExternalPtr, ALTREP, worker thread)
- `miniextendr-macros/` - Proc macros (#[miniextendr], miniextendr_module!)
- `rpkg/` - Example R package demonstrating all features

## Key Concepts

- **Worker thread pattern**: Rust code runs on worker thread for proper panic handling
- **ExternalPtr**: Box-like owned pointer using R's EXTPTRSXP with type safety via R symbols
- **ALTREP**: Lazy/compact vectors via proc-macro method traits
- **R_UnwindProtect**: Ensures Rust destructors run on R errors
