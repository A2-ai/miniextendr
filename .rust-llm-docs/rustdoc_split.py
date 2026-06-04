#!/usr/bin/env python3
"""
Generate split markdown documentation from rustdoc JSON.

Creates separate files for easier LLM consumption:
- index.md: Crate overview with links to all types
- structs/<name>.md: Individual struct documentation
- enums/<name>.md: Individual enum documentation

Usage:
    python rustdoc_split.py <input.json> [output_dir]
"""

import json
import os
import sys
from pathlib import Path

from rustdoc_common import format_type, format_function_signature


def document_method(method: dict, index: dict) -> list[str]:
    """Generate documentation for a method."""
    lines = []
    name = method.get("name", "Unknown")
    docs = method.get("docs", "")
    func = method.get("inner", {}).get("function", {})

    sig = format_function_signature(func, index, name)
    lines.append(f"### `{name}`")
    lines.append("")
    lines.append("```rust")
    lines.append(sig)
    lines.append("```")
    lines.append("")

    if docs:
        lines.append(docs)
        lines.append("")

    return lines


def generate_struct_doc(struct: dict, index: dict, crate_name: str) -> str:
    """Generate documentation for a single struct."""
    lines = []
    name = struct.get("name", "Unknown")
    docs = struct.get("docs", "")
    struct_info = struct.get("inner", {}).get("struct", {})
    span = struct.get("span", {})
    filename = span.get("filename", "")

    lines.append(f"# {crate_name}::{name}")
    lines.append("")
    lines.append(f"**Type:** Struct")
    if filename:
        lines.append(f"**Defined in:** `{filename}`")
    lines.append("")

    if docs:
        lines.append("## Description")
        lines.append("")
        lines.append(docs)
        lines.append("")

    # Document fields
    kind = struct_info.get("kind", {})
    if "plain" in kind:
        field_ids = kind["plain"].get("fields", [])
        public_fields = []
        for fid in field_ids:
            field = index.get(str(fid), {})
            if field.get("visibility") == "public":
                public_fields.append(field)

        if public_fields:
            lines.append("## Fields")
            lines.append("")
            lines.append("| Field | Type | Description |")
            lines.append("|-------|------|-------------|")
            for field in public_fields:
                field_name = field.get("name", "?")
                field_docs = field.get("docs", "").split("\n")[0] if field.get("docs") else ""
                field_type = field.get("inner", {}).get("struct_field", {})
                type_str = format_type(field_type, index)
                # Escape pipes in type strings for markdown tables
                type_str = type_str.replace("|", "\\|")
                lines.append(f"| `{field_name}` | `{type_str}` | {field_docs} |")
            lines.append("")

    # Document impl methods
    impl_ids = struct_info.get("impls", [])
    methods = []

    for impl_id in impl_ids:
        impl_item = index.get(str(impl_id), {})
        impl_info = impl_item.get("inner", {}).get("impl", {})

        if impl_info.get("trait") is not None:
            continue

        for method_id in impl_info.get("items", []):
            method = index.get(str(method_id), {})
            if method.get("visibility") != "public":
                continue
            if "function" in method.get("inner", {}):
                methods.append(method)

    if methods:
        lines.append("## Methods")
        lines.append("")
        for method in sorted(methods, key=lambda x: x.get("name", "")):
            lines.extend(document_method(method, index))

    return "\n".join(lines)


