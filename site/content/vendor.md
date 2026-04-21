+++
title = "CRAN & Vendoring"
weight = 9
description = "Offline builds, dependency vendoring, and CRAN release prep"
+++

CRAN requires packages to build with **no network access** during
`R CMD INSTALL`. If cargo prints `Downloading crates` at any point during
`R CMD check` on the built tarball, that is an immediate CRAN failure.
miniextendr prevents this by vendoring all Rust dependencies into
`inst/vendor.tar.xz`, a self-contained offline build artifact, before the
tarball is assembled.

`inst/vendor.tar.xz` is **not tracked in git**. It is regenerated
deterministically from `Cargo.lock` plus workspace sources. Locally, run
`just vendor` to create it. In CI, `just r-cmd-check` depends on `just vendor`
and runs it automatically before every check.

## CRAN release flow

```bash
just vendor            # 1. Create inst/vendor.tar.xz from Cargo.lock + workspace
just configure-cran    # 2. Configure in prepare-cran mode (NOT_CRAN=false)
just r-cmd-build       # 3. Build tarball
just r-cmd-check       # 4. Check the built tarball (runs just vendor again)
```

`just r-cmd-check` re-runs `just vendor` as a dependency, so the tarball
baked into the built package is always current. Never run `R CMD check`
directly on the source directory -- use `just r-cmd-check` or
`just devtools-check` which go through the correct sequence.

## Build contexts

The configure script detects which context it is running in and adapts
accordingly. There is no flag to set manually -- context is derived from
environment and artifacts on disk.

| Context | Trigger | Behavior |
|---|---|---|
| `dev-monorepo` | Default inside the miniextendr repo | `[patch]` paths to workspace crates; no tarball unpacking |
| `dev-detached` | No monorepo, no vendor artifacts | Git/network deps; requires network at build time |
| `vendored-install` | `inst/vendor.tar.xz` present, `NOT_CRAN` unset or false | Offline build from vendored sources |
| `prepare-cran` | `PREPARE_CRAN=true` (highest precedence) | CRAN release prep; overrides other detection |

CRAN itself runs in `vendored-install` context -- it receives the tarball with
`inst/vendor.tar.xz` inside and builds entirely offline. The `prepare-cran`
context is for generating that tarball correctly in the first place.

## How `just vendor` works

`just vendor` invokes `cargo-revendor` with the CRAN-prep flag set:

```bash
cargo revendor \
  --manifest-path rpkg/src/rust/Cargo.toml \
  --output rpkg/vendor \
  --source-root . \
  --strip-all \
  --freeze \
  --compress rpkg/inst/vendor.tar.xz \
  --blank-md \
  --source-marker \
  --force \
  -v
```

Key points:

- `--source-root .` pre-seeds `rpkg/vendor/<crate>/` from the monorepo before
  `cargo metadata` runs. This is required on a fresh clone: `rpkg/src/rust/Cargo.toml`
  ships with frozen `path = "../../vendor/..."` deps, and without pre-seeding
  the first `cargo metadata` call fails immediately (see [#280](https://github.com/A2-ai/miniextendr/issues/280)).
- `--freeze` rewrites the manifest so all deps resolve from `vendor/` only,
  then regenerates `Cargo.lock` offline.
- `--compress` packs `vendor/` into `inst/vendor.tar.xz` for inclusion in the
  R tarball.
- `--force` bypasses the cache. Workspace-crate source edits leave `Cargo.lock`
  unchanged, so the cache would otherwise skip re-vendoring and ship a stale
  tarball. `--force` ensures the vendor tree is always fresh.
- `--strip-all` removes `tests/`, `benches/`, `examples/`, and binary targets
  from every vendored crate to reduce tarball size.
- `--source-marker` writes a `.vendor-source` provenance file recording where
  the vendor tree came from.
- `--blank-md` blanks `.md` files before compression (license/readme text
  that CRAN does not need).

See [`cargo-revendor/README.md`](https://github.com/A2-ai/miniextendr/blob/main/cargo-revendor/README.md)
for the full flag reference.

## cargo-revendor

`cargo-revendor` is a standalone cargo subcommand in `cargo-revendor/`
(excluded from the main workspace so vendoring logic can evolve independently).
It replaces `cargo vendor` with CRAN-specific behavior:

- Resolves workspace crates via `cargo package` to expand
  `*.workspace = true` inheritance before vendoring.
- Strips test, bench, example, and binary targets on request.
- Freezes `Cargo.toml` so everything resolves from `vendor/` with no
  network or git access.
- Uses `--versioned-dirs` by default: `vendor/<name>-<version>/` for
  external crates, `vendor/<name>/` for workspace crates. Pass `--flat-dirs`
  to revert to flat names.
- Caches based on a FNV-1a hash of `Cargo.lock`, `Cargo.toml`, and local
  workspace source trees. The cache file lives at `vendor/.revendor-cache`.
- Supports `--sync <manifest>` to union disjoint workspaces
  (for example, including the benchmarks workspace in the same tarball).
- Supports `--verify` to assert that the existing `vendor/` and tarball
  agree with `Cargo.lock` without re-vendoring. Used by
  `just vendor-verify` in CI pre-release checks.
- Sets `COPYFILE_DISABLE=1` on macOS to suppress Apple extended-attribute
  metadata warnings on Linux/Windows GNU tar.
- `--strict-freeze`: fail if any external `git =` dep survives the freeze
  pass (requires `--freeze`).

Install with `just revendor-install` or
`cargo install --path cargo-revendor`.

## Stale-frozen-vendor recovery

After rebasing or merging main, the frozen `path = "../../vendor/..."` deps
in `rpkg/src/rust/Cargo.toml` can point at crates that no longer exist in
`vendor/`. The symptom is `cargo metadata` failing with "failed to read
.../vendor/miniextendr-api/Cargo.toml".

Recovery:

1. Reset the frozen path deps back to `"*"` in `rpkg/src/rust/Cargo.toml`.
2. Delete `rpkg/vendor/` and `rpkg/src/rust/Cargo.lock`.
3. Run `just configure` (dev-monorepo mode re-syncs vendor/ from workspace).

`just vendor` (`--force` + `--source-root`) also recovers from this state
automatically, because `--source-root` pre-seeds the vendor tree before
`cargo metadata` runs.

## Scaffolding new packages

`minirextendr` scaffolds new miniextendr projects with vendoring built in:

```r
library(minirextendr)
create_miniextendr_package("mypackage")
```

The generated `configure.ac` auto-detects the build context and handles
vendoring transparently. End users never run `just`; the standard
`R CMD INSTALL` / `R CMD check` flow handles everything via
`configure` and `tools/*.R`.

## What is changing

`cargo-revendor`'s vendoring model is actively evolving. Two design issues
are open:

- [**#290**](https://github.com/A2-ai/miniextendr/issues/290): split into
  `--external-only` / `--local-only` phase modes so CI can cache the
  expensive external layer (keyed on `Cargo.lock`) and run only the cheap
  local layer on workspace-crate changes.
- [**#291**](https://github.com/A2-ai/miniextendr/issues/291): v2
  from-scratch redesign around a content-addressable cache, no manifest
  mutation (`--freeze` deleted), and a `cargo revendor pack` subcommand
  for release-artifact concerns. Would eliminate the bootstrap-seed step,
  the stale-frozen-vendor recovery ritual, and the four build-context modes.

#290 and #291 propose different trade-offs. Neither is implemented yet.
`just vendor` and the flag surface documented here reflect the current v1
state. Check the issues for the latest design direction before writing
tooling that depends on specific cargo-revendor internals.
