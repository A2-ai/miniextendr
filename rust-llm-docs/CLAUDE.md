# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**rust-llm-docs** is a Python utility for converting Rust crate documentation from the rustdoc JSON format into human-readable markdown files. It's designed to make Rust API documentation more accessible to LLMs by providing structured markdown output.

The project provides two main conversion strategies:
- **`rustdoc_megadoc.py`**: Generates a single comprehensive markdown file covering every public item kind in rustdoc JSON format 57. Container items are nested under parents, trait impl headers live in the companion inventory, and unknown schema kinds fail instead of disappearing silently.
- **`rustdoc_split.py`**: Generates split documentation with individual files for each type (better for LLM consumption with separate files for easier chunking)

## Common Commands

### Generate JSON from a Rust crate
```bash
RUSTC_BOOTSTRAP=1 RUSTDOCFLAGS="-Z unstable-options --output-format json" cargo doc --no-deps
```
This generates `target/doc/<crate>/doc.json` with the rustdoc JSON output.

### Generate comprehensive markdown from JSON
```bash
python rustdoc_megadoc.py <input.json> [output.md]
```
- `input.json`: Path to the rustdoc JSON file
- `output.md`: Optional output file (defaults to `<input>.md` if not specified)

### Regenerate the committed miniextendr corpus
```bash
just llm-docs
just llm-docs-check
```

### Generate split markdown documentation
```bash
python rustdoc_split.py <input.json> [output_dir]
```
- `input.json`: Path to the rustdoc JSON file
- `output_dir`: Optional output directory (defaults to `docs/` if not specified)
- Creates:
  - `index.md`: Overview with links to all types
  - `structs/<name>.md`: Individual struct files
  - `enums/<name>.md`: Individual enum files

## Architecture

### Shared Type Formatting
Both scripts use a shared `format_type()` function that recursively converts rustdoc JSON type representations into readable Rust type syntax. It handles:
- **Primitive types**: bool, u16, i32, etc.
- **Generic parameters**: T, Self
- **Complex types**: Vec<T>, Result<T, E>, Option<T>
- **References**: &T, &mut T, &'a T
- **Trait bounds**: impl Trait, dyn Trait
- **Compound types**: tuples (A, B, C), arrays [T; N], slices [T]
- **Pointers**: *const T, *mut T
- **Function pointers**: fn(A, B) -> C
- **Qualified paths**: <T as Trait>::Item

### rustdoc_megadoc.py Structure
- `format_type()`: Core type formatter (shared logic)
- `format_function_signature()`: Formats complete function/method signatures with generics
- `generate_megadoc()`: Main orchestrator that collects and documents all public items
- Per-kind renderers cover modules/re-exports, structs/unions/enums, traits and aliases, functions/macros, constants/statics/type aliases, extern items, and primitives
- `document_method()`: Generates markdown for individual methods
- `validate_item_kinds()`: Fails fast when rustdoc adds an unaudited item kind

### rustdoc_split.py Structure
- Shared: `format_type()`, `format_function_signature()`, `document_method()`
- `generate_struct_doc()`: Full markdown for a single struct (table format for fields)
- `generate_enum_doc()`: Full markdown for a single enum
- `generate_index()`: Creates index.md with summary links and method signatures
- Helper functions for extracting metadata:
  - `get_struct_fields()`: List of (field_name, type_str) tuples
  - `get_struct_methods()`: List of (method_name, signature) tuples
  - `get_enum_variants()`: List of variant signatures
  - `get_enum_methods()`: List of (method_name, signature) tuples
- `generate_split_docs()`: Orchestrates creation of directory structure and all files

### rustdoc JSON Index
Both scripts rely on the `index` dictionary from the JSON, which maps item IDs (as strings) to item metadata:
- `"name"`: Item name
- `"docs"`: Documentation string
- `"visibility"`: "public" or "private"
- `"crate_id"`: 0 for crate items, non-zero for external dependencies
- `"inner"`: Type-specific data (struct, enum, function, etc.)
- `"id"`: Item identifier

The `root` field in the JSON indicates the crate root ID, and structs/enums have `"impls"` that reference impl blocks.

## Notes

- Both scripts filter to only document public items (`visibility == "public"`)
- By default, only items from the current crate are documented (`crate_id == 0`)
- The generated `docs/` directory is gitignored
- Function signatures are fully rendered including async/const/unsafe modifiers and generic constraints
- Type rendering is recursive and handles nested generic arguments properly