def generate_enum_doc(enum: dict, index: dict, crate_name: str) -> str:
    """Generate documentation for a single enum."""
    lines = []
    name = enum.get("name", "Unknown")
    docs = enum.get("docs", "")
    enum_info = enum.get("inner", {}).get("enum", {})
    span = enum.get("span", {})
    filename = span.get("filename", "")

    lines.append(f"# {crate_name}::{name}")
    lines.append("")
    lines.append(f"**Type:** Enum")
    if filename:
        lines.append(f"**Defined in:** `{filename}`")
    lines.append("")

    if docs:
        lines.append("## Description")
        lines.append("")
        lines.append(docs)
        lines.append("")

    # Document variants
    variant_ids = enum_info.get("variants", [])
    if variant_ids:
        lines.append("## Variants")
        lines.append("")
        for vid in variant_ids:
            variant = index.get(str(vid), {})
            variant_name = variant.get("name", "?")
            variant_docs = variant.get("docs", "")
            variant_info = variant.get("inner", {}).get("variant", {})
            kind = variant_info.get("kind", {})

            # Format variant signature
            if "plain" in kind:
                lines.append(f"### `{variant_name}`")
            elif "tuple" in kind:
                field_ids = kind["tuple"]
                fields = []
                for fid in field_ids:
                    if fid is None:
                        fields.append("_")
                    else:
                        field = index.get(str(fid), {})
                        field_type = field.get("inner", {}).get("struct_field", {})
                        fields.append(format_type(field_type, index))
                lines.append(f"### `{variant_name}({', '.join(fields)})`")
            elif "struct" in kind:
                lines.append(f"### `{variant_name} {{ ... }}`")
            else:
                lines.append(f"### `{variant_name}`")

            lines.append("")
            if variant_docs:
                lines.append(variant_docs)
                lines.append("")

    # Document impl methods
    impl_ids = enum_info.get("impls", [])
    methods = []

    for impl_id in impl_ids:
        impl_item = index.get(str(impl_id), {})
        impl_info = impl_item.get("inner", {}).get("impl", {})

        if impl_info.get("trait") is not None:
            continue

        for method_id in impl_info.get("items", []):
            method = index.get(str(method_id), {})
            if method.get("visibility") != "public":
                continue
            if "function" in method.get("inner", {}):
                methods.append(method)

    if methods:
        lines.append("## Methods")
        lines.append("")
        for method in sorted(methods, key=lambda x: x.get("name", "")):
            lines.extend(document_method(method, index))

    return "\n".join(lines)


def get_struct_fields(struct: dict, index: dict) -> list[tuple[str, str]]:
    """Get list of (field_name, type_str) for a struct."""
    fields = []
    struct_info = struct.get("inner", {}).get("struct", {})
    kind = struct_info.get("kind", {})

    if "plain" in kind:
        field_ids = kind["plain"].get("fields", [])
        for fid in field_ids:
            field = index.get(str(fid), {})
            if field.get("visibility") != "public":
                continue
            field_name = field.get("name", "?")
            field_type = field.get("inner", {}).get("struct_field", {})
            type_str = format_type(field_type, index)
            fields.append((field_name, type_str))

    return fields


def get_struct_methods(struct: dict, index: dict) -> list[tuple[str, str]]:
    """Get list of (method_name, signature) for a struct."""
    methods = []
    struct_info = struct.get("inner", {}).get("struct", {})
    impl_ids = struct_info.get("impls", [])

    for impl_id in impl_ids:
        impl_item = index.get(str(impl_id), {})
        impl_info = impl_item.get("inner", {}).get("impl", {})

        if impl_info.get("trait") is not None:
            continue

        for method_id in impl_info.get("items", []):
            method = index.get(str(method_id), {})
            if method.get("visibility") != "public":
                continue
            if "function" in method.get("inner", {}):
                name = method.get("name", "Unknown")
                func = method.get("inner", {}).get("function", {})
                sig = format_function_signature(func, index, name)
                methods.append((name, sig))

    return sorted(methods, key=lambda x: x[0])


def get_enum_variants(enum: dict, index: dict) -> list[str]:
    """Get list of variant signatures for an enum."""
    variants = []
    enum_info = enum.get("inner", {}).get("enum", {})
    variant_ids = enum_info.get("variants", [])

    for vid in variant_ids:
        variant = index.get(str(vid), {})
        variant_name = variant.get("name", "?")
        variant_info = variant.get("inner", {}).get("variant", {})
        kind = variant_info.get("kind", {})

        if "plain" in kind:
            variants.append(variant_name)
        elif "tuple" in kind:
            field_ids = kind["tuple"]
            fields = []
            for fid in field_ids:
                if fid is None:
                    fields.append("_")
                else:
                    field = index.get(str(fid), {})
                    field_type = field.get("inner", {}).get("struct_field", {})
                    fields.append(format_type(field_type, index))
            variants.append(f"{variant_name}({', '.join(fields)})")
        elif "struct" in kind:
            variants.append(f"{variant_name} {{ ... }}")
        else:
            variants.append(variant_name)

    return variants


def get_enum_methods(enum: dict, index: dict) -> list[tuple[str, str]]:
    """Get list of (method_name, signature) for an enum."""
    methods = []
    enum_info = enum.get("inner", {}).get("enum", {})
    impl_ids = enum_info.get("impls", [])

    for impl_id in impl_ids:
        impl_item = index.get(str(impl_id), {})
        impl_info = impl_item.get("inner", {}).get("impl", {})

        if impl_info.get("trait") is not None:
            continue

        for method_id in impl_info.get("items", []):
            method = index.get(str(method_id), {})
            if method.get("visibility") != "public":
                continue
            if "function" in method.get("inner", {}):
                name = method.get("name", "Unknown")
                func = method.get("inner", {}).get("function", {})
                sig = format_function_signature(func, index, name)
                methods.append((name, sig))

    return sorted(methods, key=lambda x: x[0])


