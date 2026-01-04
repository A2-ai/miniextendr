# {{{crate_name}}}

A Rust-R package using miniextendr.

## Project Structure

```
{{{crate_name}}}/
├── {{{crate_name}}}/     # Main Rust crate
├── {{{rpkg_name}}}/      # R package with Rust backend
└── Cargo.toml            # Workspace root
```

## Build Commands

```bash
# Rust development
just check              # Run cargo check
just test               # Run cargo tests
just clippy             # Run lints
just fmt                # Format Rust code

# R package development
just configure          # REQUIRED before any R CMD operations
just rcmdinstall        # Build and install R package
just devtools-test      # Run R tests
just devtools-document  # Regenerate R wrappers
```

## Critical: Configure Before R CMD Operations

**ALWAYS run `./configure` (or `just configure`) before any R CMD operation.**

The configure script:
1. Syncs Rust crates to `{{{rpkg_name}}}/src/vendor/`
2. Vendors crates.io dependencies
3. Generates `Makevars` from `Makevars.in`

```bash
# WRONG - will fail or use stale code
R CMD build {{{rpkg_name}}}

# CORRECT
cd {{{rpkg_name}}} && ./configure
R CMD build {{{rpkg_name}}}
```

## Development Workflow

For changes to fully propagate (especially macro changes):

```bash
just configure          # 1. Sync crates to vendor/
just rcmdinstall        # 2. Build and install (compiles Rust)
just devtools-document  # 3. Regenerate R wrappers
just rcmdinstall        # 4. Rebuild with updated R code
```
