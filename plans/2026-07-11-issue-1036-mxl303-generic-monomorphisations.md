# Plan: #1036 — MXL303 vs generic monomorphisations (audit, then thread generic args)

Date: 2026-07-11. Anchors verified against main @ 17f634d8.
Branch: `fix/1036-mxl303-generic-vtable`.

File-overlap note: `fix/1273-c-symbol-crate-prefix`
(`plans/2026-07-10-issue-1273-c-symbol-crate-prefix.md` item 9) also edits
`trait_tag_collision.rs` (crate-prefix suffix comparison). Whichever lands
second rebases; the changes compose (prefix-invariance × generic-hash
suffix).

## Verified anchors

- Lint: `vtable_symbol(trait_name, type_name)` at
  `miniextendr-lint/src/rules/trait_tag_collision.rs:111`, consumer `:48`,
  tests `:121-128`. Type name captured via `impl_type_name` at
  `crate_index.rs:560,569` into `AttributedTraitImpl` (`crate_index.rs:149`)
  — generic args dropped.
- Macro truth: `type_to_uppercase_name` at
  `miniextendr-macros/src/miniextendr_impl_trait.rs:447` — appends a 16-char
  FNV-1a hash for generic types (`MyType<u32>` → `MYTYPE_<hash16>`).

## Step 1 — audit (fully specified, decides between two pre-specified branches)

Write a lint-level test fixture (follow the existing MXL303 test layout in
`miniextendr-lint` — grep how `trait_tag_collision` rule tests build crate
indexes) containing:
(a) `#[miniextendr] impl MyTrait for MyType<u32>` + `... for MyType<f64>`
    (distinct monomorphisations — macro emits distinct symbols);
(b) `... for MyType<u32>` + `... for MyType` (bare) — distinct symbols;
(c) two impls for the SAME `MyType<u32>` via case-folded trait names
    (`Counter`/`counter` — the real MXL303 trigger) — colliding symbols.
Record what the CURRENT lint reports for each.

- **Branch A (lint mis-reports (a) or (b) as collisions, or misses (c) for
  generics)** → implement Step 2.
- **Branch B (current bucketing already yields correct verdicts for all
  three)** → no code fix: keep the new tests as pins, update the rule's doc
  comment + `lint_code.rs` MXL303 text to state generic handling explicitly,
  and the PR closes the issue as audited-no-op with the evidence.

## Step 2 — fix (Branch A only)

1. `crate_index.rs`: capture the full self-type tokens (including generic
   args) alongside the base ident in `AttributedTraitImpl` (new field, e.g.
   `type_tokens: String`), populated at the `:560/:569` capture sites.
2. `trait_tag_collision.rs`: replicate `type_to_uppercase_name`'s exact
   uppercase + FNV-1a-16 suffix logic in `vtable_symbol` for generic types.
   Do NOT import from miniextendr-macros if the lint crate doesn't already
   depend on that module path — copy the function with a comment pinning it
   to `miniextendr_impl_trait.rs:447` and add a cross-crate parity unit test
   (same input → same symbol string, hardcoding one known hash vector) so
   drift breaks tests.
3. Extend the rule tests (`:121-128` region) with the audit fixtures as
   permanent pins.

## Exact commands (worktree)

```bash
just worktree-sync                               # FIRST
cargo test -p miniextendr-lint 2>&1 > /tmp/1036-lint.log   # Read it
just test 2>&1 > /tmp/1036-rust.log
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
```

No R build needed unless a fixture lands in rpkg (it shouldn't — lint-level
tests only). No snapshots.

## Must NOT touch

- The macro's `type_to_uppercase_name` (it is the truth; the lint adapts).
- MXL303's non-generic verdicts (existing tests must stay green).
- Other MXL rules.

## Done criteria

- Audit results recorded in the PR body; the three fixture cases have pinned
  verdicts (no spurious, no missed); Branch A: lint symbol matches macro
  symbol for generic monomorphisations with a parity test; `Fixes #1036`.

## Escalation rule

If reality diverges from this plan — the lint's parse layer cannot see
generic args at all, the FNV replication cannot be made to match — **stop,
commit nothing further, and report back. Do not improvise.**
