+++
title = "CRAN & Vendoring"
weight = 9
description = "Offline builds, dependency vendoring, and CRAN release prep"
+++

CRAN requires packages to build with **no network access** during
`R CMD INSTALL`. miniextendr satisfies this by shipping all Rust
dependencies inside `inst/vendor.tar.xz`, a self-contained offline
build artifact bundled into the source tarball.

The full reference — install-mode decision tree, lockfile shape,
constraints, CI strategy — lives at
[CRAN compatibility](@/manual/cran-compatibility.md).

## Two install modes

There are exactly two install modes, selected automatically by configure:

| Mode | Triggered when | Cargo behavior |
|---|---|---|
| **Source install** | `inst/vendor.tar.xz` is **absent** | Cargo resolves deps normally. In monorepo dev, configure writes a `[patch."git+url"]` block pointing the workspace crates at sibling paths. Otherwise cargo follows the git URL. |
| **Tarball install** | `inst/vendor.tar.xz` is **present** | Configure unpacks the tarball, writes `.cargo/config.toml` with `[source]` redirected to `vendored-sources`, and cargo builds offline. |

That's the whole decision tree. There is no `NOT_CRAN`, no `PREPARE_CRAN`,
no `FORCE_VENDOR`, no auto-detected build-context enum — just the
file-existence test.

## CRAN release flow

```bash
just vendor             # 1. Regenerate Cargo.lock in tarball-shape, vendor
                        #    deps to rpkg/vendor/, compress to inst/vendor.tar.xz.
just r-cmd-build        # 2. R CMD build rpkg → miniextendr_X.Y.Z.tar.gz.
just r-cmd-check        # 3. R CMD check the built tarball (--as-cran).
```

Day-to-day commands (`just rcmdinstall`, `just devtools-install`,
`just devtools-test`, `just devtools-document`, `just devtools-load`)
do **not** depend on `just vendor`. They install via source mode, which
needs no vendor tarball at all. Run `just vendor` only when producing a
build artifact for CRAN.

`inst/vendor.tar.xz` is gitignored — regenerated deterministically from
`Cargo.lock` plus workspace sources. CI regenerates it before every
R CMD check; release tooling regenerates it at version-bump time.

## Tooling

- [`cargo-revendor`](https://github.com/A2-ai/miniextendr/blob/main/cargo-revendor/README.md)
  is a standalone cargo subcommand that powers `just vendor`. It expands
  `*.workspace = true` inheritance via `cargo package`, vendors external
  deps via `cargo vendor`, and clears `.cargo-checksum.json` for offline
  install. Install with `just revendor-install`.
- [`minirextendr`](@/manual/minirextendr.md) scaffolds new miniextendr
  projects with the same configure / Makevars / vendoring shape as
  `rpkg`.

## See also

- [CRAN compatibility](@/manual/cran-compatibility.md) — full reference
  (decision tree, lockfile shape, CI strategy, "symbols cleanup" list).
- [R build system](@/manual/r-build-system.md) — how R wires configure
  and Makevars together for compiled packages.
- [Environment variables](@/manual/environment-variables.md) — all env
  vars the build honors (and which legacy ones are gone).

## Full reference

This page is a curated entry point. The "See also" links above lead directly to the relevant manual pages for the exhaustive treatment, edge cases, and every feature switch.
