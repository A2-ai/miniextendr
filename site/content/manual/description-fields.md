+++
title = "DESCRIPTION Fields for miniextendr Packages"
weight = 5
description = "When you inspect rpkg/DESCRIPTION or a scaffolded package such as tests/model_project/DESCRIPTION, some lines are ordinary R-package metadata and some are there specifically to support the miniextendr build flow."
+++

When you inspect `rpkg/DESCRIPTION` or a scaffolded package such as `tests/model_project/DESCRIPTION`, some lines are ordinary R-package metadata and some are there specifically to support the miniextendr build flow.

This page explains the less-obvious fields, especially the ones that tend to make people ask "why is this in DESCRIPTION at all?"

## Quick reference

| Field | Who reads it | Why it is there |
|-------|--------------|-----------------|
| `Roxygen: list(markdown = TRUE)` | `roxygen2` | Lets roxygen comments use Markdown syntax |
| `RoxygenNote: 7.3.x` | `roxygen2` and maintainers | Records which roxygen2 version last generated the docs |
| `Config/testthat/edition: 3` | `testthat` | Opts the package into the current testthat behavior |
| `SystemRequirements: Rust (>= 1.85)` | Humans, CI, package tooling | Declares that the package needs a Rust toolchain outside R |
| `Config/build/bootstrap: TRUE` | `pkgbuild` / `devtools` workflows | Runs `bootstrap.R` before build steps |
| `Config/build/never-clean: true` | `pkgbuild` / `devtools` workflows | Avoids forced preclean installs that would discard previous build outputs |
| `Config/build/extra-sources: src/rust/Cargo.lock` | `pkgbuild` | Treats additional build-relevant files as compilation inputs |

## Which fields are miniextendr-specific?

Only some of these are specific to the Rust-backed package flow:

- `SystemRequirements`
- `Config/build/bootstrap`
- `Config/build/never-clean`
- `Config/build/extra-sources`

The roxygen and testthat fields are ordinary R-package tooling settings. They show up in scaffolded projects because miniextendr creates a real R package, not a separate custom package format.

## `Roxygen: list(markdown = TRUE)`

This tells `roxygen2` to parse package documentation with Markdown enabled.

Without it, roxygen comments have to stick much more closely to raw Rd syntax. With it, docs can use common Markdown features such as:

- backticks for inline code
- fenced code blocks
- normal bullet lists
- Markdown links

This is not a miniextendr requirement. It is just the modern default for many R packages.

## `RoxygenNote`

`RoxygenNote` is bookkeeping written by `roxygen2`. It records the roxygen2 version that most recently generated the `man/*.Rd` files and `NAMESPACE`.

Practical meaning:

- you usually do not edit it by hand
- it changes when you regenerate docs with a newer roxygen2 version
- it helps explain why documentation output changed across machines or commits

It is useful metadata, but it is not part of the Rust build logic.

## `Config/testthat/edition: 3`

This opts the package into testthat edition 3.

That matters because testthat uses "editions" to bundle behavior changes that would otherwise be backward incompatible. In a new package, edition 3 is the normal choice.

This field is unrelated to Rust specifically. It lives in scaffolded packages because the templates assume a modern testthat setup.

## `SystemRequirements`

`SystemRequirements` is where an R package declares dependencies that live **outside** the R package library.

For miniextendr packages, that means the Rust toolchain:

- `rustc`
- `cargo`

This field is descriptive, not magic. It does **not** install Rust for the user. Instead, it communicates to:

- people trying to build the package
- CI setup
- maintainers checking package requirements

The exact wording can vary. For example:

- `SystemRequirements: Rust (>= 1.85)`
- `SystemRequirements: Cargo (Rust package manager), rustc >= 1.85.0`

The important point is that the package needs Rust tooling outside R itself.

## `Config/build/bootstrap: TRUE`

This is the most important custom build field in scaffolded miniextendr packages.

When `pkgbuild`-style workflows see:

```text
Config/build/bootstrap: TRUE
```

they run `bootstrap.R` in the package root before the later build steps.

For this repository, that is what makes the one-step dev workflow possible. `bootstrap.R` is responsible for kicking off the package's configure logic so that files such as:

- `configure`
- `src/Makevars`
- `src/rust/cargo-config.toml`

are in the right shape before the package build proceeds.

In practice, this is what lets commands like `devtools::document()` or related package-tooling flows trigger the Rust/configure pipeline instead of assuming the package is a plain R-only package.

If this field is missing, the rest of the miniextendr build chain becomes much less automatic.

## `Config/build/never-clean: true`

This tells `pkgbuild` not to add `--preclean` to `R CMD INSTALL`.

Why that matters:

- `--preclean` wipes previous build outputs before install
- Rust-backed packages can have expensive rebuild steps
- removing all prior objects can make the edit-build-test loop slower than necessary

For miniextendr packages, the intent is to preserve useful intermediate build state between installs unless there is a specific reason to do a full clean rebuild.

This is a performance and workflow choice, not a semantic requirement of Rust itself.

## `Config/build/extra-sources: src/rust/Cargo.lock`

`pkgbuild` already knows about standard compilation inputs such as `src/*.c`, `src/*.cpp`, and, for Rust-aware workflows, the usual Rust source files.

`Config/build/extra-sources` exists for files that also affect compilation decisions but are easy for generic tooling to miss.

In this repo the important extra input is:

```text
src/rust/Cargo.lock
```

That matters because changing `Cargo.lock` can change the compiled result even if:

- no `.rs` file changed
- `DESCRIPTION` did not change

So this field helps package tooling decide that the package should be rebuilt when dependency resolution changed.

## What should package authors usually edit?

Usually:

- keep `Config/build/bootstrap: TRUE`
- keep `Config/build/never-clean: true`
- keep `Config/build/extra-sources` unless your package needs additional tracked inputs
- update `SystemRequirements` if the required Rust toolchain version changes
- let `RoxygenNote` update automatically
- leave `Config/testthat/edition: 3` alone unless you have a specific reason to pin older testthat behavior

## Where these fields show up in this repo

The easiest concrete examples are:

- `rpkg/DESCRIPTION` for the example package
- `tests/model_project/DESCRIPTION` for the scaffold/tutorial package
- `minirextendr::use_miniextendr_description()` for the helper that writes these fields into a package DESCRIPTION

## See also

- `MINIREXTENDR.md` for the scaffolded package workflow
- `R_BUILD_SYSTEM.md` for the configure/build pipeline
- `PACKAGES.md` for the repo-wide map of crates, packages, and fixtures
