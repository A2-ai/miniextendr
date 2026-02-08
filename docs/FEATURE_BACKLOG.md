# Feature Backlog

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

### 4. Strict conversion mode — impl methods ✓

- `#[miniextendr(r6, strict)]` (or any class system + `strict`) extends strict
  output conversion to impl block methods.
- `strict` field added to `ImplAttrs`, `ParsedImpl`, `CWrapperContext`, and builder.
- `sexp_conversion_expr()` helper on `CWrapperContext` handles bare, `Option<T>`,
  and `Result<T, E>` return types — delegates to `strict_conversion_for_type()`.
- All 6 `IntoR` return handling branches (3 main thread + 3 worker thread) updated.

### 5. Strict conversion mode — inputs (TryCoerce) ✓

- `#[miniextendr(strict)]` now also validates input parameters for lossy types
  (i64/u64/isize/usize + Vec variants).
- Only INTSXP and REALSXP accepted; RAWSXP and LGLSXP rejected with
  "strict conversion failed for parameter '{name}'" error.
- REALSXP values go through `TryCoerce` to catch fractional, NaN, and overflow.
- Strict takes priority over `coerce` for lossy types.
- Wired through both standalone functions (`lib.rs`) and impl methods
  (`c_wrapper_builder.rs`) via `RustConversionBuilder::with_strict()`.

## Active: Next Up

## Parked: Needs Evidence

These items are plausible but lack a demonstrated need or clear design.

- **Pooled PROTECT scope** — `Rf_protect()` is already cheap (stack counter increment).
  No benchmark shows this is a bottleneck. Revisit if profiling reveals protect overhead
  in tight loops.
- **ALTREP region prefetch** — R controls `get_region` invocation, not the ALTREP class.
  Can't prefetch from the implementor side as described. Reframe as docs/examples on
  efficient `get_region` implementation if needed.
- **Microbenchmark gating in CI** — valuable but depends on CI infrastructure (not yet
  set up). Revisit when CI exists.

## Dropped

- **Wrapper diff mode** — `git diff` already handles this. Wrappers are mechanical
  `.Call()` functions; a dedicated diff tool adds complexity for no real gain.

## Backlog: Nice to Have

- **Rustdoc examples for feature-gated modules** — one example per feature module
  (vctrs, rayon, serde, connections). Lowers discovery cost.
- **Conversion behavior matrix** — test-driven markdown artifact for edge cases
  (NA, overflow, UTF-8, class checks). Keeps docs and behavior in sync.
