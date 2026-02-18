# TODO — miniextendr remaining work items

Last updated: 2026-02-18
Previous session completed Tier 1 + most of Tier 2-3 (commits b70e3cd..044146c).

---

## Tier 2 — Important

### B4. Safe mutable input helpers

Copy-in/copy-out pattern for users who want to "mutate" R vectors from Rust.
`&mut [T]` parameters are banned at the macro boundary because writing through
a slice backed by an R SEXP is unsafe: setting an element (e.g. to R_NilValue
in a `&mut [SEXP]`) can drop the last reference to an object, triggering GC,
which may shrink/move the backing storage — invalidating the slice pointer.
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

## Allow attribute cleanup (completed)

All items from the audit have been addressed:

- **Dead code allows**: 5 items in previous commit + 3 in this session
  (`has_default` field, `ReceiverKind::is_mut()`, `LifecycleSpec::roxygen_tags()`)
- **Dead parsed fields**: 3 items — 2 in previous commit, 1 in this session (`has_default`)
- **Trivial lint fixes**: 2 items — completed in previous commit
- **too_many_arguments**: 4 remaining → 3 resolved with param structs
  (`AltrepFamilyConfig`, `ColumnRegistry`, `AnalysisCtx`), 1 annotated
  (`derive_interaction_factor` — single call site, generics plumbing)
- **Feature-gate test-only APIs**: `preserve::count()` / `count_unchecked()`
  gated with `#[cfg(feature = "debug-preserve")]`
