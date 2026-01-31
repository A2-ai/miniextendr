#import "@preview/cheq:0.3.0": checklist
#show: checklist

#set page(numbering: "1 of 1", paper: "a5", margin: (top: 2em, left: 1em, right: 2em, bottom: 2.2em))


== Codex Review Findings (2024)

=== API/Ergonomics
- [x] `#[miniextendr]` impl blocks: consuming `self` by value not fully supported
  - DONE: Added compile error for consuming `self` unless constructor/finalizer (2026-01-31)
  - `miniextendr-macros/src/miniextendr_impl.rs:1024-1052`
=== Testing
- [ ] No automated regression test for registration bug
  - Note: User indicated this is likely a fluke, low priority.
- [ ] Thread-safety assertions not covered by tests
  - Note: Would require embedded R runtime for meaningful tests.


== GC Protect Review Findings (2026-01-12)
See `reviews/gc_protect_review.md` for full context.

=== Testing Gaps
- [x] Add tests for TLS panic cleanup behavior
  - DONE: `tls_cleanup_on_panic`, `tls_nested_cleanup_on_panic` in gc_protect.rs (2026-01-31)
- [x] Add tests for `ReprotectSlot::set` invalidation semantics
  - DONE: `reprotect_slot_old_value_unprotected`, `reprotect_slot_multiple_replacements` (2026-01-31)

== ExternalPtr Sidecar R Wrappers (Planned Feature)
See `plans/externalptr_sidecar_r_wrappers.md` for full design.

=== Overview
Expose `#[r_data]` sidecar fields from `ExternalPtr` structs to R as getter/setter functions.

=== Implementation Tasks
- [x] The `rdname` is not default to file-name for the sidecar impls.
  - DONE: Added documentation header with `@name {type}` (2026-01-31)

=== Tests
- [x] UI tests: multiple selector fields error, generic type error
  - DONE: `rdata_multiple_selectors.rs`, `rdata_generic_with_pub.rs`
  - Note: "non-marker type error" N/A - current design accepts any type as sidecar slot
- [x] Runtime tests: getter returns stored SEXP, setter updates and returns invisible(x)
  - DONE: Added `invisible(x)` return tests, SEXP identity tests in test-sidecar.R (2026-01-31)

=== Reference Study Tasks (from background/)

==== Class System References
- [ ] Study S7 patterns (`background/S7-main/`) for class generation
  - How does S7 handle method dispatch?
  - How does S7 generate constructors?
  - Patterns for property access/validation
  - Reference for improving #[miniextendr] impl block codegen
- [ ] Study R6 patterns (`background/R6-main/`) for reference class generation
  - How does R6 handle private vs. public fields?
  - How does R6 handle inheritance?
  - Reference for R6 class generator in miniextendr-macros
- [ ] Study vctrs patterns (`background/vctrs-main/`) for type coercion
  - How does vctrs handle type casting?
  - Recycling rules for binary operations
  - Patterns for Vec<T> / Option<T> conversions in miniextendr-api

==== ALTREP Implementation References
- [ ] Study `background/Rpkg-mutable-master/` for mutable ALTREP patterns
  - How does it handle write barriers?
  - How does it handle copy-on-modify semantics?
- [ ] Study `background/Rpkg-simplemmap-master/` for memory-mapped ALTREP
  - How does it handle lazy loading?
  - How does it handle file descriptor lifecycle?
- [ ] Study `background/vectorwindow-main/` for ALTREP views
  - How does it implement subset views without copying?
  - How does it handle window lifecycle?

==== Documentation & Tooling References
- [ ] Study roxygen2 (`background/roxygen2-main/`) for R wrapper generation
  - How does roxygen2 parse `@param`, `@return`, `@export` tags?
  - Patterns for improving miniextendr-macros/src/roxygen.rs
  - Reference for R documentation generation
- [ ] Study mirai (`background/mirai-main/`) for async patterns
  - How does mirai handle clean environment evaluation?
  - Patterns for worker thread communication
  - Reference for potential async miniextendr features

=== Build/Infrastructure (from reviews 03, 04, 07)
- [ ] Consider processx-based execution in minirextendr
  - `reviews/07_minirextendr.md` section "system2() portability"
  - Location: `minirextendr/R/*.R`
  - Purpose: Better cross-platform command execution with proper quoting/output capture
  - Note: processx is common in R tooling ecosystem

