---
name: miniextendr-scaffolding
description: Use when creating a new R package with a Rust backend via minirextendr, upgrading an existing package's scaffold, diagnosing a broken setup with minirextendr_doctor(), maintaining or syncing templates between rpkg and minirextendr/inst/templates/, adding configure-time feature detection, or using use_release_workflow() for CRAN CI scaffolding.
---

# miniextendr Scaffolding (minirextendr)

minirextendr is the pure-R scaffolding helper that generates new R packages
with miniextendr Rust backends. It writes `configure.ac`, `Makevars.in`,
`Cargo.toml`, `lib.rs`, `stub.c`, and related boilerplate so users never
touch the build plumbing directly. It also maintains the template sync
pipeline, doctor checks, and release-workflow scaffolding.

## When to use this skill

- "How do I create a new R package with Rust using miniextendr?"
- "How do I upgrade an existing package scaffold to a newer miniextendr version?"
- "minirextendr_doctor() is reporting a problem — how do I fix it?"
- "I changed rpkg — how do I sync the templates?"
- "What is `just templates-approve` / `just templates-check`?"
- "How does configure-time feature detection work?"
- "I need to scaffold release CI for AlmaLinux 8 / macOS arm64."
- "What is the difference between `use_miniextendr()` and `create_miniextendr_package()`?"

## Key concepts

### minirextendr vs miniextendr

- **minirextendr** — pure-R scaffolding helper. Generates project structure.
  Installed once; users run `use_miniextendr()` to set up a package and never
  invoke minirextendr again during normal development.
- **miniextendr** — the runtime framework and macro system that the scaffolded
  Rust crate links against. End users add it as a Cargo dependency.

Cross-link: see `miniextendr-getting-started` for the end-user walkthrough.

### Template system

Templates live in `minirextendr/inst/templates/`. They are derived from `rpkg/`
(the canonical live example) with an approved delta recorded in
`patches/templates.patch`. The approved delta captures standalone-project
differences: templates may include extra logic for checking monorepo siblings
before applying path overrides, or running `cargo vendor` for transitive deps
in a non-monorepo context.

Templates use mustache-style substitution at scaffolding time (not autoconf
`@VAR@` substitution). The scaffolding functions call
`minirextendr::use_template()` / `render.R` to interpolate package name,
author, and other values into the emitted files.

The two template subtrees are:
- `minirextendr/inst/templates/rpkg/` — scaffolds a new package from scratch.
- `minirextendr/inst/templates/monorepo/` — scaffold variants for monorepo use.

### Source direction: rpkg → templates (never the reverse)

`rpkg/` is the master source. Always edit `rpkg/` first, port the change to
`minirextendr/inst/templates/`, then lock the delta with `just templates-approve`.
Never edit templates to change behavior that should live in `rpkg/`.

### `use_template()` and the silent-skip trap

`usethis::write_over()` silently skips overwriting a file in non-interactive
mode. `minirextendr::use_template()` (in `minirextendr/R/render.R`) deletes the
target file first, ensuring that `upgrade_miniextendr_package()` actually
overwrites existing scaffold files rather than leaving stale versions in place.

## How it works

### Scaffolding API

Key functions in `minirextendr/R/create.R`:

- `use_miniextendr()` — main entry point for adding Rust scaffolding to an
  existing R package or a newly created one. Detects an existing `DESCRIPTION`
  and adds files without overwriting existing R code.
- `create_miniextendr_package(path, ...)` — creates a new package skeleton and
  calls `use_miniextendr()` on it.

Both functions emit: `configure.ac`, `configure` (generated), `src/Makevars.in`,
`src/stub.c`, `src/rust/Cargo.toml`, `src/rust/src/lib.rs`, and `.gitignore`
updates.

After scaffolding, the user runs `bash ./configure && R CMD INSTALL .` to build.

### Upgrading an existing package

