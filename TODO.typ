#import "@preview/cheq:0.3.0": checklist
#show: checklist

#set page(numbering: "1 of 1", paper: "a5", margin: (top: 2em, left: 1em, right: 2em, bottom: 2.2em))

== R6 Deep Integration (Recommended Next Target)

Basic R6 support exists (`#[miniextendr(r6)]`, active bindings, public/private).
See `plans/r6-deep-integration-plan.md` for full spec.

- [ ] Inheritance (`inherit = ParentType`)
- [ ] `portable` / `non_portable` flags
- [ ] `lock_objects` / `lock_class` flags
- [ ] `cloneable` flag with deep clone hook
- [ ] Finalizer as private member
- [ ] Active binding getter+setter pairs
- [ ] Field-level `#[r6(public|private|skip)]` annotations

== Testing

- [ ] Thread-safety assertions not covered by tests
  - Note: Would require embedded R runtime for meaningful tests.

== Build / Infrastructure

- [ ] Consider processx-based execution in minirextendr
  - Purpose: Better cross-platform command execution with proper quoting/output capture
  - Note: processx is common in R tooling ecosystem

- [ ] Windows CI debugging
  - The `-l` (login) flag in bash might change working directory
  - Path format when passing Windows paths to bash

== Optional Features

=== Serialization

- [ ] `borsh` optional feature for binary serialization
  - `Borsh<T>` wrapper: `IntoR` → RAWSXP, `TryFromSexp` → decode RAWSXP

- [ ] `rkyv` optional feature for zero-copy serialization
  - `Rkyv<T>` wrapper with bytecheck validation

=== Adapter Traits

- [ ] serde_json R list bridge (direct SEXP serialization without JSON intermediate)

=== Concurrency (POSTPONED)

- [ ] crossbeam channel adapters (`RSender`, `RReceiver`)
- [ ] Future/async adapters (`RFuture`) — requires async runtime integration

== Low Priority / Nice to Have

- [ ] `miniextendr.yml` config file support for user defaults (yaml package)
- [ ] `lifecycle` package for deprecation warnings and API evolution
- [ ] `num-traits` as internal helper for generic numeric implementations
