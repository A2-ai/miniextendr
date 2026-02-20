#import "@preview/cheq:0.3.0": checklist
#show: checklist

#set page(numbering: "1 of 1", paper: "a5", margin: (top: 2em, left: 1em, right: 2em, bottom: 2.2em))

== R6 Deep Integration

Basic R6 support exists (`#[miniextendr(r6)]`, active bindings, public/private).
See `plans/r6-deep-integration-plan.md` for full spec.

- [x] `lock_objects` / `lock_class` flags — fully working + tested
- [x] `cloneable` flag with deep clone hook — `#[miniextendr(r6(deep_clone))]` works
- [x] Finalizer as private member — `#[miniextendr(r6(finalize))]` works
- [x] Active binding getter+setter pairs — `#[miniextendr(r6(active))]` + `setter` works
- [x] Inheritance (`inherit = "ParentType"`) — 3-level chain tested (R6Animal → R6Dog → R6GoldenRetriever)
- [x] `portable` flag — tested (`portable = false` works)
- [~] Field-level `#[r6(public|private|skip)]` annotations
  - Architecturally impossible: proc macros can't access struct field definitions
  - Workaround: use `#[r_data]` sidecar pattern (documented in CLASS_SYSTEMS.md)

== API Gaps

- [ ] Safe mutable input helpers — copy-in/copy-out pattern for `&mut [T]`
  - `&mut [T]` banned at macro boundary (GC can invalidate slice pointer)
  - Formalize as helper type or documented pattern with examples
  - Files: `from_r.rs`, `docs/GAPS.md` §2.1
- [~] String ndarray/matrix conversion
  - ndarray is designed for numeric/Copy types; `String` doesn't fit the model
  - `Vec<String>` / `Vec<Vec<String>>` are the natural Rust representations
- [ ] Quoted-expression evaluation helpers
  - Wrapper types for LANGSXP, safe `Rf_eval` with explicit environment
- [ ] S4 compatibility helpers — helper generation, migration notes S4 → S7

== Performance

- [ ] Worker batching/context reuse API
  - Current `with_r_thread` is one message per call (440µs amortizable overhead)
  - Batch API to send multiple work items in one round-trip
- [ ] Direct/no-wrapper export mode for hot functions
  - Skip R wrapper layer, direct `.Call` entrypoints
  - Benchmark shows 2x overhead (249ns vs 126ns)
- [x] Name-indexed list API — `NamedList` wrapper with `HashMap<String, usize>`
  - O(1) lookup via `get()`, `contains()`, `get_raw()`; `TryFromSexp` for use as fn param

== Testing

- [x] Property-based roundtrip tests — 24 proptest tests for all scalar/vector/option types
- [x] Macro codegen snapshot tests — 12 expect-test snapshots for R wrappers and class systems
- [x] Thread-safety assertions — 198 R-level worker thread tests, checked FFI wrapper
  panic tests, RAII cleanup across thread boundaries (test-worker.R, panic_tests.rs)
  - Remaining gap: RThreadBuilder direct tests skipped (crashes R runtime)
- [x] String ALTREP NA serialization — fixed in cc115a7 (use `Vec<Option<String>>`,
  register `Vec_Option_String` ALTREP class)
- [~] Worker thread test re-enablement
  - `test-thread.R` — disabled anti-pattern tests (R API calls from worker thread); correctly disabled
  - `test-thread-broken.R` — `RThreadBuilder` crashes R's stack checking; needs R-level API changes
  - Both files serve as documentation; `test-worker.R` (198 tests) covers the correct patterns
- [ ] Cross-package test expansion
  - Currently tests basic dispatch only
  - Expand to complex trait patterns, version compat, multiple trait impls

== Build / Infrastructure

- [x] processx-based execution in minirextendr — migrated from system2()
- [ ] Windows CI debugging
  - The `-l` (login) flag in bash might change working directory
  - Path format when passing Windows paths to bash
- [ ] Module `#[cfg]` friction reduction
  - Macro-level support for `#[cfg]`-aware module wiring
  - Currently requires path-based module switching pattern

== Optional Features

=== Serialization

- [x] `borsh` optional feature for binary serialization
  - `Borsh<T>` wrapper: `IntoR` → RAWSXP, `TryFromSexp` → decode RAWSXP
  - `RBorshOps` adapter trait with blanket impl
- [ ] `rkyv` optional feature for zero-copy serialization (DEFERRED)
  - Complex: lifetime/validation issues with R's GC model

=== Adapter Traits

- [x] serde_json R list bridge — direct SEXP ↔ JsonValue conversion
  - `JsonValue` IntoR/TryFromSexp, homogeneous array optimization
  - NA/NaN/Inf handling via `JsonOptions`, factor support

=== Connections

- [ ] Connections API stability
  - Capability probing, stronger runtime version checks, binary/stat support

=== Concurrency (POSTPONED)

- [ ] crossbeam channel adapters (`RSender`, `RReceiver`)
- [ ] Future/async adapters (`RFuture`) — requires async runtime integration
- [ ] Async-like handle model for background tasks (poll/wait/cancel)

== Documentation

- [ ] Connection & progress bar guides (`docs/CONNECTIONS.md`, `docs/PROGRESS.md`)
- [ ] Intermediate minirextendr vignettes
  - "Adding Rust Functions" hands-on tutorial
  - ALTREP quick start

== Low Priority / Nice to Have

- [ ] `miniextendr.yml` config file support for user defaults (yaml package)
- [ ] `lifecycle` package for deprecation warnings and API evolution
- [x] `num-traits` as internal helper for generic numeric implementations
  - `RNum`, `RFloat`, `RSigned` adapter traits with blanket impls
  - Feature-gated: `miniextendr-api = { features = ["num-traits"] }`