`upgrade_miniextendr_package()` in `minirextendr/R/upgrade.R` overwrites all
scaffold files with the latest templates. Because `use_template()` deletes the
target before writing, the upgrade is unconditional — all scaffold-managed
files are refreshed.

Edits made directly to scaffold-managed files (e.g., `configure.ac`,
`Makevars.in`) will be overwritten. Put project-specific logic in `tools/*.R`
and hook it from configure via `Rscript tools/foo.R`.

### Template sync workflow (contributor-facing)

When `rpkg/` changes in a way that should propagate to end-user packages:

1. Make the change in `rpkg/` first.
2. Port the change to `minirextendr/inst/templates/`.
3. Run `just templates-check` to see unexpected drift (non-zero exit = drift
   detected).
4. Run `just templates-approve` to regenerate `patches/templates.patch` with
   the new delta as the approved baseline.
5. Commit `rpkg/` changes, the template changes, and `patches/templates.patch`
   together.

`patches/templates.patch` records the approved delta between `rpkg/` and
`inst/templates/`. CI runs `just templates-check` to verify no unexpected
drift has accumulated.

### minirextendr_doctor()

`minirextendr/R/doctor.R` — detects two common broken states:

1. **Stale-latch**: `inst/vendor.tar.xz` is present in a development context
   where it should have been trap-cleaned. Causes configure to switch to
   tarball mode, silently ignoring monorepo path overrides. Fix:
   `just clean-vendor-leak`.

2. **Missing `.cargo/config.toml`**: configure was not run (or ran before the
   tarball was removed). Fix: `bash ./configure` (or `just configure`).

Run `minirextendr_doctor()` from R when a build behaves unexpectedly — it is
the first diagnostic step for any configure or cargo resolution issue.

### Configure-time feature detection

`minirextendr/R/feature-detect-configure.R` implements a system for probing
optional R package dependencies at configure time and enabling corresponding
cargo features.

- `add_feature_rule(feature, package, ...)` — registers a detection rule
  mapping a cargo feature to an R package. If the R package is installed at
  configure time, the feature is enabled.
- `use_configure_feature_detection()` — scaffolds the detection machinery into
  a package's `tools/detect-features.R`.

The detected feature list is passed to cargo via `CARGO_FEATURES` in `Makevars`.
Users can also override `CARGO_FEATURES` directly in their environment.

### Vendoring helpers

`minirextendr/R/vendor.R` (and `minirextendr/R/vendor-lib.R`):

- `miniextendr_vendor()` — R-side entry point equivalent to `just vendor`.
  Runs `cargo-revendor` to produce `inst/vendor.tar.xz`.
- `strip_toml_sections()` — strips `[[bench]]`, `[[test]]`, and
  `[dev-dependencies]` sections from vendored `Cargo.toml` files when the
  corresponding directories are absent in scaffolded packages. Without
  stripping, cargo fails to resolve the vendored tree because the referenced
  bench/test directories do not exist.

### use_release_workflow()

`minirextendr/R/use-release-workflow.R` scaffolds a known-good release CI
workflow template for R packages targeting CRAN on AlmaLinux 8 (the default
CRAN Linux build platform) and macOS arm64. Created to resolve #448 — the
AlmaLinux 8 and macOS arm64 combination has build environment quirks that
require pinning the macOS SDK and CRAN-provided library prefixes. See
`docs/RELEASE_WORKFLOW.md` for the full set of requirements.

### `just` is maintainer-only

Scaffolded packages must build via `configure.ac` / `tools/*.R` / standard R
mechanisms (`R CMD INSTALL`, `devtools::install()`). Never put `just` in
scaffolded package instructions or templates. If a template requires `just`, fix
the template.

## Decision trees

### Starting a new package

1. Install minirextendr: `install.packages("minirextendr")` or
   `remotes::install_github("A2-ai/minirextendr")`.
2. `usethis::create_package("mypkg")` — creates the R package skeleton.
3. `minirextendr::use_miniextendr()` — from the package root, adds Rust
   scaffolding.
