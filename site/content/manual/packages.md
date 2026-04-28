+++
title = "Package Map"
weight = 4
description = "This page explains every package-like component in the repository: the core Rust crates, the excluded support tools, the R packages, and the fixture packages used to test cross-package behavior."
+++

This page explains every package-like component in the repository: the core Rust crates, the excluded support tools, the R packages, and the fixture packages used to test cross-package behavior.

## Big picture

The repository is not a single crate or a single R package. It is a workspace plus a set of support packages:

- **Core Rust crates** implement the runtime, proc macros, embedding support, linting, benchmarking, and CLI.
- **Support tooling** handles vendoring and offline packaging.
- **R packages** expose the framework to R users and demonstrate the full build flow.
- **Fixture packages** verify cross-package ABI behavior and scaffolded-project expectations.

## Quick reference

| Package | Kind | Workspace status | Publish status | What it is for |
|---------|------|------------------|----------------|----------------|
| `miniextendr-api` | Rust crate | Workspace member | Published | Main runtime crate for Rust-to-R interop |
| `miniextendr-macros` | Rust crate | Workspace member | Published | Proc macros such as `#[miniextendr]` and derive macros |
| `miniextendr-engine` | Rust crate | Workspace member | Published | Standalone R embedding and initialization engine |
| `miniextendr-cli` | Rust crate / binary | Workspace member | Internal for now | CLI for scaffolding and workflow commands |
| `miniextendr-lint` | Rust crate | Workspace member | Internal only | Build-time source linter for `#[miniextendr]` usage |
| `miniextendr-bench` | Rust crate | Workspace member | Internal only | Benchmarks for runtime behavior and feature-gated paths |
| `cargo-revendor` | Rust crate / binary | Excluded from workspace | Internal only | Vendoring tool for offline and hermetic Rust dependency packaging |
| `rpkg/` | R package with nested Rust crate | Excluded from workspace | Repo example package | Full example package used to exercise the framework end to end |
| `minirextendr/` | R package | Excluded from workspace | R package | Scaffolding and maintenance helpers for miniextendr-based R packages |
| `tests/cross-package/producer.pkg` | R fixture package | Excluded from workspace | Test fixture | Produces shared trait-ABI objects for integration testing |
| `tests/cross-package/consumer.pkg` | R fixture package | Excluded from workspace | Test fixture | Consumes producer objects to verify cross-package dispatch |
| `tests/model_project/` | Scaffold fixture | Excluded from workspace | Test fixture | Snapshot-style sample project used in template and workflow tests |

## Core Rust crates

### `miniextendr-api`

This is the crate most downstream Rust code should depend on. It provides:

- R FFI bindings and `SEXP`-oriented helpers
- Rust-to-R and R-to-Rust conversions
- `ExternalPtr<T>`, trait ABI support, and class-system support
- ALTREP support and feature-gated integrations
- re-exports of the main proc macros so users usually only need one dependency

If you are writing Rust code for an R package, this is normally your starting point.

### `miniextendr-macros`

This crate contains the procedural macros:

- `#[miniextendr]`
- derive macros such as `ExternalPtr`, `DataFrameRow`, and `Vctrs`
- source-level defaults and macro-side feature behavior

Downstream users usually access these macros through `miniextendr-api`, but contributors working on macro expansion logic will edit this crate directly.

### `miniextendr-engine`

This crate is for **embedding R outside a normal R package build**. It handles:

- finding and configuring `R_HOME`
- standalone R initialization
- embedded evaluation used by tests, tooling, and benchmark scenarios

It is not the crate typical R package authors start with. If your code lives inside an R package, `miniextendr-api` is the relevant runtime layer. If you are building standalone tools or Rust-side integration tests that need an embedded R session, this is the right component.

### `miniextendr-cli`

This is the `miniextendr` command-line binary. It groups commands for:

- scaffolding
- workflow automation
- vendoring
- cargo helpers
- template and config-related operations

It overlaps somewhat with `minirextendr`, but from the Rust/CLI side rather than the R side.

### `miniextendr-lint`

