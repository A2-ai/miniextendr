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

Requires a `rustc` with `RUSTC_BOOTSTRAP=1` support (stable is fine) and Python 3.
The recipe runs `cargo doc --no-deps --document-private-items` for each crate,
using `miniextendr-api`'s maintained `full` feature aggregate and all benchmark
features, then renders markdown into `generated/`. The output is deterministic:
the same sources render byte-identical files. `just llm-docs-check` runs the
renderer tests, then regenerates the corpus; the regeneration fails on any
rustdoc item kind the renderer does not cover.

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
RUSTC_BOOTSTRAP=1 RUSTDOCFLAGS="-Z unstable-options --output-format json" cargo doc --no-deps

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
