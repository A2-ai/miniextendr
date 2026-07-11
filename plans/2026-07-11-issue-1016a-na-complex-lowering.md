# Plan: #1016 (part 1 only) — `NA_complex_` atom in `r!()` lowering

Date: 2026-07-11. Anchors verified against main @ 17f634d8.
Branch: `feat/1016-na-complex-lowering`.

Scope: ONLY the NA_complex_ half of #1016. The token-interpolation half is a
design pass on the maintainer's decision list — do NOT implement or sketch
it. PR references #1016 (partial — do NOT `Fixes #1016`).

## Verified state (the issue is slightly stale — simpler than described)

- `SEXP::scalar_complex(Rcomplex)` ALREADY exists
  (`miniextendr-api/src/sexp.rs:208`; `_unchecked` at `:276`) — the issue's
  "no constructor available" premise is outdated.
- The gap is one explicit bail:
  `miniextendr-macros/src/r_macro/lowering.rs:350` —
  `"NA_complex_" => return None, // no constructor available`.
- Sibling NA atoms: `LowerAtom::NaInteger/NaReal/NaCharacter`
  (`lowering.rs:102-106`), classified at `:347-349`, emitted at
  `emit_atom` `:489-499`.

## The one semantic that must be right

R's `NA_complex_` has BOTH parts equal to `NA_real_` (R's tagged NaN), not
`f64::NAN`/`0.0` (the issue's sketch is wrong on this). Verify in
`background/r-svn/src/main/` (grep `NA_COMPLEX` / `R_NaComplex` in
arithmetic.c/complex.c) before writing the emission, and mirror EXACTLY how
the `LowerAtom::NaReal` arm (`lowering.rs:492`) obtains R's NA_real_ value —
use the same source for both parts.

## Work items

1. Add `LowerAtom::NaComplex` (doc comment mirroring `:105-106` style, citing
   `SEXP::scalar_complex`); classify at `:350` (replace the `return None`);
   emit in `emit_atom` building
   `Rcomplex { r: <NA_real_>, i: <NA_real_> }` → `SEXP::scalar_complex(...)`
   with the same checked/unchecked + scope-let pattern its NaReal sibling
   uses. Update the module-doc atom list at `lowering.rs:33` (it already
   names `NA_complex_` — make it true).
2. Macro-side tests: mirror how the other NA atoms are unit-tested in the
   r_macro test module (grep `NaReal` under `miniextendr-macros/src/r_macro/`)
   — assert `r!(f(NA_complex_))` lowers (does NOT fall back to the string
   path; the lowering tests have an is-lowered probe — reuse it).
3. rpkg fixture + testthat: a fixture calling `r!()` with `NA_complex_`
   (extend the existing r-macro fixture file — grep `r!` usage under
   `rpkg/src/rust/`), asserting in R:
   `identical(result_component, NA_complex_)` and `is.na(...)`. New export →
   ×2 install rule.
4. Docs: if `docs/`'s r!() page lists lowerable atoms, add `NA_complex_`.

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1
just worktree-sync                               # FIRST
cargo test -p miniextendr-macros 2>&1 > /tmp/1016-macros.log
just configure && just rcmdinstall && just force-document && just rcmdinstall
just devtools-test 2>&1 > /tmp/1016-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/1016-devtools.log  # devtools::test always exits 0
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
```

## Must NOT touch

- The clean-fallback guarantee and grammar-validation stages of `r!()`.
- Any interpolation syntax (`#ident`/`$ident`) — decision-list item.
- Other atoms' emission.

## Done criteria

- `NA_complex_` lowers without string-path fallback and round-trips
  `identical()`-equal to R's `NA_complex_`; suites + three clippy legs
  green; PR references #1016 with the interpolation half explicitly left
  open.

## Escalation rule

If reality diverges from this plan — the R-source check contradicts the
both-parts-NA_real_ claim, the emit pattern can't reuse the NaReal idiom —
**stop, commit nothing further, and report back. Do not improvise.**
