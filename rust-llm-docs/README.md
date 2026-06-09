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

## generated/ — pre-built miniextendr docs

`generated/` contains a committed snapshot of LLM-ready docs for every workspace
crate. Use them directly in Claude Code or any other LLM context when you need
full API coverage:

| File | Contents |
|---|---|
| `miniextendr-api.md` | Full API digest for `miniextendr-api` (broad feature set) |
| `miniextendr-macros.md` | Proc-macro public API |
| `miniextendr-engine.md` | Codegen engine public API |
| `miniextendr-lint.md` | Lint rule public API |
| `miniextendr-cli.md` | CLI helper public API |
| `*-impl-inventory.md` | Every trait impl grouped by trait + span |
| `conversion-impl-inventory.md` | Conversion traits only — the dedup-audit view |
| `conversion-manual-vs-macro.md` | Hand-rolled impls a proc-macro could absorb |

### Regenerating

```bash
bash rust-llm-docs/generate-miniextendr-docs.sh
```

Requires a `rustc` with `RUSTC_BOOTSTRAP=1` support (stable is fine) and Python 3.
The script runs `cargo doc --no-deps` for each crate then calls the Python scripts
to render markdown into `generated/`. Commit the result alongside any macro/API
changes that affect the public surface.

## Using the scripts for other crates

### Public crate from docs.rs

```bash
uv run rustdoc_public.py axum
```

Downloads and generates docs in `docs/`. Requires `requests` and `zstandard`:

```bash
pip install requests zstandard
```

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
| `rustdoc_megadoc.py` | Single comprehensive markdown file from rustdoc JSON |
| `rustdoc_split.py` | Split docs with one file per type |
| `rustdoc_impl_inventory.py` | Impl inventory grouped by trait + span |
| `rustdoc_manual_vs_macro.py` | Hand-rolled-vs-macro analysis |
| `rustdoc_common.py` | Shared type-formatting utilities |
| `generate-miniextendr-docs.sh` | Regenerate `generated/` for the miniextendr workspace |