4. Write Rust functions with `#[miniextendr]` in `src/rust/src/lib.rs`.
5. `bash ./configure && R CMD INSTALL .` — builds and installs.

### Upgrading an existing package

1. `minirextendr::upgrade_miniextendr_package()` — refreshes all
   scaffold-managed files from the latest templates.
2. Run `bash ./configure && R CMD INSTALL .` to rebuild.
3. If configure-time behavior changed (e.g., new feature detection), also run
   `minirextendr::use_configure_feature_detection()` to update `tools/`.

### Diagnosing a broken setup

1. `minirextendr_doctor()` from R — reports stale tarball or missing
   `.cargo/config.toml`.
2. If stale tarball: `just clean-vendor-leak` (monorepo) or manually
   `rm rpkg/inst/vendor.tar.xz` (standalone).
3. `bash ./configure` to regenerate `Makevars` and `.cargo/config.toml`.
4. If still broken: check `miniextendr-build` skill for detailed pipeline
   diagnostics.

### Template drift detected in CI

1. Identify which files drifted: `just templates-check` output lists them.
2. If the drift is intentional (rpkg changed, templates should follow):
   port the change to `inst/templates/` and run `just templates-approve`.
3. If the drift is unintentional (templates were edited directly):
   revert the template edits and make the change in `rpkg/` first.

## Key files

- `minirextendr/R/create.R` — `use_miniextendr()` and
  `create_miniextendr_package()` entry points.
- `minirextendr/R/upgrade.R` — `upgrade_miniextendr_package()`.
- `minirextendr/R/doctor.R` — `minirextendr_doctor()` health checks.
- `minirextendr/R/render.R` — `use_template()` (delete-then-write wrapper).
- `minirextendr/R/vendor.R` — `miniextendr_vendor()` and related helpers.
- `minirextendr/R/vendor-lib.R` — `strip_toml_sections()`.
- `minirextendr/R/feature-detect-configure.R` — `add_feature_rule()`,
  `use_configure_feature_detection()`.
- `minirextendr/R/use-release-workflow.R` — `use_release_workflow()`.
- `minirextendr/inst/templates/` — scaffolding template files.
- `patches/templates.patch` — approved rpkg → templates delta.
- `docs/RELEASE_WORKFLOW.md` — release CI details for AlmaLinux 8 / macOS arm64.

## Common pitfalls

- **Never edit templates to fix something that should be fixed in rpkg.**
  Templates are derived, not authoritative. The delta should be small and
  intentional; unreviewed drift accumulates and breaks `just templates-check`.

- **`upgrade_miniextendr_package()` overwrites scaffold files.** Put
  project-specific logic in `tools/*.R`, not in `configure.ac` or
  `Makevars.in` directly. Scaffold-managed files are always refreshed.

- **`bootstrap.R` is the auto-vendor trigger.** When devtools or pkgbuild
  invokes `R CMD build` on a source tree, `bootstrap.R` runs configure in a
  staging directory with no `.git` ancestor. If `cargo-revendor` is on PATH,
  auto-vendor fires. This is expected and correct behavior, not a bug.

- **Regression tests in `minirextendr/tests/testthat/` grep function source.**
  These tests use `deparse(body())` to check that template strings appear
  literally in function bodies. Inlining a helper just to pass such a test is
  implementation theater — fix the test or accept the indirection.

- **`configure.ac` must not call `minirextendr::*`.** Configure runs in a
  minimal environment at install time. Template-generated configure scripts
  follow the same rule; any configure-time R logic must go in `tools/*.R`.

## Related skills

- `miniextendr-getting-started` — end-user walkthrough from empty package to
  callable Rust function.
- `miniextendr-build` — configure.ac mechanics, install-mode latch, and
  Makevars pipeline in detail.
- `miniextendr-architecture` — the install-mode latch and the cdylib-to-staticlib
  double-link.
