# miniextendr-lint

Build-time static analysis. Runs from a downstream crate's `build.rs` (via the `build.rs` integration in `miniextendr-api`). Disable with `MINIEXTENDR_LINT=0`. See root `CLAUDE.md` for project rules.

## Layout
- `lib.rs` — entrypoint + module-tree walker with `cfg`/feature evaluation.
- `crate_index.rs` — resolves `mod foo;` → file paths through `#[cfg(feature = "...")]` gates so feature-gated modules are visited only when active.
- `rules.rs` (+ `rules/`) — one rule per file, registered into a dispatcher.
- `diagnostic.rs` — span+code emission.
- `lint_code.rs` — the `MXL*` code registry.
- `helpers.rs` — shared AST predicates.

## Rule registry
- **MXL008** — trait-impl class-system compat with inherent impl.
- **MXL009** — multiple impl blocks need distinct `label = "..."`.
- **MXL010** — duplicate labels.
- **MXL106** — non-`pub` fn would get `@export` → make `pub` or add `#[miniextendr(noexport)]`.
- **MXL110** — parameter name is an R reserved word.
- **MXL111** — `s4_*` method on `#[miniextendr(s4)]` impl (codegen auto-prefixes — yields `s4_s4_*`).
- **MXL112** — explicit lifetime param on `#[miniextendr]` fn/impl.
- **MXL203** — redundant `internal` + `noexport`.
- **MXL300** — direct `Rf_error`/`Rf_errorcall` → replace with `panic!()` (framework converts to R error via tagged-SEXP transport).
- **MXL301** — `_unchecked` FFI outside known-safe contexts (ALTREP callbacks, `with_r_unwind_protect`, `with_r_thread`).

## Adding a rule
- New file under `rules/`, register in `rules.rs`, add code to `lint_code.rs`.
- Build cfg evaluation lives in `crate_index.rs` — don't re-implement.
- Shares the parser layer with `miniextendr-macros` (in-crate since `miniextendr-macros-core` retirement).