=== Optional Enhancements (lower priority)
==== minirextendr Dependency Rationalization
Source: `reviews/dependency-idiomaticity.md`

Strong fit (replace manual code):
Good fit (add functionality):
Optional:
- [ ] Add `miniextendr.yml` config file support for user defaults using `yaml` package
  - Store: crate name, rpkg name, version, features
- [ ] Add `clipr` for copying "next steps" commands to clipboard
- [ ] Add `lifecycle` for deprecation warnings and API evolution

==== minirextendr usethis Replacements
Source: `reviews/usethis-replacements.md`

checking available recipes (`just --list`) - ALL EXIST
=== Planned: Feature shortlist from Rust ecosystem
Source: `reviews/feature-plans-uuid-time-regex-indexmap.md`, `reviews/feature-shortlist.md`

Common scaffolding for all features:
1. Add optional dep + feature in `miniextendr-api/Cargo.toml` (non-default)
2. Create feature module: `*_impl.rs`
3. Gate module in `lib.rs` with `#[cfg(feature = "...")]`
4. Add doc block per feature in `lib.rs` with example snippets
5. Add feature-gated tests under `miniextendr-api/tests/`

=== Planned: External-trait export strategy
Source: `reviews/trait-export-and-numeric-crates.md`

\*Key constraint:\* Cannot directly export external (non-owned) traits to R.

Solution: Adapter trait pattern
=== Planned: Numeric crate feature candidates
Source: `reviews/trait-export-and-numeric-crates.md`

Common scaffolding (same as feature shortlist):
1. Add optional dep + feature in `miniextendr-api/Cargo.toml`
2. Create `*_impl.rs` module
3. Gate module with `#[cfg(feature = "...")]`
4. Add doc block + tests

==== num-traits (internal only)
- [ ] Optional helper for generic implementations
- [ ] NOT a public R-facing feature (internal use only)
- [ ] Consider for implementing generic numeric helpers

==== rug (LGPL + system GMP)
- [ ] Keep out of defaults due to LGPL license and system GMP dependency
- [ ] Document as advanced/opt-in if ever added
- [ ] Include clear license notes if implemented

=== Planned: Additional Adapter Trait Candidates
Source: ADAPTER_TRAITS.md pattern - applicable to many external traits

The adapter trait pattern (local trait + blanket impl) enables exporting external traits to R.
Each candidate below can follow the pattern documented in ADAPTER_TRAITS.md.

==== std library traits

Iterator adapter:
Display/FromStr adapters:
Debug adapter:

==== serde trait adapters (with serde feature)

- [ ] Consider serde_json R list bridge
  - Direct SEXP serialization without JSON intermediate
  - Similar to jsonlite's R ↔ JSON model



==== IO trait adapters (with connections feature)

NOTE: IO adapters are provided by the connection module (`miniextendr-api/src/connection.rs`):
- `IoRead<T>` for `T: std::io::Read`
- `IoWrite<T>` for `T: std::io::Write`
- `IoBufRead<T>` for `T: std::io::BufRead`
- `IoReadWrite<T>`, `IoReadSeek<T>`, `IoWriteSeek<T>`, `IoReadWriteSeek<T>`
- Use `RConnectionIo` builder for easy creation

Standalone adapter traits not needed - use connection framework instead.










==== crossbeam channel adapters (potential new feature) - POSTPONED

*POSTPONED:* Do these last - complex concurrency patterns require careful design.

- [ ] Add `crossbeam-channel = { version = "0.5", optional = true }` feature (if useful)
- [ ] Create `RSender` adapter trait for channel senders
  - `r_send(&self, item: T) -> bool` - send item, return success
  - `r_try_send(&self, item: T) -> Option<T>` - non-blocking send
  - `r_is_full(&self) -> bool` - check if channel full
  - Use case: Inter-thread communication from R
- [ ] Create `RReceiver` adapter trait for channel receivers
  - `r_recv(&self) -> Option<T>` - blocking receive
  - `r_try_recv(&self) -> Option<T>` - non-blocking receive
  - `r_is_empty(&self) -> bool` - check if channel empty
  - Use case: Receive from background threads in R

