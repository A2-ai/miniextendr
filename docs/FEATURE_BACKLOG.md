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
- Impl method support deferred (uses separate `CWrapperContext` codegen path).

## Active: Next Up

### 4. Strict conversion mode — impl methods

- **Goal:** extend `#[miniextendr(strict)]` to impl block methods.
- **Scope:** add `strict` field to `CWrapperContext`/builder, modify
  `generate_return_handling` and `generate_worker_return_handling` to emit
  `strict::checked_*()` for lossy return types when strict is set.
- **Effort:** Medium — mirrors standalone fn approach but in the builder path.

### 5. Strict conversion mode — inputs (TryCoerce)

- **Goal:** opt-in strict input coercion that rejects lossy narrowing.
- **Scope:** `#[miniextendr(strict)]` also affects parameter conversion,
  using `TryCoerce` or a similar mechanism to reject widening inputs.
- **Effort:** Medium — needs design for how strict inputs interact with coerce.

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
