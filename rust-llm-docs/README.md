# rust-llm-docs

Python tooling for generating structured markdown documentation from Rust crates,
optimised for LLM consumption.

## Origin

These scripts started life as a standalone tool written by the miniextendr maintainers
to solve a concrete problem: rustdoc HTML is not useful to LLMs, and docs.rs JSON is
huge and unstructured. The scripts convert the rustdoc JSON format (emitted by
`rustdoc --output-format json`) into compact, LLM-friendly markdown.

The tooling lives here (rather than as a separate repo) so it can be run directly
against the miniextendr workspace without any extra setup. The scripts have no
third-party Python dependencies for the *local-crate* path — only `requests` and
`zstandard` are needed when downloading from docs.rs.

## generated/ — miniextendr docs, built on demand

`just llm-docs` writes LLM-ready docs for every root workspace crate plus the
standalone `cargo-revendor` utility into `generated/`. The directory is
gitignored: the files change with every public-API change, so tracking them
made almost every open PR conflict. Generate them locally, then use them
directly in an LLM context when you need full public API coverage. Do not
hand-edit them. The standalone R-package and cross-package fixture workspaces
are test surfaces rather than framework API references, so they are not
documented.

| File | Contents |
|---|---|
| `<crate>.md` | Single-file public API digest: modules, re-exports, extern crates/types, structs, unions, enums, traits/trait aliases, functions, macros, constants, statics, type aliases, and primitives. Container items live under their parent; signatures include generics, bounds, ABI, variadics, and where clauses. One file each for `miniextendr-api`, `-macros`, `-engine`, `-bench`, `-lint`, `-cli` and `cargo-revendor`. |
| `<crate>-impl-inventory.md` | Every non-blanket, non-synthetic trait `impl` in the crate, grouped by trait, with fully-resolved `for`-type, generics, kind, and source span (the summary table still counts blanket/synthetic impls). Includes a per-trait "for-types sharing a source span" cluster: macro-expanded families collapse to one line, hand-rolled one-offs stand out. |
| `conversion-impl-inventory.md` | Same inventory restricted to the R↔Rust conversion traits (`TryFromSexp`, `IntoR`, `IntoRAs`, `Coerce`, `TryCoerce`, serde-native, ALTREP). The dedup-audit lens: re-run it after a conversion refactor to confirm the set of `for`-types is unchanged. |
| `conversion-manual-vs-macro.md` | Hand-rolled (unique-span) impls grouped by container shape, flagging shapes a macro already generates. The "which manual impls could a macro absorb?" lens. |

`miniextendr-api` is documented with its maintained `full` feature aggregate so
feature-gated integrations are visible; `miniextendr-bench` uses all features.
The others use default features. `miniextendr-macros` is a proc-macro crate, so
its rustdoc surface is intentionally thin.

### Regenerating

```bash
just llm-docs
```

Requires an installed nightly toolchain (`rustup toolchain install nightly`)
and Python 3. JSON output requires unstable rustdoc options; the recipe uses
`cargo +nightly` rather than enabling unstable behavior in the stable compiler.
Pass another installed nightly explicitly with `just llm-docs nightly-YYYY-MM-DD`
(or `just llm-docs-check nightly-YYYY-MM-DD`). Metadata and documentation use
the same selected toolchain.

The recipe runs `cargo +nightly doc --no-deps --document-private-items` for each crate,
using `miniextendr-api`'s maintained `full` feature aggregate and all benchmark
features, then renders markdown into `generated/`. The output is deterministic:
the same sources and rustdoc JSON render byte-identical files. `just llm-docs-check` runs the
renderer tests, then regenerates the corpus; the regeneration fails on any
rustdoc item kind the renderer does not cover.

## generated/type-matrix.* — the type matrix

`just type-matrix` lines up every type miniextendr defines or converts and
shows how each one behaves across the framework, as a color-coded table. It is
built from the same rustdoc JSON as the corpus (`miniextendr-api` with `full`,
plus `miniextendr-engine`) and from scans of the repository sources, and lands
in `generated/` with the rest (gitignored, never committed).