==== Future/async adapters (long-term, if async support added) - POSTPONED

*POSTPONED:* Do these last - requires async runtime integration and careful design around R's single-threaded nature.

- [ ] Create `RFuture` adapter trait for `std::future::Future`
  - `r_poll(&mut self) -> Option<T>` - check if ready (simplified poll)
  - `r_block_on(&mut self) -> T` - blocking wait (using tokio/async-std runtime)
  - Use case: Basic async/await integration with R
  - Note: Requires careful design around R's single-threaded nature







== New Optional Features (from reviews/ plans 2026-01-04)




==== borsh feature

- [ ] Add `borsh` optional feature for binary serialization
  - `borsh = { version = "1", optional = true }`
  - Create `miniextendr-api/src/borsh_impl.rs`
  - Wrapper type: `Borsh<T>` to avoid trait conflicts
  - `IntoR for Borsh<T>` where `T: BorshSerialize` → RAWSXP
  - `TryFromSexp for Borsh<T>` where `T: BorshDeserialize` → decode RAWSXP
  - Helpers: `borsh_to_raw()`, `borsh_from_raw()`
  - Map decode errors to `SexpError::InvalidValue`
  - Optional `mx_version` attribute for versioning
  - Plan: `reviews/borsh-rkyv-plan.md`

==== rkyv feature

- [ ] Add `rkyv` optional feature for zero-copy serialization
  - `rkyv = { version = "0.7", optional = true }`
  - `bytecheck = { version = "0.6", optional = true }` (required for safety)
  - Create `miniextendr-api/src/rkyv_impl.rs`
  - Wrapper type: `Rkyv<T>` to avoid trait conflicts
  - `IntoR for Rkyv<T>` → RAWSXP
  - `TryFromSexp for Rkyv<T>` → validate with bytecheck before deserialize
  - Always use `rkyv::check_archived_root` to avoid UB
  - Plan: `reviews/borsh-rkyv-plan.md`









== Test Infrastructure (from reviews/ plans)

==== rpkg adapter trait tests

- Plan: `reviews/rpkg-adapter-tests-plan.md`



== Coerce Integration (from coerce-coverage-review-2026-01-04)

==== Feature module Coerce/TryCoerce integration

- [x] Add `Coerced<T, R>` support to container types in feature modules (2026-01-31)
  - Added to `tinyvec_impl.rs`: TinyVec/ArrayVec with Coerced elements
  - Added to `nalgebra_impl.rs`: DVector/DMatrix with Coerced elements
- [~] Use `Coerced<T, R>` in scalar feature `TryFromSexp` impls
  - Analysis: Direct impls (e.g., `TryFromSexp for OrderedFloat<f64>`) are cleaner for scalar types
  - `Coerced<T, R>` wrapper is better suited for container element coercion
  - Feature modules like `ordered_float_impl` already have proper Coerce/TryCoerce impls
- [x] Document per-feature coercion policy (2026-01-31)
  - DONE: Added "Feature Module Coercion Policies" section to docs/COERCE.md
  - Clarifies integer inputs for float-centric types (ordered-float, rust-decimal)
  - Documents truncation/rounding behavior
  - Notes lossy vs strict conversions (rust-decimal string vs numeric)
  - Covers Coerced<T, R> pattern for container types



== ALTREP Serialization Gaps - FIXED

=== Problem (RESOLVED)

The `AltrepSerialize` trait was implemented on data types (Vec, Box, Range) but the bridge to R's `Serialized_state` ALTREP method was not connected.

=== Solution Implemented

1. Added `serialize` option to macros: `impl_altlogical_from_data!`, `impl_altraw_from_data!`, `impl_altstring_from_data!`
2. Added `AltrepSerialize` implementations for Range<i32>, Range<i64>, Range<f64>
3. Updated all builtin type macro invocations to use the `serialize` option

Types now correctly serialize via `Serialized_state` method:
=== Test Coverage

Tests exist in `rpkg/tests/testthat/test-altrep-serialization.R`:
- All tests should pass now that serialization bridge is implemented
- Remove any remaining `skip()` calls after verification

== Windows CI Debugging (In Progress)

