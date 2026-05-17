+++
title = "Packages"
weight = 3
description = "A map of every major crate, R package, tool, and fixture package in the repository."
+++

This page is the short package map for the repository. Use it when you want to know what each crate or package is for before diving into subsystem-specific docs.

## The short version

The repository has four broad groups:

- **Core Rust crates** for the runtime, proc macros, embedding support, benchmarking, linting, and CLI commands.
- **Support tooling** for vendoring and offline packaging.
- **R packages** for the example package flow and user-facing scaffolding.
- **Fixture packages** for cross-package ABI tests and template validation.

## Package groups

| Package | Kind | Best way to think about it |
|---------|------|----------------------------|
| `miniextendr-api` | Runtime crate | The crate downstream Rust code usually depends on |
| `miniextendr-macros` | Proc-macro crate | The macro layer behind `#[miniextendr]` and derives |
| `miniextendr-engine` | Wrapper-codegen crate | Reads linkme registrations from a cdylib and emits `miniextendr-wrappers.R` |
| `miniextendr-cli` | CLI tool | Rust-side workflow and scaffolding commands |
| `miniextendr-lint` | Internal lint crate | Build-time checks for framework usage |
| `miniextendr-bench` | Benchmark crate | Performance and runtime experiments |
| `cargo-revendor` | Vendoring tool | Offline/hermetic dependency packaging support |
| `rpkg/` | Example R package | The repo's reference package that exercises the full build flow |
| `minirextendr/` | R helper package | Scaffolding and maintenance helpers for end users |
| `tests/cross-package/*` | Fixture packages | Producer/consumer packages for trait-ABI testing |
| `tests/model_project/` | Fixture project | Scaffold snapshot used in template/workflow tests and the full-project tutorial |

## When to start where

- If you are **building an R package with Rust**, start with [Getting Started](/getting-started/), then use `minirextendr` plus `miniextendr-api`.
- If you are **trying to understand the repo itself**, start with [Architecture](/architecture/) and then read the full [Package Map](/manual/packages/).
- If you are **debugging release or packaging behavior**, expect to spend time in `rpkg/`, `minirextendr/`, and the vendoring/build-system docs.

## Full package map

The detailed version, including workspace membership, publish status, and how the packages relate, lives in the manual:

- [Open the full package map](/manual/packages/)
- [Read the DESCRIPTION fields guide](/manual/description-fields/)
- [Read the full project tutorial](/full-project-tutorial/)

## Full reference

This page is a curated entry point. See the [user manual](/manual/packages/) for the exhaustive treatment, edge cases, and every feature switch.
