# miniextendr Documentation

Comprehensive documentation for the miniextendr Rust-R interoperability
framework.

## Quick links

| I want to... | Read |
|---|---|
| Build my first package | [Getting Started](GETTING_STARTED.md) |
| Use the CLI tool | [miniextendr CLI](../miniextendr-cli/README.md) |
| Use the R scaffolding helper | [minirextendr](MINIREXTENDR.md) |
| Add a Rust function to R | [Type Conversions](TYPE_CONVERSIONS.md) |
| Expose a Rust struct to R | [ExternalPtr](EXTERNALPTR.md) |
| Work with R matrices/arrays | [RArray](RARRAY.md) |
| Generate random numbers | [RNG](RNG.md) |
| Use lazy/compact vectors | [ALTREP](ALTREP.md) |
| Understand the architecture | [Architecture](ARCHITECTURE.md) |
| See what features exist | [Features](FEATURES.md) |
| Debug a problem | [Troubleshooting](TROUBLESHOOTING.md) |

## Documentation map

### Getting started

- **[Getting Started](GETTING_STARTED.md)** -- End-to-end guide: create a
  package, write Rust, call from R
- **[Developer Workflow](DEVELOPER_WORKFLOW.md)** -- Day-to-day development
  commands and patterns

### Core concepts

- **[Architecture](ARCHITECTURE.md)** -- Crate structure, call flow, how Rust
  talks to R
- **[Type Conversions](TYPE_CONVERSIONS.md)** -- `TryFromSexp` / `IntoR`
  system, `NamedList`
- **[Expression Evaluation](EXPRESSION_EVAL.md)** -- `RSymbol`, `RCall`,
  `REnv` for calling R from Rust
- **[Error Handling](ERROR_HANDLING.md)** -- Panics, R errors, `Result<T>`,
  `error_in_r` mode, backtrace control
- **[ExternalPtr](EXTERNALPTR.md)** -- Box-like owned pointer wrapping R's
  `EXTPTRSXP`
- **[GC Protection](GC_PROTECT.md)** -- RAII-based protect/unprotect
- **[R Allocator](ALLOCATOR.md)** -- R-backed `GlobalAlloc` using `RAWSXP` plus
  preserve lists
- **[FFI Guard & Panic Telemetry](FFI_GUARD.md)** -- Panic boundaries and
  telemetry hooks
- **[Encoding](ENCODING.md)** -- UTF-8 locale requirement and encoding probing
- **[RNG](RNG.md)** -- R random number generation from Rust
- **[RArray](RARRAY.md)** -- N-dimensional R arrays
- **[Safety](SAFETY.md)** -- Safety invariants and guarantees
- **[Threads](THREADS.md)** -- Worker-thread architecture and main-thread
  safety

### Macro reference

- **[`#[miniextendr]` Attribute](MINIEXTENDR_ATTRIBUTE.md)** -- Complete
  reference for functions, impls, traits, structs, and enums

### Class systems

- **[Class Systems](CLASS_SYSTEMS.md)** -- Env (default), R6, S3, S4, S7
  generation plus S4 helpers
- **[S3 Methods](S3_METHODS.md)** -- Implementing print, format, and dots with
  S3

### Features