| File | Contents |
|---|---|
| `type-matrix.json` | The data: two tables of rows, each cell a status plus evidence (`file:line` where there is one), the legend with every rule, per-column counts, the commit. Deterministic for a given commit. |
| `type-matrix.html` | One self-contained page (embedded JSON, inline CSS and JS): two tabs, sticky header and first column, sortable columns, filters (crate / origin, module, kind, feature, text, rows with gaps only), per-column counts, and a side panel with a cell's evidence linked to GitHub at the commit. |
| `type-matrix.pdf`, `type-matrix-<page>.svg` | Both tables from `type-matrix.typ` (typst), one page per table, cells as colored glyph chips, legend and commit; evidence reduced to the counts. |

The two tables:

- **miniextendr types**: every public struct, enum, union, trait and type alias
  reachable from the crate roots (one row per item at its canonical path;
  prelude and other re-export paths are evidence).
- **converted types**: the std and third-party types the conversion traits are
  implemented for, folded to one row per shape (`Vec<i32>` and `Vec<f64>` are
  the row `Vec<T>`; `Vec<Option<T>>` and `HashMap<String, V>` stay their own).

The columns: capabilities (`arg`, `ret`, `opt/vec`, `coerce`, `serde`,
`altrep`, `class`: yes / partial / no / n/a, where `no` is neutral), `macro`
(the proc macros match the type's name), and coverage (`rustdoc`, `docs/`,
`rpkg`, `R tests`, `rust tests`, `cross-pkg`: yes / partial / gap / n/a, where
`gap` is red). The legend in the page and in the JSON states each rule; the
`R tests` column is a heuristic (an rpkg `#[miniextendr]` fn using the type is
named in a testthat file).

```bash
just type-matrix                      # nightly + typst
just type-matrix nightly-YYYY-MM-DD   # another installed nightly
```

To render from JSON you already have (from the repository root):

```bash
python3 rust-llm-docs/rustdoc_type_matrix.py --api target/doc/miniextendr_api.json \
  --engine target/doc/miniextendr_engine.json \
  --out rust-llm-docs/generated/type-matrix.json --html rust-llm-docs/generated/type-matrix.html
typst compile rust-llm-docs/type-matrix.typ rust-llm-docs/generated/type-matrix.pdf
typst compile rust-llm-docs/type-matrix.typ 'rust-llm-docs/generated/type-matrix-{p}.svg'
```

## Using the scripts for other crates

### Public crate from docs.rs

```bash
uv run rustdoc_public.py axum
```

Downloads and generates docs in `docs/`. Requires `requests` and `zstandard`:

```bash
pip install requests zstandard
```

The generated `API.md` is the complete item-kind view. The additional split
files are optimized indexes for structs, enums, traits, functions, and modules.

#### Re-exports

```bash
# Include re-exports (default)
uv run rustdoc_public.py axum

# Exclude re-exports
uv run rustdoc_public.py axum --no-rexports

# Custom version and output dir
uv run rustdoc_public.py axum --version 0.7.0 --output-dir ./my-docs
```

### Local crate

```bash
# 1. Generate the JSON
RUSTDOCFLAGS="-Z unstable-options --output-format json" cargo +nightly doc --no-deps

# 2. Single-file digest
python3 rustdoc_megadoc.py target/doc/<crate>.json output.md

# 3. Or split docs (one file per type)
python3 rustdoc_split.py target/doc/<crate>.json ./docs-out
```

## Scripts

| Script | Purpose |
|---|---|
| `rustdoc_public.py` | Download and document public crates from docs.rs |
| `rustdoc_megadoc.py` | Complete public-item markdown from rustdoc JSON; fails on unknown schema kinds |
| `rustdoc_split.py` | Split docs with one file per type |
| `rustdoc_impl_inventory.py` | Impl inventory grouped by trait + span |
| `rustdoc_manual_vs_macro.py` | Hand-rolled-vs-macro analysis |
| `rustdoc_common.py` | Shared type-formatting utilities |
| `generate-miniextendr-docs.sh` | Regenerate `generated/` for the miniextendr workspace |
| `rustdoc_type_matrix.py` | The type matrix data (`generated/type-matrix.json`) from rustdoc JSON and source scans |
| `rustdoc_type_matrix_html.py` | The type matrix as one self-contained HTML page |
| `type-matrix.typ` | The type matrix as typst tables (SVG / PDF) |
| `generate-type-matrix.sh` | Build the rustdoc JSON, then every type matrix output (`just type-matrix`) |
| `rustdoc-json.sh` | Shared rustdoc JSON flags and helpers, sourced by both generate scripts |
