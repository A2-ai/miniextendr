# Quick Start Guide: rust-llm-docs

## Installation

```bash
# From the miniextendr repository
cd rust-llm-docs

# Install uv (one-time, see https://docs.astral.sh/uv/getting-started/)
# On macOS: brew install uv
# Or: pip install uv
```

## Generating Documentation

### For Public Crates (docs.rs)

Generate comprehensive documentation for any public Rust crate:

```bash
# Basic usage (generates to ./docs/)
uv run rustdoc_public.py axum

# Specify output directory
uv run rustdoc_public.py tokio --output-dir ./tokio-docs

# Generate specific crate version
uv run rustdoc_public.py hyper --version 1.0.0 --output-dir ./hyper-docs
```

**Output Structure:**
```
docs/
├── README.md              # Crate overview + indexes
├── API.md                 # Complete public-item digest
├── TRAITS.md / STRUCTS.md / ENUMS.md / FUNCTIONS.md
├── structs/ and traits/   # Individual type/trait files
└── modules/
    ├── routing.md         # Module docs + types + functions
    ├── extract.md
    ├── middleware.md
    └── ... (one file per module)
```

### For Local Crates (with cargo)

Generate a single comprehensive markdown file:

```bash
# First, generate rustdoc JSON from your crate
RUSTC_BOOTSTRAP=1 RUSTDOCFLAGS="-Z unstable-options --output-format json" cargo doc --no-deps

# Then generate markdown
python3 rustdoc_megadoc.py target/doc/<crate>.json
# Output: target/doc/<crate>.md
```

Generate split documentation with separate files:

```bash
python3 rustdoc_split.py target/doc/<crate>/doc.json --output-dir ./docs

# Output:
# docs/
# ├── index.md
# ├── structs/
# │   ├── MyStruct.md
# │   └── ...
# └── enums/
#     ├── MyEnum.md
#     └── ...
```

## Features

### rustdoc_public.py (Public Crates)
- ✅ Downloads from docs.rs with zstd decompression
- ✅ Caches downloaded JSON locally (instant subsequent runs)
- ✅ Comprehensive module-based organization
- ✅ Preserves all narrative documentation
- ✅ Inline type documentation (structs, enums, traits)
- ✅ Compact markdown format optimized for LLMs
- ✅ Complete public-item `API.md` plus browsable split references
- ✅ ~100KB documentation for major crates

### rustdoc_megadoc.py (Local Crates)
- ✅ Single comprehensive markdown file
- ✅ Every public rustdoc item kind, including modules and re-exports
- ✅ Full signatures with generics, bounds, ABI, variadics, and where clauses
- ✅ Fails when rustdoc introduces an unaudited item kind
- ✅ Easy to search and navigate

### rustdoc_split.py (Local Crates)
- ✅ Separate files per type (easier for LLM chunking)
- ✅ Index with links to all items
- ✅ Table-based field documentation

## Examples

### Generate axum documentation

```bash
uv run rustdoc_public.py axum --output-dir ./axum-docs

# Output:
# axum-docs/
# ├── README.md (13.8 KB) - Overview + tutorials + module links
# ├── modules/
# │   ├── routing.md - Router, MethodRouter, etc.
# │   ├── extract.md - Request extractors
# │   ├── handler.md - Handler trait
# │   ├── middleware.md - Middleware utilities
# │   ├── response.md - Response types
# │   └── ... (9 modules total)
# Total: ~100KB, ~2784 lines
```

### Generate tokio documentation

```bash
uv run rustdoc_public.py tokio --output-dir ./tokio-docs
# Generates async runtime documentation with all modules
```

### Generate documentation for a specific crate version

```bash
uv run rustdoc_public.py serde --version 1.0.0 --output-dir ./serde-1.0.0-docs
```

## Understanding the Output

### README.md
- Crate overview (description, high-level features)
- Examples from root module documentation
- Links to all available modules

### API.md
- Every public item kind in one searchable file
- Module-qualified headings and complete signatures
- Nested fields, variants, inherent associated items, and trait members

### Module Files (modules/*.md)
Each module file contains:
- **Module path** and description
- **Narrative documentation** (tutorials, guides, patterns)
- **Submodules** (listed with links)
- **Type aliases** and **constants**
- **Functions** (with signatures and docs)
- **Traits** (with method lists)
- **Structs** (with field tables and methods)
- **Enums** (with variants and methods)

### Field Tables
Structs use markdown tables for compact field documentation:
```
| Name | Type |
|------|------|
| `field1` | `String` |
| `field2` | `Option<Vec<T>>` |
```

## Tips for LLM Consumption

1. **Start with README.md** to understand the crate structure
2. **Use API.md** when completeness matters
3. **Navigate by modules** - each module file is self-contained
4. **Use module links** - jump directly to the module that interests you
5. **Check struct/enum tables** - quickly see available fields
6. **Review method lists** - understand the API surface at a glance

## Troubleshooting

### "uv: command not found"
Install uv first:
```bash
# macOS
brew install uv

# Or using pip
pip install uv

# Or using pipx
pipx install uv
```

### Network timeout downloading from docs.rs
- Check your internet connection
- Try again (the file will be cached after first download)
- Use `--version latest` explicitly if needed

### "Error decompressing JSON"
- Update uv to the latest version: `uv self update`
- Or reinstall dependencies: `uv run --refresh rustdoc_public.py <crate>`

## Configuration

### Cache Location
Downloaded JSON files are cached in `.cache/rustdoc/` by default. To clear:
```bash
trash .cache/rustdoc/
```

### Output Directory
Specify custom output location with `--output-dir`:
```bash
uv run rustdoc_public.py axum --output-dir /path/to/docs
```

### Refresh Dependencies
To update dependencies (useful if new versions are available):
```bash
uv run --refresh rustdoc_public.py axum
```

## API Reference

See the docstrings in each script for detailed function documentation:
```bash
python3 -c "from rustdoc_public import download_crate_json; help(download_crate_json)"
```

## More Information

- **CLAUDE.md** - Project architecture and design decisions
- **Source code** - Comprehensive docstrings on all functions
- **Examples** - Check generated documentation from axum, tokio, etc.

---

**Happy documenting!** 📚