def generate_index(data: dict, structs: list, enums: list) -> str:
    """Generate the index.md file."""
    index = data.get("index", {})
    root_id = data.get("root")
    crate_version = data.get("crate_version", "unknown")

    root = index.get(str(root_id), {})
    crate_name = root.get("name", "unknown")
    crate_docs = root.get("docs", "")

    lines = []
    lines.append(f"# {crate_name} v{crate_version}")
    lines.append("")

    if crate_docs:
        lines.append("## Overview")
        lines.append("")
        lines.append(crate_docs)
        lines.append("")

    lines.append("## API Reference")
    lines.append("")

    if structs:
        lines.append("### Structs")
        lines.append("")
        for struct in sorted(structs, key=lambda x: x.get("name", "")):
            name = struct.get("name", "Unknown")
            docs = struct.get("docs", "")
            summary = docs.split("\n")[0] if docs else ""

            lines.append(f"#### [`{name}`](structs/{name}.md)")
            if summary:
                lines.append(f"{summary}")
            lines.append("")

            # Add fields
            fields = get_struct_fields(struct, index)
            if fields:
                lines.append("**Fields:**")
                for field_name, type_str in fields:
                    lines.append(f"- `{field_name}`: `{type_str}`")
                lines.append("")

            # Add methods
            methods = get_struct_methods(struct, index)
            if methods:
                lines.append("**Methods:**")
                for method_name, sig in methods:
                    lines.append(f"- `{sig}`")
                lines.append("")

    if enums:
        lines.append("### Enums")
        lines.append("")
        for enum in sorted(enums, key=lambda x: x.get("name", "")):
            name = enum.get("name", "Unknown")
            docs = enum.get("docs", "")
            summary = docs.split("\n")[0] if docs else ""

            lines.append(f"#### [`{name}`](enums/{name}.md)")
            if summary:
                lines.append(f"{summary}")
            lines.append("")

            # Add variants
            variants = get_enum_variants(enum, index)
            if variants:
                lines.append("**Variants:**")
                for variant in variants:
                    lines.append(f"- `{variant}`")
                lines.append("")

            # Add methods
            methods = get_enum_methods(enum, index)
            if methods:
                lines.append("**Methods:**")
                for method_name, sig in methods:
                    lines.append(f"- `{sig}`")
                lines.append("")

    return "\n".join(lines)


def generate_split_docs(data: dict, output_dir: Path):
    """Generate split documentation files."""
    index = data.get("index", {})
    root_id = data.get("root")

    root = index.get(str(root_id), {})
    crate_name = root.get("name", "unknown")

    # Create output directories
    structs_dir = output_dir / "structs"
    enums_dir = output_dir / "enums"
    structs_dir.mkdir(parents=True, exist_ok=True)
    enums_dir.mkdir(parents=True, exist_ok=True)

    # Collect items
    structs = []
    enums = []

    for key, item in index.items():
        if item.get("crate_id") != 0:
            continue
        if item.get("visibility") != "public":
            continue

        inner = item.get("inner", {})
        if "struct" in inner:
            structs.append(item)
        elif "enum" in inner:
            enums.append(item)

    # Generate struct files
    for struct in structs:
        name = struct.get("name", "Unknown")
        content = generate_struct_doc(struct, index, crate_name)
        filepath = structs_dir / f"{name}.md"
        with open(filepath, "w") as f:
            f.write(content)
        print(f"  Created: {filepath}")

    # Generate enum files
    for enum in enums:
        name = enum.get("name", "Unknown")
        content = generate_enum_doc(enum, index, crate_name)
        filepath = enums_dir / f"{name}.md"
        with open(filepath, "w") as f:
            f.write(content)
        print(f"  Created: {filepath}")

    # Generate index
    index_content = generate_index(data, structs, enums)
    index_path = output_dir / "index.md"
    with open(index_path, "w") as f:
        f.write(index_content)
    print(f"  Created: {index_path}")

    return len(structs), len(enums)


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)

    input_path = Path(sys.argv[1])
    output_dir = Path(sys.argv[2]) if len(sys.argv) > 2 else Path("docs")

    print(f"Loading {input_path}...")
    with open(input_path) as f:
        data = json.load(f)

    print(f"Generating split documentation in {output_dir}/...")
    num_structs, num_enums = generate_split_docs(data, output_dir)

    print(f"\nDone! Generated documentation for {num_structs} structs and {num_enums} enums.")


if __name__ == "__main__":
    main()
