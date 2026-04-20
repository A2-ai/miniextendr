+++
title = "miniextendr"
sort_by = "weight"
description = "Ship R packages with Rust backends using generated wrappers, ALTREP support, and CRAN-minded packaging."
+++

## Why teams pick miniextendr

miniextendr is a Rust-R interoperability framework built for packages that need to survive real tooling, real release processes, and real data sizes.

- **Macro-first exports**: mark functions and impl blocks with `#[miniextendr]` and keep R wrappers generated from Rust docs.
- **Runtime built for R's constraints**: unwind protection, GC-aware pointer types, and optional worker-thread execution when you need it.
- **Packaging that respects CRAN**: vendoring, configure-based builds, and template-driven scaffolding via `minirextendr`. The [`justfile`](https://github.com/A2-ai/miniextendr/blob/main/justfile) provides all monorepo build recipes.

## What the documentation covers

The guide pages below start broad and then narrow into specific subsystems. Use the manual when you want exhaustive behavior, feature switches, packaging details, or edge-case references.