This crate provides source-level diagnostics for framework usage, especially around `#[miniextendr]` attributes and related invariants. It is a contributor and build-time tool, not a downstream dependency most users interact with directly.

### `miniextendr-bench`

This is the benchmark harness for the repository. It measures conversion paths, ALTREP behavior, trait ABI overhead, and other runtime-sensitive parts of the framework. It is useful when changing internals, not when building a package on top of miniextendr.

## Support tooling outside the main workspace

### `cargo-revendor`

`cargo vendor` is not enough for this repository's R-package packaging needs because it does not fully cover workspace/path dependencies in the way the project needs for offline builds. `cargo-revendor` exists to fill that gap.

Use it when you need:

- hermetic vendoring
- offline/CRAN-oriented packaging flows
- path/workspace dependency handling beyond standard `cargo vendor`

It is intentionally kept out of the main workspace so its dependencies do not pollute the main `Cargo.lock`.

## R packages

### `rpkg/`

This directory is the **example package** used throughout the repository. The installed R package name is `miniextendr`, but in the repo the directory is named `rpkg/`.

It is important because it exercises the full real-world flow:

- `configure` generation
- `Makevars` generation
- Rust compilation during `R CMD INSTALL`
- generated R wrappers
- vendoring and CRAN-oriented packaging

When docs mention the "example package", this is usually what they mean.

### `minirextendr/`

This is the R helper package for users who want to create and maintain their own miniextendr-based R packages. It provides:

- scaffolding functions
- configure/build helpers
- vendoring helpers
- package workflow utilities
- output formats and helper integrations on the R side

If `rpkg/` shows how the framework works in a concrete package, `minirextendr/` is the package that helps users create packages like that themselves.

## Fixture packages and project fixtures

### `tests/cross-package/producer.pkg`

This fixture package exports types and trait ABI objects that other packages can consume. It exists to prove that cross-package dispatch works across package boundaries, not just within one package.

### `tests/cross-package/consumer.pkg`

This fixture package consumes producer-side objects and dispatches through the shared ABI. Together with `producer.pkg`, it validates one of the more subtle promises of the framework.

### `tests/model_project/`

This is a scaffold-like sample project used by tests. It is less about framework internals and more about validating that generated projects, templates, and workflow assumptions still behave as expected.

It also makes a good tutorial reference because it shows a complete package with:

- package metadata
- configure/bootstrap glue
- a nested Rust crate
- generated R wrappers
- generated manual pages

On the website, this is surfaced as the full-project tutorial at `/full-project-tutorial/`.

## How the pieces fit together

At a high level, the intended flow looks like this:

1. **Framework internals** live in the Rust crates such as `miniextendr-api`, `miniextendr-macros`, and `miniextendr-engine`.
2. **Authoring helpers** live in `minirextendr/` and `miniextendr-cli`.
3. **The reference package flow** is exercised in `rpkg/`.
4. **Cross-package guarantees** are tested in `tests/cross-package/`.
5. **Packaging and offline distribution** are supported by `cargo-revendor` and the vendoring/configure pipeline.

## Which package should you care about?

### If you are a downstream R package author

Start with:

- `minirextendr/` if you want scaffolding from R
- `miniextendr-cli` if you want CLI-driven setup/workflow
- `miniextendr-api` if you are writing the Rust code itself

### If you are contributing to the framework

You will usually touch:

- `miniextendr-api` for runtime behavior
- `miniextendr-macros` for macro behavior
- `miniextendr-lint` for source diagnostics
- `rpkg/` for end-to-end package validation

### If you are debugging packaging or release behavior

Look at:

- `rpkg/`
- `minirextendr/`
- `cargo-revendor`
- vendoring and configure-related docs such as `CRAN_COMPATIBILITY.md`, `R_BUILD_SYSTEM.md`, and `TEMPLATES.md`

## See also

- `README.md` for the short workspace layout summary
- `ARCHITECTURE.md` for the call flow and build-system view
- `DESCRIPTION_FIELDS.md` for the scaffolded `DESCRIPTION` entries in `rpkg/` and the model project
- `MINIEXTENDR.md` for the R helper package details
- `TEMPLATES.md` for how the example package and scaffolding templates relate
