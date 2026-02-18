# TODO — miniextendr remaining work items

Last updated: 2026-02-18
Previous session completed Tier 1 + most of Tier 2-3 (commits b70e3cd..044146c).

---

## Tier 2 — Important

### B4. Safe mutable input helpers

Copy-in/copy-out pattern for users who want to "mutate" R vectors from Rust.
R vectors are copy-on-write, so `&mut [T]` parameters are banned at the macro boundary.
The workaround is: accept `&[T]`, copy into `Vec<T>`, mutate, return `Vec<T>`.
Formalize this as a helper type or documented pattern with examples.

**Files:**
- `miniextendr-api/src/from_r.rs` (TryFromSexp for slices)
- `docs/GAPS.md` section 2.1 (documents the limitation)
- New tests in `rpkg/src/rust/` demonstrating the pattern

**Acceptance:** doc examples, at least one test showing copy-in/copy-out with NA handling.

---

### C1. String ndarray/matrix conversion (Gap 2.2)

R's STRSXP is a vector of CHARSXP pointers, not contiguous memory.
ndarray assumes contiguous backing. Need special-case conversion:
- `Array1<String>` / `Array1<Option<String>>` ↔ character vector
- `Array2<String>` ↔ character matrix (column-major)

**Files:**
- `miniextendr-api/src/into_r.rs` — add IntoR for Array1/Array2<String>
- `miniextendr-api/src/from_r.rs` — add TryFromSexp for same
- `rpkg/tests/testthat/test-ndarray.R` — R-level roundtrip tests
- Feature-gated behind `ndarray`

**Acceptance:** roundtrip tests for Array1<String>, Array2<String>, Array1<Option<String>> with NA.

---

## Tier 3 — Nice to have

### A5. Worker batching/context reuse API

Current `with_r_thread` is one message per call (440µs amortizable overhead).
Batch API to send multiple work items in one round-trip.

**Files:**
- `miniextendr-api/src/worker.rs:286-333`

**Acceptance:** batch API with benchmark showing amortized overhead.

---

### A6. Name-indexed list API

Optional cached name index for `List::get_named` and typed-list validation.
Currently every `get_named` call does a linear scan of R's names vector.

**Files:**
- `miniextendr-api/src/list.rs`
- `miniextendr-api/src/typed_list.rs`

**Acceptance:** indexed lookup API, benchmark showing improvement for repeated lookups.

---

### F3. Worker thread test re-enablement

`rpkg/tests/testthat/test-thread.R` and `test-thread-broken.R` have disabled tests.
Investigate RThreadBuilder issues, re-enable or document why they must stay skipped.

**Files:**
- `rpkg/tests/testthat/test-thread.R`
- `rpkg/tests/testthat/test-thread-broken.R`

---

### F4. Cross-package test expansion

Currently tests basic dispatch only. Expand to complex trait patterns,
version compatibility scenarios, multiple trait impls across packages.

**Files:**
- `tests/cross-package/consumer.pkg/tests/testthat/`
- `tests/cross-package/producer.pkg/src/rust/`

---

### G3. Connection & progress bar guides

Dedicated docs with usage examples. Currently only feature-gated rustdoc.

**Files:**
- New `docs/CONNECTIONS.md`
- New `docs/PROGRESS.md`
- Reference: `miniextendr-api/src/connection.rs`, `miniextendr-api/src/optionals/indicatif_bridge.rs`

---

### G4. Intermediate minirextendr vignettes

Currently only 1 getting-started vignette. Add:
- "Adding Rust Functions" hands-on tutorial
- ALTREP quick start

**Files:**
- `minirextendr/vignettes/`

---

## Tier 4 — When needed

### A7. Direct/no-wrapper export mode

Skip R wrapper layer for hot functions. Direct `.Call` entrypoints.
Benchmark shows 2x overhead vs direct (249ns vs 126ns).

**Files:** `miniextendr-macros/src/c_wrapper_builder.rs:920`

---

### C2. Module cfg friction reduction

Macro-level support for `#[cfg]`-aware module wiring.
Currently requires path-based module switching pattern.

**Files:** `miniextendr-macros/src/lib.rs`, `miniextendr-lint/src/`

---

### C4. Connections API stability

Capability probing, stronger runtime version checks, binary/stat support.

**Files:** `miniextendr-api/src/connection.rs`

---

### D1. Async-like handle model (Gap 4.3)

Handle objects for background tasks (poll/wait/cancel) on worker thread.

**Files:** `miniextendr-api/src/worker.rs`, `miniextendr-api/src/externalptr.rs`

---

### D2. Quoted-expression evaluation helpers (Gap 4.4)

Wrapper types for LANGSXP, safe `Rf_eval` with explicit environment.

**Files:** `miniextendr-api/src/from_r.rs`, `miniextendr-api/src/ffi.rs`

---

### D3. S4 compatibility helpers (Gap 3.4)

Helper generation for S4 wrapper patterns. Migration notes S4 → S7.

**Files:** `miniextendr-macros/src/miniextendr_impl.rs`, `docs/CLASS_SYSTEMS.md`

---

## Allow attribute cleanup

Small mechanical items from the audit (`analysis/allow_attributes_refactor_audit.md`,
now deleted but findings are captured here).

### Remove stale dead_code allows (5 items)

These helpers/methods appear unused:
- `miniextendr-api/src/list.rs:228` — `set_attr_impl_unchecked`
- `miniextendr-api/src/rarray.rs:574` — `set_attr_impl_unchecked`
- `miniextendr-macros/src/dataframe_derive.rs:290` — `column_count`
- `miniextendr-macros/src/lifecycle.rs:89` — `needs_signal`
- `miniextendr-macros/src/r_class_formatter.rs:310` — `with_export`

### Remove dead parsed fields (3 items)

Fields parsed but never consumed by codegen:
- `miniextendr-macros/src/miniextendr_impl_trait.rs:129` — `worker` field
- `miniextendr-macros/src/miniextendr_impl.rs:621` — `generics` field
- `miniextendr-macros/src/miniextendr_trait.rs:834` — `has_default` field

### Trivial lint fixes (2 items)

- `miniextendr-macros/src/lib.rs:2481` — `clippy::collapsible_if` (merge nested if-let)
- `miniextendr-lint/src/crate_index.rs:353` — `clippy::only_used_in_recursion` (drop unused `path` param)

### too_many_arguments (5 functions, needs param structs)

These have 8+ parameters. Introduce `*Ctx`/`*Config` structs:
- `miniextendr-macros/src/altrep_derive.rs:145`
- `miniextendr-macros/src/dataframe_derive.rs:1267`
- `miniextendr-macros/src/factor_derive.rs:265`
- `miniextendr-macros/src/miniextendr_impl_trait.rs:1197`
- `miniextendr-macros/src/return_type_analysis.rs:160`

### Feature-gate test-only APIs (2 items)

- `miniextendr-api/src/preserve.rs:133` — `count()`
- `miniextendr-api/src/preserve.rs:152` — `count_unchecked()`

Gate with `#[cfg(any(test, feature = "debug-preserve"))]`.
