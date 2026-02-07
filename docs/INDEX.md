# Documentation Index

## Getting Started

- [GETTING_STARTED.md](GETTING_STARTED.md) -- End-to-end guide for creating your first miniextendr package

## Core Concepts

- [TYPE_CONVERSIONS.md](TYPE_CONVERSIONS.md) -- Rust <-> R type conversion system
- [ERROR_HANDLING.md](ERROR_HANDLING.md) -- Panic handling, R errors, and error propagation
- [GC_PROTECT.md](GC_PROTECT.md) -- GC protection toolkit (OwnedProtect, ProtectScope)
- [SAFETY.md](SAFETY.md) -- Safety invariants and guarantees
- [THREADS.md](THREADS.md) -- Worker thread architecture and thread safety

## Class Systems

- [CLASS_SYSTEMS.md](CLASS_SYSTEMS.md) -- S3, S4, R7 class generation from Rust types

## Features

- [ALTREP_INDEX.md](ALTREP_INDEX.md) -- ALTREP documentation index (lazy/compact vectors)
- [ALTREP.md](ALTREP.md) -- ALTREP overview and proc-macro usage
- [ALTREP_EXAMPLES.md](ALTREP_EXAMPLES.md) -- Practical ALTREP examples
- [ALTREP_QUICKREF.md](ALTREP_QUICKREF.md) -- ALTREP quick reference card
- [RAYON.md](RAYON.md) -- Parallel iteration with rayon
- [VCTRS.md](VCTRS.md) -- vctrs integration with `#[derive(Vctrs)]`
- [serde_r.md](serde_r.md) -- Direct Rust-R serialization via serde
- [dots_typed_list.md](dots_typed_list.md) -- R's `...` (dots) and `typed_list!` validation
- [dataframe.md](dataframe.md) -- Data frame conversion with `#[derive(DataFrameRow)]`

## Cross-Package

- [TRAIT_ABI.md](TRAIT_ABI.md) -- Cross-package trait dispatch ABI
- [TRAIT_AS_R.md](TRAIT_AS_R.md) -- Trait-based ABI implementation plan

## Build System

- [TEMPLATES.md](TEMPLATES.md) -- Template system (`.in` files and configure)
- [VENDOR.md](VENDOR.md) -- Vendoring strategy for crates.io and workspace crates
- [LINKING.md](LINKING.md) -- Linking strategy for shared libraries
- [ENTRYPOINT.md](ENTRYPOINT.md) -- R package entry point (`R_init_*`)
- [ENGINE.md](ENGINE.md) -- miniextendr-engine: code generation engine

## Adapter Traits

- [ADAPTER_TRAITS.md](ADAPTER_TRAITS.md) -- Exporting external traits to R
- [ADAPTER_COOKBOOK.md](ADAPTER_COOKBOOK.md) -- Adapter trait cookbook with recipes

## Type System Reference

- [COERCE.md](COERCE.md) -- Type coercion in miniextendr
- [CONVERSION_SEMANTICS.md](CONVERSION_SEMANTICS.md) -- Storage-directed conversion semantics
- [as_coerce.md](as_coerce.md) -- `as.<class>()` coercion methods

## Internals

- [NONAPI.md](NONAPI.md) -- Non-API R functions tracking
- [TRACK_CALLER.md](TRACK_CALLER.md) -- `#[track_caller]` in miniextendr

## Benchmarks

- [ALTREP_BENCHMARKS.md](ALTREP_BENCHMARKS.md) -- ALTREP performance benchmarks
- [ALTREP_PERFORMANCE_REPORT.md](ALTREP_PERFORMANCE_REPORT.md) -- ALTREP performance analysis report
- [ALTREP_AS_FOUNDATION.md](ALTREP_AS_FOUNDATION.md) -- ALTREP as a foundation: building blocks and conveniences

## Project Status

- [GAPS.md](GAPS.md) -- **Known gaps and limitations** (760+ lines of valuable context on what's missing, what's broken, and why)
- [MAINTAINER.md](MAINTAINER.md) -- Maintainer guide
- [docs.md](docs.md) -- Documentation meta-notes

## Archived

Superseded working notes moved to `archive/`:

- [archive/20251230_bench_summary.md](archive/20251230_bench_summary.md) -- Superseded by ALTREP_PERFORMANCE_REPORT.md
- [archive/summary_bench_2026-01-12.md](archive/summary_bench_2026-01-12.md) -- Superseded by ALTREP_PERFORMANCE_REPORT.md
- [archive/COERCE_AND_INTO_R_REVIEW.md](archive/COERCE_AND_INTO_R_REVIEW.md) -- Findings incorporated into COERCE.md
- [archive/PROTECTED_PLAN.md](archive/PROTECTED_PLAN.md) -- Planning doc, now implemented
