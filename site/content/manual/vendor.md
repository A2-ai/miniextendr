+++
title = "Vendoring and CRAN Release Prep"
weight = 9
description = "How miniextendr packages its dependencies for CRAN offline builds."
+++

How miniextendr packages its dependencies for CRAN offline builds.

## Background

CRAN requires packages to build **offline** — no network access during
`R CMD INSTALL`. Rust packages depend on crates from crates.io plus
miniextendr's own workspace crates. The vendoring system pre-bundles
all of these into `inst/vendor.tar.xz` so the package is self-contained.

## Quick Reference

```bash
# CRAN release prep (from monorepo root):
just vendor            # 1. Create inst/vendor.tar.xz
just configure-cran    # 2. Configure in prepare-cran mode
just r-cmd-build       # 3. Build tarball
just r-cmd-check       # 4. Check the built tarball (--as-cran)
```

## PREPARE_CRAN

`PREPARE_CRAN` is an environment variable that triggers the `prepare-cran`
build context. It has **highest precedence** over all other build context
signals.

### What It Does

When `PREPARE_CRAN=true`:

1. **Sets `BUILD_CONTEXT=prepare-cran`** — overrides both `NOT_CRAN` and auto-detection
2. **Sets `NOT_CRAN=false`** — derived from the build context for backward compatibility
3. **Enables `--offline`** — cargo must resolve all deps from vendored sources
4. **Keeps `.cargo/config.toml`** — for vendored source directory replacement
5. **Unpacks `inst/vendor.tar.xz`** — if `vendor/` doesn't already exist
6. **Rewrites `Cargo.toml`** — git deps become path deps pointing to `vendor/`
7. **Strips `[patch]` section** — monorepo paths are not available
8. **Adds `[patch.crates-io]`** — for transitive miniextendr deps in vendor
9. **Regenerates `Cargo.lock`** — from vendored sources only

### How to Use

```bash
# Via justfile (recommended):
just configure-cran

# Manual (from rpkg/):
cd rpkg && PREPARE_CRAN=true bash ./configure

# With explicit vendor step:
just vendor && just configure-cran
```

### Where It's Referenced

| File | Purpose |
|------|---------|
| `rpkg/configure.ac` | Build context resolution (lines 33-38, 139-140) |
| `justfile` | `configure-cran` recipe |
| `rpkg/bootstrap.R` | Sets `PREPARE_CRAN=false` to prevent accidental inheritance during devtools workflows |
| `CLAUDE.md` | Build context table in documentation |

### Safety: bootstrap.R

`rpkg/bootstrap.R` explicitly sets `PREPARE_CRAN=false`:

```r
env <- c(NOT_CRAN = "true", PREPARE_CRAN = "false")
```

This prevents accidental CRAN-mode configuration during `devtools::install()`
or `devtools::document()`, which trigger bootstrap before configure. Without
this guard, an inherited `PREPARE_CRAN=true` from a parent shell could cause
devtools workflows to fail (they need network access for `[patch]` resolution).

## Build Contexts

The configure script resolves one of four build contexts. `PREPARE_CRAN` is
one input; the full truth table is:

```text
PREPARE_CRAN=true                              → prepare-cran
NOT_CRAN explicit=true  + monorepo present     → dev-monorepo
NOT_CRAN explicit=true  + monorepo absent      → dev-detached
NOT_CRAN explicit=false + any                  → vendored-install
auto-detect: monorepo present                  → dev-monorepo
auto-detect: vendor hint present               → vendored-install
auto-detect: neither                           → dev-detached
```

| Context | Cargo Config | `[patch]` | Vendor | Offline |
|---------|-------------|-----------|--------|---------|
| `dev-monorepo` | Removed | Kept (path deps) | Cleaned | No |
| `dev-detached` | Removed | Stripped | Cleaned | No |
| `vendored-install` | Kept | Rewritten | Unpacked/fetched | Yes |
| `prepare-cran` | Kept | Rewritten | Unpacked/fetched | Yes |

### dev-monorepo (default for developers)

Normal development in the monorepo. Cargo resolves workspace crates via
`[patch."https://..."]` paths in `Cargo.toml` that point to sibling
directories (`../../miniextendr-api`, etc.). No vendoring, no offline flag.

```bash
just configure   # or: cd rpkg && NOT_CRAN=true bash ./configure
```

### dev-detached

The example package directory (`rpkg/`) exists outside the monorepo (e.g., after scaffolding with
minirextendr). Cargo uses git deps directly from the Cargo.toml. The `[patch]`
section is stripped since monorepo paths are unavailable.

### vendored-install

Triggered when `NOT_CRAN` is explicitly false, or auto-detected when
`vendor/` or `inst/vendor.tar.xz` exists but no monorepo is present.
This is what CRAN and `R CMD INSTALL` from a tarball see.

### prepare-cran

Explicit CRAN release preparation. Functionally identical to `vendored-install`
but triggered by intent (`PREPARE_CRAN=true`) rather than detection. Use this
when preparing a submission to guarantee the correct build context regardless
of what else exists on disk.

## Vendor Pipeline

### Step 1: `just vendor`

Creates `rpkg/inst/vendor.tar.xz` containing all dependencies:

