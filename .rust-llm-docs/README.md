# rust-llm-docs

Generate structured markdown documentation from Rust crates for LLM consumption.

## Features

- **Download from docs.rs** - Automatically fetch rustdoc JSON for any public crate
- **Hierarchical structure** - Organized by modules with individual files for structs/traits
- **Re-export aware** - Includes re-exported items from dependencies with clear attribution
- **Comprehensive** - Documents all public items: traits, structs, enums, functions, modules
- **LLM-friendly** - Clean markdown output optimized for token efficiency

## Quick Start

### Generate docs for a public crate (from docs.rs)

```bash
uv run rustdoc_public.py axum
```

This downloads axum documentation and generates it in the `docs/` directory.

### Controlling re-exported items

By default, **re-exported items are included** (e.g., RequestExt from axum_core). You can customize this:

```bash
# Include re-exports (DEFAULT)
uv run rustdoc_public.py axum

# Exclude re-exports (only locally-defined items)
uv run rustdoc_public.py axum --no-rexports

# Via environment variable
NO_REXPORTS=1 uv run rustdoc_public.py axum
```

### Custom version and output directory

```bash
uv run rustdoc_public.py axum --version 0.7.0 --output-dir ./my-docs
```

## Example Output Structure

using axum as an example of what comes out: 

```
docs/
├── README.md                    # Crate overview
├── TRAITS.md                    # Consolidated traits reference
├── STRUCTS.md                   # Consolidated structs reference
├── ENUMS.md                     # Consolidated enums reference
├── FUNCTIONS.md                 # Module-level functions
├── modules/                     # One file per module
│   ├── routing.md
│   ├── extract.md
│   └── ...
├── structs/                     # Individual struct files
│   ├── Router.md
│   ├── State.md
│   └── ...
└── traits/                      # Individual trait files
    ├── Handler.md
    ├── RequestExt.md           # Re-exported from axum_core
    └── ...
```

## Re-exported Items

When a trait, struct, or enum is re-exported from another crate, it's clearly marked:

**In individual files (traits/RequestExt.md):**
```markdown
# axum::RequestExt *(re-exported from `axum_core::RequestExt`)*

> **Re-exported from:** `axum_core::RequestExt`

(trait documentation)
```

**In consolidated references (TRAITS.md):**
```markdown
## RequestExt *(re-exported from `axum_core::RequestExt`)*
```

Use `--no-rexports` if you only want items defined in the crate itself.

## Local Crate Documentation

To generate docs for a local Rust project:

```bash
RUSTC_BOOTSTRAP=1 RUSTDOCFLAGS="-Z unstable-options --output-format json" cargo doc --no-deps
```

Then use `rustdoc_megadoc.py` or `rustdoc_split.py` to process the generated JSON.

## Scripts

- **`rustdoc_public.py`** - Download and document public crates from docs.rs (with re-export support)
- **`rustdoc_megadoc.py`** - Generate single comprehensive markdown file from rustdoc JSON
- **`rustdoc_split.py`** - Generate split documentation with individual files per type
- **`rustdoc_common.py`** - Shared utilities used by all scripts

## Dependencies

```
requests>=2.31.0        # Download from docs.rs
zstandard>=0.22.0       # Decompress zstd-compressed JSON
```

Install with uv:
```bash
uv run rustdoc_public.py axum
```

Or install manually:
```bash
pip install requests zstandard
```

