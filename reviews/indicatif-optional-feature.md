# Optional feature proposal: indicatif support for R console progress

Date: 2025-12-30
Scope: optional feature (opt-in) to make Rust-side progress bars usable from R.

## Summary

Add an **optional “indicatif” feature** to the Rust side of miniextendr that
enables progress bars via a custom `TermLike` backend which writes to the R
console. This keeps the default build lean, but makes it easy for users to opt
into progress bars for long-running Rust tasks.

This proposal follows the constraints you outlined:

- R console writes must be main-thread only.
- `TermLike` is implementable today, but verbose.
- A “stdout vs stderr” hint is useful for coloring/behavior heuristics.

## User-facing design (opt‑in)

- **Cargo feature:** `features = ["indicatif"]` on the Rust crate.
- **R API:** add a helper like `miniextendr::enable_progress()` or a flag in
  generated wrappers to opt into progress output.
- **Behavior defaults:** if not interactive (R_Interactive == 0), progress bars
  are hidden; if the frontend lacks ANSI support, degrade to “single‑line” mode.

## Implementation sketch (downstream, non‑breaking)

### 1) Rust: `TermLike` implementation for R

- New module (behind feature flag): `src/r/progress.rs` (or similar).
- Implements `indicatif::TermLike` by calling:
  - `ptr_R_WriteConsoleEx` for output (stdout/stderr)
  - `R_FlushConsole` for flushing
- Records the main thread ID at construction and **no‑ops off‑thread** to keep
  R safe when users call `enable_steady_tick()` elsewhere.

### 2) Construction API

- Provide a helper:
  - `fn r_progress_bar(len: u64, width: u16, stream: RStream) -> ProgressBar`
- Wire it into miniextendr wrappers via feature flag, so users can opt in without
  writing Rust by hand.

### 3) ANSI vs non‑ANSI fallback

If ANSI not available:

- implement cursor ops by returning `Ok(())` (no multi‑line control)
- rely on `\r` for single‑line progress
This prevents garbled output in non‑ANSI frontends (some RStudio contexts).

## Upstream indicatif improvements (optional but ideal)

To make `indicatif-for-r` smaller and more idiomatic, propose upstream changes
(non‑breaking, non‑R‑specific):

1) **ANSI defaults for TermLike methods**
   - Provide default impls for cursor movement, clear line, and write_line using
     ANSI escape sequences.
   - Downstream backends then only need `width()`, `write_str()`, `flush()`.

2) **Stream‑hint constructors for TermLike**
   - Add `term_like_stdout(...)` / `term_like_stderr(...)` (or a stream enum)
     so the draw target can be annotated as “stderr‑like”.
   - Helpful for color decisions and to align with R’s typical progress stream.

3) **Docs: thread‑affinity guidance**
   - Document that indicatif can draw from non‑main threads if steady tick is
     enabled; recommend that thread‑affine backends no‑op or buffer.

## Why this fits miniextendr

- Adds **no breaking changes**; only an optional feature and helpers.
- Keeps default builds lightweight.
- Enables users to build “native‑feeling” progress bars when running Rust
  functions from R.

## Open questions

- Should progress default to stderr (typical for progress) or stdout?
- How should ANSI capability be detected in R (option, env var, or heuristic)?
- Should we forbid `enable_steady_tick()` in the R wrapper, or allow with
  off‑thread no‑ops?

If you want, I can draft the Rust module and feature flag wiring, plus a small
R wrapper that exposes progress‑enabled helpers.
