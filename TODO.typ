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

== Testing

- [x] Property-based roundtrip tests — 24 proptest tests for all scalar/vector/option types
- [x] Macro codegen snapshot tests — 12 expect-test snapshots for R wrappers and class systems
- [ ] Thread-safety assertions not covered by tests
  - Note: Would require embedded R runtime for meaningful tests.
- [ ] String ALTREP NA serialization — `saveRDS`/`readRDS` loses NA (becomes empty string)

== Build / Infrastructure

- [x] processx-based execution in minirextendr — migrated from system2()
- [ ] Windows CI debugging
  - The `-l` (login) flag in bash might change working directory
  - Path format when passing Windows paths to bash

== Optional Features

=== Serialization

- [x] `borsh` optional feature for binary serialization
  - `Borsh<T>` wrapper: `IntoR` → RAWSXP, `TryFromSexp` → decode RAWSXP
  - `RBorshOps` adapter trait with blanket impl
- [ ] `rkyv` optional feature for zero-copy serialization (DEFERRED)
  - Complex: lifetime/validation issues with R's GC model
  - Revisit after borsh is in place

=== Adapter Traits

- [x] serde_json R list bridge — direct SEXP ↔ JsonValue conversion
  - `JsonValue` IntoR/TryFromSexp, homogeneous array optimization
  - NA/NaN/Inf handling via `JsonOptions`, factor support

=== Concurrency (POSTPONED)

- [ ] crossbeam channel adapters (`RSender`, `RReceiver`)
- [ ] Future/async adapters (`RFuture`) — requires async runtime integration

== Low Priority / Nice to Have

- [ ] `miniextendr.yml` config file support for user defaults (yaml package)
- [ ] `lifecycle` package for deprecation warnings and API evolution
- [ ] `num-traits` as internal helper for generic numeric implementations
