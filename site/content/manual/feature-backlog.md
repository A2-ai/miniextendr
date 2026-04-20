+++
title = "Feature Backlog"
weight = 66
description = "This document lists practical feature candidates for upcoming maintenance cycles. Items are scoped to be incremental and compatible with the current architecture."
+++

This document lists practical feature candidates for upcoming maintenance cycles.
Items are scoped to be incremental and compatible with the current architecture.

## Completed

### 1. Harden ALTREP registration diagnostics ✓

- `validate_altrep_class()` checks for null handles after `R_make_alt*_class()`.
- Wired into `make_class_by_base()`, all 7 `impl_inferbase_*` macros, and
  explicit-base codegen path.

### 2. Structured panic telemetry hook ✓

- `panic_telemetry` module with `PanicSource` enum and optional hook via `AtomicPtr`.
- `fire()` called at all 3 panic→R-error sites (worker, altrep_bridge, unwind_protect).
- Zero overhead when no hook is set (single atomic load).

### 3. Strict conversion mode (outputs, standalone fns) ✓

- `#[miniextendr(strict)]` on standalone functions panics instead of silently
  widening i64/u64/isize/usize to f64.
- `strict` module with `checked_into_sexp_*` helpers.
- Codegen wired via `return_type_analysis.rs` for lossy scalar and `Vec` return types.

### 4. Strict conversion mode: impl methods ✓

- `#[miniextendr(r6, strict)]` (or any class system + `strict`) extends strict
  output conversion to impl block methods.
- `strict` field added to `ImplAttrs`, `ParsedImpl`, `CWrapperContext`, and builder.
- `sexp_conversion_expr()` helper on `CWrapperContext` handles bare, `Option<T>`,
  and `Result<T, E>` return types, delegating to `strict_conversion_for_type()`.
- All 6 `IntoR` return handling branches (3 main thread + 3 worker thread) updated.

### 5. Strict conversion mode: inputs (TryCoerce) ✓

- `#[miniextendr(strict)]` now also validates input parameters for lossy types
  (i64/u64/isize/usize + Vec variants).
- Only INTSXP and REALSXP accepted; RAWSXP and LGLSXP rejected with
  "strict conversion failed for parameter '{name}'" error.
- REALSXP values go through `TryCoerce` to catch fractional, NaN, and overflow.
- Strict takes priority over `coerce` for lossy types.
- Wired through both standalone functions (`lib.rs`) and impl methods
  (`c_wrapper_builder.rs`) via `RustConversionBuilder::with_strict()`.

### 6. `#[miniextendr(internal)]` and `#[miniextendr(noexport)]` attributes ✓

- `internal` injects `@keywords internal` and suppresses `@export`.
- `noexport` suppresses `@export` only (no `@keywords internal`).
- Works on standalone functions and all 6 class system impl blocks
  (env, R6, S3, S4, S7, vctrs) via `ClassDocBuilder::with_export_control()`.

### 7. String ALTREP Dataptr ✓

- Bridge-layer materialization: Rust `Vec<String>`/`Box<[String]>` materialize into
  native R STRSXP cached in the ALTREP data2 slot.
- Enables `saveRDS`/`readRDS` roundtrip and `identical()` for string ALTREP vectors.
- Uses `DATAPTR_RO` with cast (DATAPTR is behind `nonapi` feature gate).

### 8. Adapter test coverage expansion ✓

- Added 3–5 edge-case functions per adapter for 13 thin adapter modules
  (sha2, aho-corasick, time, tinyvec, indexmap, ordered-float, toml, bytes,
  either, url, bitvec, bitflags, tabled).
- Corresponding R test expectations in `test-feature-adapters.R`.

### 9. Conversion behavior matrix ✓

- `docs/CONVERSION_MATRIX.md`: R input type × Rust target type → behavior reference.
- Covers INTSXP, REALSXP, LGLSXP, RAWSXP, STRSXP against i32, f64, u8, bool,
  String, i64/u64/isize/usize in normal, coerce, and strict modes.

### 10. Sparse iterator ALTREP guide ✓

- `docs/SPARSE_ITERATOR_ALTREP.md`: compute-on-access pattern, `Iterator::nth()`
  for efficient skipping, comparison with materialization, usage guidance.

### 11. vctrs documentation expansion ✓

- Expanded `docs/VCTRS.md` with record type example, list-of pattern,
  advanced coercion, and troubleshooting section.

### 12. Fix `has_roxygen_tag` for multi-word tags ✓

- `has_roxygen_tag("keywords internal")` was broken: `tag_names()` only extracts
  first word after `@`. Added multi-word branch matching full content after `@`.
- Added comprehensive unit tests for `has_roxygen_tag`, `tag_names`, `find_tag_value`.

### 13. `Vec<Option<T>>` IntoR for extended numeric types ✓

- Smart i32/f64 conversion for `Vec<Option<i64/u64/isize/usize>>`: checks if all
  non-None values fit i32 → INTSXP, otherwise REALSXP with NA_REAL for None.
- Simple coercion for `Vec<Option<i8/i16/u16/u32/f32>>` via widening macro.
- Strict mode: `checked_vec_option_{i64,u64,isize,usize}_into_sexp()` helpers.
- Proc-macro detection: `strict_conversion_for_type()` handles `Vec<Option<lossy>>`.

### 14. S7 multi-level inheritance tests ✓

- 3-level chain: `S7Animal` (abstract) → `S7Dog` → `S7GoldenRetriever`.
- R tests verify `S7::S7_inherits()` through full chain, abstract rejection.
- GAPS.md section 3.1 updated: inheritance chains marked as implemented.

### 15. Rustdoc examples for feature-gated modules ✓

- Added `/// # Examples` blocks (ignore-marked) to `progress.rs`, `vctrs.rs`,
  `connection.rs` for key public items.

### 16. Field access documentation ✓

- GAPS.md section 3.3 marked RESOLVED via `#[r_data]` + `RSidecar` sidecar pattern.
- CLASS_SYSTEMS.md: added "Field Access via Sidecar" subsection.
- GAPS.md section 2.3: `Vec<Option<T>>` updated to "Works (all scalar types)".

### 17. Fix String ALTREP NA serialization ✓

- `into_sexp_altrep` STRSXP branch now uses `Vec<Option<String>>` instead of `Vec<String>`,
  preserving `NA_character_` through `saveRDS`/`readRDS` roundtrips.
- Added `RegisterAltrep` + `InferBase` for `Vec<Option<String>>`.
- Test suite: 0 FAIL, 2868 PASS (previously 2 FAIL).

## Active: Next Up

## Parked: Needs Evidence

These items are plausible but lack a demonstrated need or clear design.

- **Pooled PROTECT scope**: `Rf_protect()` is already cheap (stack counter increment).
  No benchmark shows this is a bottleneck. Revisit if profiling reveals protect overhead
  in tight loops.
- **ALTREP region prefetch**: R controls `get_region` invocation, not the ALTREP class.
  Can't prefetch from the implementor side as described. Reframe as docs/examples on
  efficient `get_region` implementation if needed.
- **Microbenchmark gating in CI**: valuable but depends on CI infrastructure (not yet
  set up). Revisit when CI exists.

## Dropped

- **Wrapper diff mode**: `git diff` already handles this. Wrappers are mechanical
  `.Call()` functions; a dedicated diff tool adds complexity for no real gain.

## Backlog: Nice to Have
