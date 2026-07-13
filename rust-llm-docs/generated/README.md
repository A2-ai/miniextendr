# Generated doc corpus

Machine-generated, LLM-parseable digests of every miniextendr root-workspace
crate plus `cargo-revendor`, produced from rustdoc JSON. **Do not hand-edit** —
regenerate with `just llm-docs` from the repository root.

## Files

| File | What |
|---|---|
| `<crate>.md` | Single-file public API digest — modules, re-exports, extern crates/types, structs, unions, enums, traits/trait aliases, functions, macros, constants, statics, type aliases, and primitives. Container items live under their parent; signatures include generics, bounds, ABI, variadics, and where clauses. |
| `<crate>-impl-inventory.md` | Every non-blanket, non-synthetic trait `impl` in the crate, grouped by trait, with fully-resolved `for`-type, generics, kind, and source span (the summary table still counts blanket/synthetic impls). Includes a per-trait "for-types sharing a source span" cluster — macro-expanded families collapse to one line, hand-rolled one-offs stand out. |
| `conversion-impl-inventory.md` | Same inventory restricted to the R↔Rust conversion traits (`TryFromSexp`, `IntoR`, `IntoRAs`, `Coerce`, `TryCoerce`, serde-native, ALTREP). The dedup-audit lens. |
| `conversion-manual-vs-macro.md` | Hand-rolled (unique-span) impls grouped by container shape, flagging shapes a macro already generates. The "which manual impls could a macro absorb?" lens. |

`miniextendr-api` is documented with its maintained `full` feature aggregate so
feature-gated integrations are visible; `miniextendr-bench` uses all features.
The others use default features. `miniextendr-macros` is a proc-macro crate, so
its rustdoc surface is intentionally thin. Standalone R-package and
cross-package fixture workspaces are deliberately not presented as framework
APIs.

## Why this exists

The impl inventory is the evidence base for
`analysis/conversion-dedup-audit-2026-06-03.md`. Re-run it after any conversion
refactor to confirm the set of `for`-types is unchanged (same impls, fewer
macros).
