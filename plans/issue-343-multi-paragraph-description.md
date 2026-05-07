# Issue #343 — extract paragraphs 3+ as @details

## Root cause
`miniextendr-macros/src/roxygen.rs:355` `implicit_description_from_attrs` extracts only the second paragraph and breaks at the first blank line within it. Paragraphs 3+ are dropped before the first `@tag`.

## Fix
Add `implicit_details_from_attrs(attrs: &[syn::Attribute]) -> Option<String>` that walks the doc lines, tracks paragraph index (0=title, 1=description, 2+=details), stops at the first `@tag`, and returns paragraphs 2+ joined with `\n\n` (so multiple `\details{}` paragraphs render correctly).

Wire into the auto-tag pass in `roxygen.rs` (~lines 85–166): after the existing `@description` insertion, also insert `@details {extracted}` if (a) no explicit `@details` exists and (b) the implicit details string is non-empty. Order in the generated wrapper: `@title` → `@description` → `@details` → `@param` …

## Tests (cargo unit tests in roxygen.rs `#[cfg(test)]`)
1. One paragraph: no `@description`, no `@details` injected.
2. Two paragraphs: `@description` injected (existing), no `@details`.
3. Three paragraphs: `@description` (para 2) + `@details` (para 3) injected.
4. Four paragraphs: `@details` joins para 3 + para 4 with `\n\n`.
5. Three paragraphs + `@param`: `@details` injected from para 3, `@param` preserved.
6. Explicit `@details` already present: no auto-injection (idempotency).

## Integration
Add a small fixture in `rpkg/src/rust/` exercising a 3-paragraph doc comment so a regenerated `man/*.Rd` shows the new `\details{}` block. Pick a thin function (e.g., `docs_demo_three_paras` returning a constant) — keep it minimal.

## Acceptance
- `cargo test -p miniextendr-macros` passes including new tests (run with `--features doc-lint` if needed for the `cfg_attr` gate).
- `just devtools-document` regenerates wrappers; the new fixture's Rd has a `\details{}` block.
- `git diff rpkg/man/` shows only the new fixture's Rd changing (no spurious churn on existing single/double-paragraph docs).
- `just check && just clippy && just devtools-test` clean.

## Out of scope
- Markdown-formatting tweaks inside `\details{}`.
- Migration of existing single/double-paragraph docs.
