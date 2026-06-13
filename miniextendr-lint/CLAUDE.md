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
- **MXL120** — vctrs constructor returns `Self`/named type, or impl has an instance-method receiver (`&self`, `self: &ExternalPtr<Self>`, etc.). Mirrors the proc-macro hard error in `miniextendr-macros`.
- **MXL203** — redundant `internal` + `noexport`.
- **MXL300** — direct `Rf_error`/`Rf_errorcall` → replace with `panic!()` (framework converts to R error via tagged-SEXP transport).
- **MXL301** — `_unchecked` FFI outside known-safe contexts (ALTREP callbacks, `with_r_unwind_protect`, `with_r_thread`).
- **MXL302** — `into_sexp()`/`into_sexp_unchecked()` inside a `vec!`/`&[…]` literal (the use-after-free idiom, #307/#1025) → wrap each element in `__scope.protect_raw(…)` via a `ProtectScope`, or prefer the `IntoList`/`DataFrameRow` derives. Raw-text scanner: tracks `vec!`/`&[` bracket depth and flags `into_sexp(` while inside; the safe `vec![…].into_sexp()` whole-vec form is not flagged. Escape hatch: `// mxl::allow(MXL302)`.
- **MXL303** — two `#[miniextendr]` trait impls collapse to the same vtable symbol (`__VTABLE_{TRAIT}_FOR_{TYPE}`) after the macro's case-folding (`trait.to_uppercase()` + `type.to_uppercase()`). Crate-wide; mirrors `miniextendr-macros/src/miniextendr_impl_trait/vtable.rs`. Distinct-only renderings count (Rust coherence forbids a verbatim duplicate), so a hit always means a case-fold collapse (`impl Counter for Foo` vs `impl counter for Foo`). Severity Error — the duplicate `#[no_mangle]` static would otherwise fail at link time with a message divorced from the source. Escape hatch: `// mxl::allow(MXL303)` on or above either impl.

## Adding a rule
- New file under `rules/`, register in `rules.rs`, add code to `lint_code.rs`.
- Build cfg evaluation lives in `crate_index.rs` — don't re-implement.
- Shares the parser layer with `miniextendr-macros` (in-crate since `miniextendr-macros-core` retirement).
