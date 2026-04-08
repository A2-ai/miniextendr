+++
title = "CRAN & Vendoring"
weight = 9
description = "Offline builds, dependency vendoring, and CRAN release prep"
+++

CRAN requires packages to build **offline** -- no network access during `R CMD INSTALL`. miniextendr vendors all Rust dependencies into `inst/vendor.tar.xz` for self-contained builds.

## Quick Reference

```bash
# CRAN release prep:
just vendor            # 1. Create inst/vendor.tar.xz
just configure-cran    # 2. Configure in prepare-cran mode
just r-cmd-build       # 3. Build tarball
just r-cmd-check       # 4. Check the built tarball
```

## Build Contexts

The configure script resolves one of four contexts:

| Context | When | Behavior |
|---|---|---|
| `dev-monorepo` | Inside the miniextendr repo | Uses `[patch]` paths, no vendoring |
| `dev-detached` | Standalone, no vendor artifacts | Uses git/network deps |
| `vendored-install` | Vendor artifacts present | Offline build from vendored sources |
| `prepare-cran` | `PREPARE_CRAN=true` | Explicit CRAN release prep |

## How Vendoring Works

1. **`cargo-revendor`** vendors all crate dependencies (workspace + crates.io)
2. Dependencies are stripped (tests, benches, examples removed)
3. `Cargo.toml` files are frozen to resolve from `vendor/` only
4. Everything is compressed into `inst/vendor.tar.xz`
5. During `./configure`, the tarball is unpacked and cargo is configured for offline builds

## cargo-revendor

A standalone cargo subcommand (excluded from the miniextendr workspace) that replaces `cargo vendor` with CRAN-specific behavior:

- Strips test/bench directories and dev-dependencies
- Freezes Cargo.toml to use vendored sources only
- Uses `cargo package` to resolve workspace inheritance
- Sets `COPYFILE_DISABLE=1` on macOS to prevent xattr metadata

## Scaffolding New Packages

The `minirextendr` R package scaffolds new miniextendr projects with vendoring built in:

```r
library(minirextendr)
create_miniextendr_package("mypackage")
```

The generated `configure.ac` auto-detects the build context and handles vendoring transparently.
