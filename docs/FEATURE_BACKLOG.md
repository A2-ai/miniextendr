# Feature Backlog

This document lists practical feature candidates for upcoming maintenance cycles.
Items are scoped to be incremental and compatible with the current architecture.

## P0: Reliability and Safety

- Add a stable "strict conversion" mode flag for generated wrappers.
  - Goal: fail fast on lossy numeric coercions and implicit recycling.
  - Scope: `#[miniextendr(strict)]` option that toggles conversion policy at wrapper boundaries.
- Add structured panic telemetry hook.
  - Goal: capture panic class (`panic`, `r_error`, `thread_violation`) and function id.
  - Scope: optional callback in `worker` to report failures before raising R errors.
- Harden ALTREP registration diagnostics.
  - Goal: make class-registration failures actionable in package init logs.
  - Scope: include class name, base type, and missing required callbacks in error output.

## P1: Developer Experience

- Add generated wrapper diff mode.
  - Goal: show stable diffs for regenerated `R/miniextendr_wrappers.R`.
  - Scope: CLI/doc tool flag that prints changed symbols and signatures.
- Add rustdoc examples for feature-gated modules.
  - Goal: lower discovery cost for `vctrs`, `rayon`, `serde`, and `connections`.
  - Scope: one minimal compile-checked example per feature module.
- Add a conversion behavior matrix page generated from tests.
  - Goal: keep docs and behavior in sync for edge cases (NA, overflow, UTF-8, class checks).
  - Scope: test-driven markdown artifact under `docs/`.

## P2: Performance

- Add pooled PROTECT scope for tight conversion loops.
  - Goal: reduce protect/unprotect overhead in large vector conversions.
  - Scope: internal allocator for short-lived protections within a single call frame.
- Add optional ALTREP region prefetch helper.
  - Goal: reduce callback overhead for sequential scans.
  - Scope: helper trait method that fills caller buffers using adaptive chunk size.
- Add microbenchmark gating in CI for regressions.
  - Goal: detect conversion/ALTREP slowdowns before release.
  - Scope: threshold checks for selected `miniextendr-bench` targets.

## Candidate Sequence

1. strict conversion mode
2. wrapper diff mode
3. ALTREP registration diagnostics
4. panic telemetry hook
5. benchmark regression gating