=== Current Status
- Pushed fix to `rpkg/bootstrap.R` that captures configure.win output (commit 06be75c)
- CI run 21034458320 in progress, Windows job pending

=== What to Check
- [ ] Look at Windows job logs from run 21034458320 for actual configure.win error
- [ ] The error was previously hidden due to `stdout = ""` suppressing output

=== Potential Issues to Investigate
- The `-l` (login) flag in bash might change working directory
- Path format when passing Windows paths to bash
- Something in configure script itself failing

=== To Revert Debugging Changes

After finding the issue, revert the verbose debugging in `rpkg/bootstrap.R`:

```bash
git revert 06be75c
```

Or manually simplify back to minimal output:
- In `run_cmd` function: change `stdout = TRUE, stderr = TRUE` back to `stdout = "", stderr = ""`
- In Windows section: remove the extra `message()` calls and simplify `system2()` call

== Deep Integration Plans (See plans/ directory)

Three comprehensive plans for deeper R class system integration. Each has detailed implementation specs in `plans/`.

=== vctrs Integration (`plans/vctrs-deep-integration-plan.md`)
*Status: Core derive macro COMPLETE, advanced features pending*

*Goal:* `#[derive(Vctrs)]` to auto-generate vctrs-compatible S3 classes from Rust types.

*Current state:*
- Runtime: `new_vctr()`, `new_rcrd()`, `new_list_of()` in `miniextendr-api/src/vctrs.rs`
- Traits: `VctrsClass`, `IntoVctrs`, `VctrsRecord`, `VctrsListOf`
- Derive macro: Complete with all core S3 methods

*Completed (2026-01-16):*
- [x] `#[derive(Vctrs)]` proc-macro
- [x] Auto-generate R methods: `format.<class>`, `vec_ptype_abbr.<class>`, `vec_ptype_full.<class>`
- [x] Auto-generate `vec_proxy.<class>`, `vec_restore.<class>`
- [x] Auto-generate `vec_ptype2.<class>.<class>`, `vec_cast.<class>.<class>` (self-coercion)
- [x] `#[vctrs(coerce = "type")]` attribute for cross-type coercion
- [x] Record type support (`base = "record"`) with proper data frame proxy
- [x] All 51 R tests passing

*Remaining advanced features:*
- [ ] List-of type support (`base = "list"`)
- [ ] Proxy equal/compare/order methods
- [ ] Arithmetic/math method generation

=== R6 Integration (`plans/r6-deep-integration-plan.md`)
*Status: Basic support exists, advanced features not started*

*Goal:* Generate full R6 classes from Rust with minimal annotations.

*Current state:*
- Basic R6Class generation from `#[miniextendr(r6)]`
- Active bindings with `#[miniextendr(r6(active))]`
- Public/private member support

*To implement:*
- [ ] Inheritance (`inherit = ParentType`)
- [ ] `portable` / `non_portable` flags
- [ ] `lock_objects` / `lock_class` flags
- [ ] `cloneable` flag with deep clone hook
- [ ] Finalizer as private member
- [ ] Active binding getter+setter pairs
- [ ] Field-level `#[r6(public|private|skip)]` annotations

=== S7 Integration (`plans/s7-computed-properties-plan.md`)
*Status: Basic support exists, comprehensive features not started*

*Goal:* Full S7 class generation with property inference, generics, and validation.

*Current state:*
- Basic `new_class()` generation from `#[miniextendr(s7)]`
- Property support via `#[externalptr(s7)]`

*To implement (5 phases):*
- [ ] Phase 1: Property inference + accessor wiring
- [ ] Phase 2: Validation, defaults, required/frozen/deprecated patterns
- [ ] Phase 3: Generics (multi-dispatch, no-dots, optional/required args)
- [ ] Phase 4: `convert()` from Rust `From`/`TryFrom`, S3/S4 interop
- [ ] Phase 5: Docs, tests, stabilization

=== Recommended Next Target

*R6 Integration* is the recommended next target because:
1. Basic R6 support already exists
2. Build on the vctrs derive pattern for method generation
3. Inheritance and portable/cloneable flags are well-understood R6 features
4. Clear value for users wanting reference class semantics

*vctrs `#[derive(Vctrs)]`* is now COMPLETE (core features). See above for remaining advanced features.
