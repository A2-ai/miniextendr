# miniextendr-lint

Build-time static analysis. Runs from a downstream crate's `build.rs` (via the `build.rs` integration in `miniextendr-api`). Disable with `MINIEXTENDR_LINT=0`. See root `CLAUDE.md` for project rules.

## Layout
- `lib.rs` — entrypoint. `build_script()` prints what `build_directives()` returns: `rerun-if-changed` for every module file the walk found (on a parse error too, carried by `IndexError::files`, #1738), plus `src/` as a directory only when the crate root lives under it. Never watch the manifest dir of a root-`lib.rs` crate (every scaffolded package): it holds `target/` / `vendor/` / `.cargo/` and would rerun the script after every build. `Cargo.toml` is watched only while no crate root file can be found or read (#1745), since nothing else would rerun the lint then: scaffolded `configure` touches it on every run, so watching it on a healthy crate rebuilds the crate on every install (#1752).
- `crate_root.rs` — `CrateRoot::resolve`: the walk's starting file is `[lib] path` from `<manifest dir>/Cargo.toml`, else `src/lib.rs` when `src/` exists, else `lib.rs` (#1745). Line-based scanner, no TOML dependency; mirror of `miniextendr-macros/src/crate_config.rs` (`normalize_key` / `parse_string_value`). A `[lib] path` it cannot read (escapes, multi-line string, inline `lib = { path = ... }`) is an error naming `Cargo.toml`, never a silent fallback to the guess; a missing root file is an error naming the resolved path and where it came from.
- `crate_index.rs` — module-tree walker from the crate root: resolves `mod foo;` → file paths through `#[cfg(feature = "...")]` gates (on inline modules too) so feature-gated modules are visited only when active. File resolution follows rustc (`rustc_expand::module`, tracked as `ModDir { dir, relative }`): the crate root is a mod-rs file whatever its name, as are `mod.rs` and every `#[path]` target (their children sit beside them); any other `foo.rs` keeps its children in `foo/`; `#[path]` on `mod x;` is relative to the declaring file's own directory; an inline `mod a { mod b; }` looks in `a/` (under `foo/` in a `foo.rs`); `mod r#type;` is `type.rs`.
- `description.rs` — `Description`: the R package's `DESCRIPTION` (`Package:` plus Depends/Imports/Suggests/Enhances/LinkingTo, version constraints dropped), located at `<manifest dir>/../../DESCRIPTION`; none elsewhere, and the rules that need it skip. `build_directives` prints `rerun-if-changed` for it when present, so adding a dependency reruns the lint.
- `doc_links.rs` — the doc-attribute walk for roxygen-bound text: `markdown_lines` (leading prose, markdown tags; skips code tags, fences and lines after single-line tags, mirroring `MULTILINE_TAGS` / `CODE_TAGS` in `miniextendr-macros/src/roxygen.rs`) and `pkg_links` on top of it. #1700's note on R links dropped under `strip` can reuse `markdown_lines`.
- `misplaced_dots.rs` — appends the `name: &Dots` hint to a "failed to parse" error when the file has a Rust `...` parameter that is not last (#1737). Mirror of `miniextendr-macros/src/misplaced_dots.rs`; keep the detector and message in sync.
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
- ~~**MXL112**~~ — *removed*: lifetime params are now allowed on `#[miniextendr]` fns/impls. Lifetimes are erased at codegen and produce a single monomorphic symbol, making them FFI-safe with `#[no_mangle]`. Only type/const generic params are rejected by the macro (they require monomorphization).
- **MXL120** — vctrs constructor returns `Self`/named type, or impl has an instance-method receiver (`&self`, `self: &ExternalPtr<Self>`, etc.). Mirrors the proc-macro hard error in `miniextendr-macros`.
- **MXL203** — redundant `internal` + `noexport`.
- **MXL204** — a `pkg::topic` link (``[`Type::method`]``, `[Type::method]`, `[text][pkg::topic]`) in a `#[miniextendr]` item's doc tags, or in leading prose under `roxygen_prose_links = "keep"`, whose `pkg` is not the package itself, not in `DESCRIPTION` Depends/Imports/Suggests/Enhances/LinkingTo and not a base/recommended R package: roxygen2 reads it as a link into an uninstalled R package (#1742). Help names the `[text][crate::path]` rewrite; the rule never rewrites. Skips without a `DESCRIPTION`. Warning.
- **MXL300** — direct `Rf_error`/`Rf_errorcall` → replace with `panic!()` (framework converts to R error via tagged-SEXP transport).
- **MXL301** — `_unchecked` FFI outside known-safe contexts (ALTREP callbacks, `with_r_unwind_protect`, `with_r_thread`).
- **MXL302** — `into_sexp()`/`into_sexp_unchecked()` inside a `vec!`/`&[…]` literal (the use-after-free idiom, #307/#1025) → wrap each element in `__scope.protect_raw(…)` via a `ProtectScope`, or prefer the `IntoList`/`DataFrameRow` derives. Raw-text scanner: tracks `vec!`/`&[` bracket depth and flags `into_sexp(` while inside; the safe `vec![…].into_sexp()` whole-vec form is not flagged. Escape hatch: `// mxl::allow(MXL302)`.
- **MXL303** — two `#[miniextendr]` trait impls collapse to the same vtable symbol (`__VTABLE_{CRATE}_{TRAIT}_FOR_{TYPE}` since #1273; the crate prefix is constant within one crate, so the rule compares the crate-invariant `__VTABLE_{TRAIT}_FOR_{TYPE}` suffix) after the macro's case-folding (`trait.to_uppercase()` + `type.to_uppercase()`). Crate-wide; mirrors `miniextendr-macros/src/naming.rs::vtable_static_ident`. Distinct-only renderings count (Rust coherence forbids a verbatim duplicate), so a hit always means a case-fold collapse (`impl Counter for Foo` vs `impl counter for Foo`). Severity Error — the duplicate `#[no_mangle]` static would otherwise fail at link time with a message divorced from the source. Escape hatch: `// mxl::allow(MXL303)` on or above either impl.
- **MXL304** — call to, or `extern` declaration of, R's non-API `ATTRIB` / `SET_ATTRIB` (or their `_unchecked` forms); `R CMD check` on R >= 4.6 reports packages that reference them. miniextendr-api declares them only behind `nonapi`, so the rule mostly catches a crate re-declaring the symbol itself. Fix: `SexpExt::has_attributes()` / `is_identical()` / `get_attr()` / `set_attr()`, or `DUPLICATE_ATTRIB`. Raw-text scanner matching whole identifiers followed by `(`, so `ANY_ATTRIB` / `CLEAR_ATTRIB` / `DUPLICATE_ATTRIB` / `SHALLOW_DUPLICATE_ATTRIB` are not flagged. Warning. Escape hatch: `// mxl::allow(MXL304)`.

## Adding a rule
- New file under `rules/`, register in `rules.rs`, add code to `lint_code.rs`.
- Build cfg evaluation lives in `crate_index.rs` — don't re-implement.
- Shares the parser layer with `miniextendr-macros` (in-crate since `miniextendr-macros-core` retirement).
