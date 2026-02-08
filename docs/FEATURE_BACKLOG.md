# Feature Backlog

This document lists practical feature candidates for upcoming maintenance cycles.
Items are scoped to be incremental and compatible with the current architecture.

## Active: Next Up

### 1. Harden ALTREP registration diagnostics

- **Goal:** make class-registration failures actionable in package init logs.
- **Scope:** validate `R_make_alt*` return values; include class name, base type, and
  missing required callbacks in error output.
- **Effort:** Low — add validation in `altrep_bridge.rs` install methods.

### 2. Strict conversion mode

- **Goal:** fail fast on lossy numeric coercions at wrapper boundaries.
- **Scope:** `#[miniextendr(strict)]` attribute that toggles conversion policy.
  Infrastructure already exists (`StorageCoerceError`, `into_r_as`).
- **Effort:** Medium — wrapper generation in macros + conversion path selection.
- **Open questions:** inputs only, outputs only, or both?

### 3. Structured panic telemetry hook

- **Goal:** capture panic class (`panic`, `r_error`, `thread_violation`) and function id.
- **Scope:** optional callback in `worker` between `catch_unwind` and error-raise.
  Worker already captures panics; this adds a structured hook point.
- **Effort:** Low — hook sits between panic capture and `panic_message_to_r_error()`.

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
