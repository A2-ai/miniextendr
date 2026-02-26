# miniextendr Documentation

Comprehensive documentation for the miniextendr Rust-R interoperability framework.

## Quick Links

| I want to... | Read |
|---|---|
| Build my first package | [Getting Started](GETTING_STARTED.md) |
| Use the CLI tool | [miniextendr CLI](../miniextendr-cli/README.md) |
| Add a Rust function to R | [Type Conversions](TYPE_CONVERSIONS.md) |
| Use lazy/compact vectors | [ALTREP](ALTREP.md) |
| Understand the architecture | [Architecture](ARCHITECTURE.md) |
| See what features exist | [Features](FEATURES.md) |
| Debug a problem | [Troubleshooting](TROUBLESHOOTING.md) |

## Documentation Map

### Getting Started

- **[Getting Started](GETTING_STARTED.md)** -- End-to-end guide: create a package, write Rust, call from R
- **[Developer Workflow](DEVELOPER_WORKFLOW.md)** -- Day-to-day development commands and patterns

### Core Concepts

How miniextendr works under the hood.

- **[Architecture](ARCHITECTURE.md)** -- Crate structure, call flow, how Rust talks to R
- **[Type Conversions](TYPE_CONVERSIONS.md)** -- `TryFromSexp`/`IntoR` system, `NamedList`, `CopySliceMut`
- **[Expression Evaluation](EXPRESSION_EVAL.md)** -- `RSymbol`, `RCall`, `REnv` for calling R from Rust
- **[Error Handling](ERROR_HANDLING.md)** -- Panics, R errors, `Result<T>`, and error propagation
- **[GC Protection](GC_PROTECT.md)** -- RAII-based protect/unprotect (`OwnedProtect`, `ProtectScope`)
- **[Safety](SAFETY.md)** -- Safety invariants and what miniextendr guarantees
- **[Threads](THREADS.md)** -- Worker thread architecture, batching, main thread safety

### Macro Reference

- **[`#[miniextendr]` Attribute](MINIEXTENDR_ATTRIBUTE.md)** -- Complete reference: what `#[miniextendr]` does on fn, impl, trait, struct, enum

### Class Systems

Generate R class wrappers from Rust structs.

- **[Class Systems](CLASS_SYSTEMS.md)** -- Env (default), R6, S3, S4, S7 generation + S4 helpers
- **[S3 Methods](S3_METHODS.md)** -- Implementing print, format, and dots with S3

### Features

Optional capabilities enabled via Cargo feature flags or proc-macro attributes.

| Feature | Guide | What it does |
|---|---|---|
| ALTREP | [ALTREP](ALTREP.md), [Examples](ALTREP_EXAMPLES.md), [Quick Ref](ALTREP_QUICKREF.md), [Guards](ALTREP_GUARDS.md) | Lazy/compact vectors via `#[derive(Altrep)]` |
| Enums & Factors | [Enums & Factors](ENUMS_AND_FACTORS.md) | RFactor, MatchArg, EnumChoices, FactorVec |
| Connections | [Connections](CONNECTIONS.md) | Custom R connections from Rust |
| Progress bars | [Progress](PROGRESS.md) | indicatif progress bars routed through R console |
| rayon | [Rayon](RAYON.md) | Parallel iteration with data-race safety |
| vctrs | [Vctrs](VCTRS.md) | vctrs integration with `#[derive(Vctrs)]` |
| serde | [Serde](SERDE_R.md) | Direct Rust-R serialization |
| DataFrames | [DataFrames](DATAFRAME.md) | `#[derive(DataFrameRow)]` + parallel fill + serde columnar |
| Dots | [Dots](DOTS_TYPED_LIST.md) | R's `...` args + `typed_list!` validation |
| Adapters | [Adapter Traits](ADAPTER_TRAITS.md), [Cookbook](ADAPTER_COOKBOOK.md) | Export external crate traits to R |
| Lifecycle | [Lifecycle](LIFECYCLE.md) | Deprecation badges and runtime warnings |
| Prefer derives | [Prefer Derives](PREFER_DERIVES.md) | Control IntoR routing (list, ExternalPtr, native) |
| Strict mode | [Strict Mode](STRICT_MODE.md) | Reject lossy i64/u64 conversions |
| Feature defaults | [Feature Defaults](FEATURE_DEFAULTS.md) | Project-wide defaults (strict, coerce, class system) |
| All flags | [Features](FEATURES.md) | Complete feature flag reference |

### Cross-Package Interop

Share Rust types and trait dispatch across R packages.

- **[Trait ABI](TRAIT_ABI.md)** -- How cross-package trait dispatch works
- **[Trait as R](TRAIT_AS_R.md)** -- Implementation details

### Build System

How miniextendr packages are built, configured, and released.

- **[R Build System](R_BUILD_SYSTEM.md)** -- How R builds packages with compiled code
- **[Templates](TEMPLATES.md)** -- `.in` template files and configure
- **[Entrypoint](ENTRYPOINT.md)** -- R package entry point (`R_init_*`)
- **[Vendoring](VENDOR.md)** -- Dependency vendoring and CRAN release prep
- **[Linking](LINKING.md)** -- Shared library linking strategy
- **[Environment Variables](ENVIRONMENT_VARIABLES.md)** -- All env vars affecting build/configure/lint
- **[Non-API Tracking](NONAPI.md)** -- Non-API R symbols used (for CRAN compliance)
- **[Engine](ENGINE.md)** -- miniextendr-engine: standalone R embedding

### Type System Deep Dive

Advanced type conversion and coercion details.

- **[Conversion Matrix](CONVERSION_MATRIX.md)** -- R type x Rust type behavior reference
- **[Coercion](COERCE.md)** -- Automatic type coercion
- **[as.class() Methods](AS_COERCE.md)** -- `as.<class>()` coercion methods
- **[Extending](EXTENDING_MINIEXTENDR.md)** -- Adding custom types to miniextendr

### Testing & Debugging

- **[Smoke Tests](SMOKE_TEST.md)** -- Quick/standard/demanding test lanes
- **[Troubleshooting](TROUBLESHOOTING.md)** -- Common issues and solutions
- **[Macro Errors](MACRO_ERRORS.md)** -- All MXL error codes explained

### Benchmarks

- **[Performance Baseline](BENCHMARKS.md)** -- All subsystems benchmarked (2026-02-18)

### CLI

- **[miniextendr CLI](../miniextendr-cli/README.md)** -- Standalone command-line tool (`miniextendr`) for all build, workflow, vendor, and cargo operations

### Project Status

- **[Known Gaps](GAPS.md)** -- What's missing, what's broken, and why
- **[Feature Backlog](FEATURE_BACKLOG.md)** -- Proposed features and sequencing
- **[Maintainer Guide](MAINTAINER.md)** -- Release process and maintenance tasks
