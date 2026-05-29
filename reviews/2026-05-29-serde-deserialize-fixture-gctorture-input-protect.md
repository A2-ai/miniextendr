# 2026-05-29 — serde deserialize gctorture fixtures fail full-suite-only (unprotected input)

## What was attempted

Run `just devtools-test` over the full `rpkg` suite on the
`feat-unified-dataframe-interface` branch before opening the PR.

## What went wrong

`[ FAIL 3 | WARN 7 | SKIP 19 | PASS 6168 ]`. The three FAILs:

- `test-borrowed-rows.R:25` — `gc_stress_borrowed_rows` →
  `STRING_ELT() can only be applied to a 'character vector', not a 'double'`
- `test-dataframe-deserialize.R:28` — `gc_stress_dataframe_to_vec` →
  `dataframe_to_vec failed: Message("missing field \`id\`")`
- `test-dataframe-deserialize.R:53` — `gc_stress_dataframe_to_vec_nested` (same family)

**All three pass when their file is run in isolation** (`just devtools-test
borrowed-rows`, `… dataframe-deserialize` → `PASS 10`). They fail only inside
the full single-process suite.

## Root cause

Same family as `reviews/2026-05-21-column-to-sexp-character-protect.md`
(unprotected SEXP reclaimed under `gctorture(TRUE)`, slot recycled to another
type), but a **different unprotected allocation** and it is **not in this PR's
code** — the serde serialize/deserialize core and these fixtures are
byte-identical to `origin/main`.

The fixtures build a data.frame *in Rust* and read it straight back:

```rust
let sexp = vec_to_dataframe(&original)?.into_sexp();   // freshly built, UNPROTECTED
let back = dataframe_to_vec(sexp)?;                    // first internal alloc → GC
```

`dataframe_de.rs:121-122` documents the load-bearing assumption:

> The input SEXP remains protected by R's argument frame for the duration of
> the call, so no extra `OwnedProtect` is needed inside this function.

That is true for the real path (`.Call(C_…, df)` — R's argument frame roots the
SEXP) but **false when the caller is Rust** and hands in a freshly-built,
unprotected data.frame. Under `gctorture(TRUE)` every allocation inside
`dataframe_to_vec` triggers a full GC; the unprotected input data.frame (and its
`names` STRSXP) is reclaimed, the SEXPREC slot is recycled — to a `REALSXP`
("double") given the full suite's free-list state — and the subsequent
name/element read fails type-checking. "missing field `id`" is the same
mechanism: the recycled `names` vector no longer reads back `"id"`.

### Why full-suite-only

In isolation the heap is near-pristine, so even under `gctorture` the reclaimed
slot is either not reused or reused as a compatible type within the short window
→ no observable error. After ~6000 prior tests the free-lists are full of
`REALSXP`/other SEXPRECs, so the recycle reliably lands on an incompatible type.
This is exactly the "other runners silently corrupt and pass" hazard in
CLAUDE.md's gctorture note, expressed as test-ordering sensitivity.

## Fix (applied — library-side)

Chose the robust, caller-agnostic fix: `OwnedProtect` the input SEXP at the top
of `dataframe_to_vec`, `with_dataframe_rows`, and `dataframe_to_vec_borrowed`
(`miniextendr-api/src/serde/dataframe_de.rs`). One guard line each; correct
regardless of whether the caller is `.Call` (R argument frame) or Rust (a
freshly-built, unprotected data.frame). This covers every fixture with the
`build-then-read` shape in one place — including the ones that happened *not* to
fail this run (`gc_stress_with_dataframe_rows`, `gc_stress_factor_labels`,
`gc_stress_iter_to_dataframe`, …) but share the latent exposure.

For `dataframe_to_vec_borrowed` the guard is scoped to the `else` block so it
drops *before* `Protected::new` re-protects `sexp` for the returned handle —
`OwnedProtect` uses `UNPROTECT(1)` (LIFO), so the order matters.

The stale doc comment on `with_dataframe_rows` ("input … protected by R's
argument frame … no extra `OwnedProtect` is needed") — the exact assumption that
was wrong for Rust callers — was corrected.

Picked up while preparing the unified-DataFrame-interface PR (the bug is
pre-existing on `main` and orthogonal, but `dataframe_de.rs` is already touched
by that PR and the rule is to fix failures you see). The PR's own additions
(`test-unified-dataframe.R`) also pass.

## Lessons

- **`build-in-Rust then read` fixtures must protect the built SEXP** — there is
  no R argument frame rooting it. A deserialize entry point that documents
  "input is protected by the caller" is only safe for the `.Call` caller.
- **Full-suite-only test failures are a GC-ordering smell**, not flakiness.
  Re-run the file in isolation: pass-in-isolation + fail-in-suite ⇒ an earlier
  test's allocation pressure is exposing an unprotected-SEXP bug in the later
  test's path (or its own, under self-enabled `gctorture`).