```text
just vendor
  │
  ├─ Rscript rpkg/tools/vendor-crates.R pack
  │
  ├─ cargo tree (discover reachable local path crates)
  │
  ├─ generate temporary cargo config
  │   ([patch.crates-io] for unpublished local crates)
  │
  ├─ cargo package --no-verify (local crates → .crate archives)
  │
  ├─ cargo vendor (crates.io deps → rpkg/vendor/)
  │
  ├─ Extract .crate archives on top of vendor/
  │   (workspace crates as vendored sources)
  │
  ├─ Strip checksums from Cargo.lock
  │
  ├─ Clean vendor/ (remove tests, benches, examples, dotfiles)
  │
  └─ tar -cJf rpkg/inst/vendor.tar.xz vendor/
```

**Key design decisions:**

- End-user vendoring goes through `rpkg/tools/vendor-crates.R`, so `configure`
  and the generated package can use the same entrypoint instead of relying on
  the miniextendr CLI.

- Local path/workspace crates are discovered from the resolved Cargo dependency
  graph, then packaged with a generated cargo config that patches unpublished
  sibling crates by path during the packaging step. This avoids hand-copying
  crate sources or hard-coding workspace metadata into the vendor pipeline.

- The resulting `.crate` archives are extracted into the vendor directory
  created by `cargo vendor`, so local crates look like any other vendored crate
  (with `.cargo-checksum.json` and versioned directory names).

- Checksum lines are stripped from `Cargo.lock` because vendored crates have
  `{"files":{}}` checksums (cargo vendor convention). Cargo regenerates
  checksums at build time.

- Tests, benchmarks, examples, and dotfiles are stripped from vendored crates
  to reduce tarball size.

### Step 2: `just configure-cran`

Runs `PREPARE_CRAN=true bash ./configure` which:

1. Detects `PREPARE_CRAN=true` → sets `BUILD_CONTEXT=prepare-cran`
2. Generates `Makevars`, `.cargo/config.toml` from templates
3. Unpacks `inst/vendor.tar.xz` → `vendor/` (if not already present)
4. Rewrites `Cargo.toml`:
   - Git deps (`miniextendr-api`, `miniextendr-lint`) → path deps to `vendor/`
   - Strips `[patch."https://..."]` section
   - Adds `[patch.crates-io]` for transitive deps (`miniextendr-macros`, etc.)
5. Strips git source replacement from `.cargo/config.toml`
6. Regenerates `Cargo.lock` offline from vendored sources
7. Extracts `CARGO_STATICLIB_NAME` via `cargo pkgid` and patches generated files

### Step 3: Build and Check

```bash
just r-cmd-build    # R CMD build rpkg → miniextendr_0.1.0.tar.gz
just r-cmd-check    # rcmdcheck with --as-cran --no-manual
```

**Important:** Always check the **built tarball**, not the source directory.
`R CMD check` on a source directory skips steps like `Authors@R` → `Author`/`Maintainer`
conversion.

## .Rbuildignore and vendor/

The `vendor/` directory at the package root is excluded by `.Rbuildignore`:

```text
^vendor$
```

This means `R CMD build` does NOT include `vendor/` in the tarball directly.
Instead, dependencies ship via `inst/vendor.tar.xz`. At install time,
configure unpacks the tarball to recreate `vendor/`.

**Why not ship vendor/ directly?**

- Thousands of `.rs` files in `vendor/` would trigger pkgbuild's rebuild
  detection on every `R CMD INSTALL` (it scans `src/` recursively)
- The compressed tarball is much smaller
- Cleaner package structure

## Cargo Config for Vendored Builds

The generated `.cargo/config.toml` (from `cargo-config.toml.in`) tells cargo
to resolve crates from the local `vendor/` directory:

```toml
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "../../vendor"
```

In dev contexts (`dev-monorepo`, `dev-detached`), this config is **removed**
so cargo uses its normal resolution with `[patch]` paths or git deps.

## Lockfile Compatibility

The configure script handles cargo lockfile version mismatches:

- Lockfile v4 requires cargo 1.78+
- If the installed cargo is older, configure regenerates the lockfile
- In release contexts, this requires vendor sources to be available first
  (so the lockfile-compat step unpacks `inst/vendor.tar.xz` if needed)

## Verify Vendor Sync

After `just vendor`, verify vendored workspace crates match their sources:

```bash
just vendor-sync-check   # Compares src/ dirs
just vendor-sync-diff    # Shows actual diffs
```

If drift is detected, re-run `just vendor` to refresh.

## Complete CRAN Release Workflow

```bash
# 1. Ensure all tests pass in dev mode
just configure
just rcmdinstall
just devtools-test

# 2. Vendor dependencies
just vendor

# 3. Configure for CRAN
just configure-cran

# 4. Build tarball
just r-cmd-build

# 5. Check tarball (CRAN mode)
just r-cmd-check

# 6. Fix any issues, repeat from step 1
```

## Known Limitations

### cargo tree text parsing

`vendor-crates.R` discovers local path-dependencies by parsing `cargo tree
--format {p}` output. `cargo metadata --format-version=1` would provide the
same information as stable JSON, but parsing it requires the `jsonlite` R
package (base R has no JSON parser). Since `vendor-crates.R` is copied into
scaffolded packages via `tools/`, it must remain zero-external-dependency.
If `cargo tree` output format changes in a future Cargo release, the
`parse_tree_packages()` function will need to be updated.

## See Also

- [R_BUILD_SYSTEM.md](R_BUILD_SYSTEM.md) — How R builds packages with compiled code
- [TEMPLATES.md](TEMPLATES.md) — Template system (configure.ac templates)
- [SMOKE_TEST.md](SMOKE_TEST.md) — Phase A4 covers CRAN-like tarball validation
