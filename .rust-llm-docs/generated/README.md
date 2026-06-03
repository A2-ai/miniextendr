# Generated doc corpus

Machine-generated, LLM-parseable digests of every miniextendr workspace crate,
produced from rustdoc JSON. **Do not hand-edit** — regenerate with
`../generate-miniextendr-docs.sh`.

## Files

| File | What |
|---|---|
| `<crate>.md` | Single-file API digest — modules, structs (field tables), enums, traits, functions, methods with full signatures. |
| `<crate>-impl-inventory.md` | Every trait `impl` in the crate, grouped by trait, with fully-resolved `for`-type, generics, kind, and source span. Includes a per-trait "for-types sharing a source span" cluster — macro-expanded families collapse to one line, hand-rolled one-offs stand out. |
| `conversion-impl-inventory.md` | Same inventory restricted to the R↔Rust conversion traits (`TryFromSexp`, `IntoR`, `IntoRAs`, `Coerce`, `TryCoerce`, serde-native, ALTREP). The dedup-audit lens. |

`miniextendr-api` is documented with a broad feature set (everything in `full`
minus the `datafusion`/tokio stack, plus `jiff`) so feature-gated conversions
are visible. The others use default features. `miniextendr-macros` is a
proc-macro crate, so its rustdoc surface is intentionally thin.

## Why this exists

The impl inventory is the evidence base for
`analysis/conversion-dedup-audit-2026-06-03.md`. Re-run it after any conversion
refactor to confirm the set of `for`-types is unchanged (same impls, fewer
macros).