| Feature | Guide | What it does |
|---|---|---|
| ALTREP | [ALTREP](ALTREP.md), [Receiving ALTREP](ALTREP_SEXP.md), [Examples](ALTREP_EXAMPLES.md), [Quick Ref](ALTREP_QUICKREF.md), [Guards](ALTREP_GUARDS.md) | Lazy/compact vectors via `#[derive(Altrep)]` |
| Enums & Factors | [Enums & Factors](ENUMS_AND_FACTORS.md) | `RFactor`, `MatchArg`, `FactorVec` |
| Connections | [Connections](CONNECTIONS.md) | Custom R connections from Rust |
| Progress bars | [Progress](PROGRESS.md) | indicatif progress bars routed through the R console |
| rayon | [Rayon](RAYON.md) | Parallel iteration with data-race safety |
| vctrs | [Vctrs](VCTRS.md) | vctrs integration with `#[derive(Vctrs)]` |
| `serde` | [Serde](SERDE_R.md) | Native Rust <-> R serialization |
| `serde_json` | [Serde](SERDE_R.md) | JSON-based serialization helpers |
| DataFrames | [DataFrames](DATAFRAME.md) | `#[derive(DataFrameRow)]` plus columnar / serde helpers |
| Dots | [Dots](DOTS_TYPED_LIST.md) | R's `...` args plus `typed_list!` validation |
| Adapters | [Adapter Traits](ADAPTER_TRAITS.md), [Cookbook](ADAPTER_COOKBOOK.md) | Export external crate traits to R |
| Lifecycle | [Lifecycle](LIFECYCLE.md) | Deprecation badges and runtime warnings |
| Prefer derives | [Prefer Derives](PREFER_DERIVES.md) | Control `IntoR` routing (list, `ExternalPtr`, native) |
| Strict mode | [Strict Mode](STRICT_MODE.md) | Reject lossy `i64` / `u64` conversions |
| Raw conversions | [Raw Conversions](RAW_CONVERSIONS.md) | POD types to/from R raw vectors via bytemuck |
| Feature defaults | [Feature Defaults](FEATURE_DEFAULTS.md) | Project-wide defaults (strict, coerce, class system) |
| All flags | [Features](FEATURES.md) | Complete feature flag reference |

### Cross-package interop

- **[Trait ABI](TRAIT_ABI.md)** -- How cross-package trait dispatch works
- **[Trait as R](TRAIT_AS_R.md)** -- Implementation details

### Build system

- **[R Build System](R_BUILD_SYSTEM.md)** -- How R builds packages with
  compiled code
- **[Templates](TEMPLATES.md)** -- `.in` template files and configure
- **[minirextendr](MINIREXTENDR.md)** -- Scaffolding, vendoring, and R workflow
  helpers
- **[Entrypoint](ENTRYPOINT.md)** -- R package entry point (`R_init_*`)
- **[Vendoring](VENDOR.md)** -- Dependency vendoring and CRAN release prep
- **[Linking](LINKING.md)** -- Shared library linking strategy
- **[Environment Variables](ENVIRONMENT_VARIABLES.md)** -- Env vars affecting
  build/configure/lint
- **[Non-API Tracking](NONAPI.md)** -- Non-API R symbols used for CRAN
  compliance
- **[Engine](ENGINE.md)** -- `miniextendr-engine` for standalone R embedding
- **[Windows Build Environment](windows-build-environment.md)** -- Reproducing
  the MSYS2 / Rtools environment R uses on Windows

### Type system deep dive

- **[Conversion Matrix](CONVERSION_MATRIX.md)** -- R type x Rust type behavior
  reference
- **[Coercion](COERCE.md)** -- Automatic type coercion
- **[as.class() Methods](AS_COERCE.md)** -- `as.<class>()` coercion methods
- **[Extending](EXTENDING_MINIEXTENDR.md)** -- Adding custom types to
  miniextendr
- **[Orphan Rule Challenges](ORPHAN_RULE_CHALLENGES.md)** -- Design notes on
  trait coherence constraints

### Testing and debugging

- **[Smoke Tests](SMOKE_TEST.md)** -- Quick/standard/demanding test lanes
- **[Troubleshooting](TROUBLESHOOTING.md)** -- Common issues and solutions
- **[Macro Errors](MACRO_ERRORS.md)** -- All MXL error codes explained
- **[Sparse Iterator ALTREP](SPARSE_ITERATOR_ALTREP.md)** -- Sparse-cache
  iterator ALTREP design and tradeoffs

### Benchmarks

- **[Performance Baseline](BENCHMARKS.md)** -- Benchmarks across subsystems

### CLI

- **[miniextendr CLI](../miniextendr-cli/README.md)** -- Standalone command-line
  tool for build, workflow, vendor, and cargo operations

### Project status

- **[Known Gaps](GAPS.md)** -- What's missing, what's broken, and why
- **[Feature Backlog](FEATURE_BACKLOG.md)** -- Proposed features and sequencing
- **[Maintainer Guide](MAINTAINER.md)** -- Release process and maintenance
  tasks
