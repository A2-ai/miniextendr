# Rustdoc tradeoff pass — master plan

## Motivation

Claude (and other AI agents) reach for the easy/permissive miniextendr API and skip the
stricter one, because the rustdoc doesn't tell them what each path buys. The codebase
has *several* paired APIs where this matters:

- `TryFromSexp` (lax, coercing) vs `strict::TryFromSexpStrict` (refuses NA, refuses truncation)
- `_unchecked` FFI variants vs the `r_ffi_checked`-generated thread-checked variants
- ALTREP guard modes (`unsafe` / `rust_unwind` / `r_unwind`)
- Hand-rolled `TryFromSexp + IntoR` vs serde vs `#[derive(DataFrameRow)]` / `#[derive(ExternalPtr)]`
- `Rf_error` vs `panic!()` vs `error!()` macro vs `Result<_, RErrorAdapter>`
- ALTREP field-based derive vs manual lowlevel traits
- Class systems: Env vs R6 vs S3 vs S4 vs S7 vs Vctrs
- Sidecar vs packed storage on ExternalPtr
- `Dots` raw access vs `typed_list!`
- `with_r_thread` vs running on worker thread

## What we found in the audit

`docs/` already has substantial coverage: CLASS_SYSTEMS.md (947 lines, flowchart),
PREFER_DERIVES.md, STRICT_MODE.md, ALTREP.md (1696 lines with field-vs-manual
flowchart), ALTREP_QUICKREF.md, VCTRS.md, DOTS_TYPED_LIST.md, SERDE_R.md.

The user's hypothesis was that `docs/` was bare. It isn't. What *is* bare is the
**rustdoc surface** — the API-level docs that Claude reads when navigating a crate
without leaving Rust. Module-level `//!` docs are present but explain WHAT, rarely
WHY. `lib.rs` has no "if you want to do X, start here" decision tree. Paired APIs
rarely cross-reference each other.

## Strategy

Pivot the effort from "write docs/ guides" to:

1. **Rustdoc first** — make module-level `//!` docs answer "which path?" for paired APIs, with concrete cross-references to the alternative. This is where Claude is during code generation.
2. **Crate-level decision map** — `lib.rs` gets a top-level "I want to do X" rustdoc table that points readers (in-crate) at the right module, *and* (out-of-crate) at the right `docs/` deep-dive.
3. **New `docs/API_CHOICE_MATRIX.md`** — single-page master index linking all the existing decision-tree docs, so a reader who enters `docs/` from a search lands on a hub, not the middle of one option.

Module-level rustdoc is the right venue for tradeoff text — confirmed. We avoid
duplicating the existing docs/ flowcharts; the rustdoc summarizes the choice in
2-5 lines and links to the deep-dive.

## PR carving

Six PRs, dispatched in parallel as Opus agents (each in its own worktree off
origin/main, opening its own draft PR). PR 1 (the top-level map) lives in the
coordination worktree and lands after the others so it can link to landed
content.

### PR 2 — Conversions: strict vs lax, derive vs hand-rolled
Files: `miniextendr-api/src/from_r.rs`, `from_r/*`, `into_r.rs`, `into_r/*`,
`strict.rs`, `coerce.rs`.
Scope: module-level `//!` doc rewrites that explicitly contrast strict vs lax,
add cross-refs between paired traits (`TryFromSexp` ↔ `TryFromSexpStrict`,
`Coerce` ↔ `TryCoerce`), document the NA / truncation / widening
behavior at the path level (not just per-fn). Link to `docs/STRICT_MODE.md`.

### PR 3 — FFI safety boundary
Files: `miniextendr-api/src/ffi.rs` (the generated bindings module),
`miniextendr-api/src/unwind_protect.rs`, `miniextendr-api/src/worker.rs`,
`miniextendr-api/src/thread.rs`, plus the `r_ffi_checked` proc macro in
`miniextendr-macros/`.
Scope: rustdoc the "checked vs unchecked vs raw" boundary. Cover: when
`_unchecked` is safe (ALTREP callbacks, inside `with_r_unwind_protect`, inside
`with_r_thread`), the worker thread invariant, the R-longjmp leak that MXL300
guards. Cross-reference MXL300/MXL301 lint rules.

### PR 4 — Error and condition transport
Files: `miniextendr-api/src/error_value.rs`, `miniextendr-api/src/condition.rs`,
`miniextendr-api/src/panic_telemetry.rs`.
Scope: rustdoc the *why* of the tagged-SEXP transport — GC safety, destructor
safety, no R-longjmp leak. Document the three error-emission entry points:
`panic!()` (escape hatch), `error!()` macro (typed conditions),
`Result<_, RErrorAdapter>` (value-style). Cover `error_in_r` default vs
`no_error_in_r` / `unwrap_in_r` opt-outs.

### PR 5 — ExternalPtr + ALTREP
Files: `miniextendr-api/src/externalptr.rs`, `externalptr/*`,
`miniextendr-api/src/typed_external.rs`,
`miniextendr-api/src/altrep_traits.rs`,
`miniextendr-api/src/altrep_bridge.rs`.
Scope: rustdoc the `Box<Box<dyn Any>>` storage rationale, pointer-provenance
rules for `cached_ptr`, sidecar vs packed storage, `TYPE_NAME_CSTR` vs
`TYPE_ID_CSTR`. For ALTREP: the three guard modes
(`unsafe` / `rust_unwind` / `r_unwind`) and when each is correct;
field-based derive vs manual path tradeoff (link to ALTREP.md flowchart).

### PR 6 — Macro crate user-facing surface
Files: `miniextendr-macros/src/miniextendr_impl.rs` and the six class-system
generators in `miniextendr_impl/*.rs`, plus `dots.rs` / `typed_list.rs` /
`match_arg_*.rs` rustdoc.
Scope: each class-system generator gets a module-level `//!` saying *what
tradeoff this class system represents* in 4-8 lines, linking to
`docs/CLASS_SYSTEMS.md`. `typed_list!` vs hand-rolled `list!` validation.
`Dots` typed access vs raw access. (We do NOT rewrite the impl code — just
the rustdoc.)

### PR 1 — Top-level map (lands last, in coordination worktree)
Files: `miniextendr-api/src/lib.rs` (rustdoc only), `docs/API_CHOICE_MATRIX.md`
(new).
Scope: crate-level `//!` rewrite with the "I want to do X" decision table.
New `docs/API_CHOICE_MATRIX.md` aggregates all existing decision-tree docs.
Done by me in the coordination worktree after the other five land (or in
parallel with stubs that get filled in).

## Agent dispatch instructions (template per PR)

Each agent gets:
- `isolation: "worktree"` and a self-contained prompt with the worktree base-verify
- The relevant section of this plan
- A reminder: rustdoc tradeoff text is 2-5 lines per module + cross-ref, NOT a
  re-write of `docs/`. Link to `docs/`, don't duplicate it.
- A reminder: keep this scoped to **rustdoc and module-level `//!` docs**. No
  refactors. No "while we're here" cleanup.
- A reminder: open a draft PR, attribute properly via ai-attribution skill.
- A reminder: verify `git log origin/main..HEAD` is empty in their worktree
  before first commit.

## Out of scope (filed as issues if encountered)

- Renaming any APIs to make them clearer
- Adding new traits, types, fns, or macros
- Deduplicating between `docs/` and rustdoc
- Rewriting any of the existing `docs/*.md`
