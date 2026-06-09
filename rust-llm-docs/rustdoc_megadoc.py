#!/usr/bin/env python3
"""
Generate a single comprehensive markdown file from rustdoc JSON.

Usage:
    python rustdoc_megadoc.py <input.json> [output.md]
"""

import json
import sys
from pathlib import Path

from rustdoc_common import format_type, format_function_signature


def get_item_parent_impl(item_id: int, index: dict) -> dict | None:
    """Find the impl block that contains this item."""
    for entry in index.values():
        inner = entry.get("inner", {})
        if "impl" in inner:
            impl_info = inner["impl"]
            if item_id in impl_info.get("items", []):
                return entry
    return None


def generate_megadoc(data: dict, crate_only: bool = True) -> str:
    """Generate a comprehensive markdown document from rustdoc JSON."""
    index = data.get("index", {})
    root_id = data.get("root")
    crate_version = data.get("crate_version", "unknown")

    # Get crate info
    root = index.get(str(root_id), {})
    crate_name = root.get("name", "unknown")
    crate_docs = root.get("docs", "")

    lines = []
    lines.append(f"# {crate_name} v{crate_version}")
    lines.append("")
    if crate_docs:
        lines.append(crate_docs)
        lines.append("")

    # Collect items by type
    structs = []
    enums = []
    functions = []  # Module-level functions

    crate_id_filter = 0 if crate_only else None

    for key, item in index.items():
        if crate_id_filter is not None and item.get("crate_id") != crate_id_filter:
            continue
        if item.get("visibility") != "public":
            continue

        inner = item.get("inner", {})
        if "struct" in inner:
            structs.append(item)
        elif "enum" in inner:
            enums.append(item)

    # Document structs
    if structs:
        lines.append("---")
        lines.append("")
        lines.append("## Structs")
        lines.append("")

        for struct in sorted(structs, key=lambda x: x.get("name", "")):
            lines.extend(document_struct(struct, index, crate_id_filter))

    # Document enums
    if enums:
        lines.append("---")
        lines.append("")
        lines.append("## Enums")
        lines.append("")

        for enum in sorted(enums, key=lambda x: x.get("name", "")):
            lines.extend(document_enum(enum, index, crate_id_filter))

    return "\n".join(lines)


def document_struct(struct: dict, index: dict, crate_id_filter: int | None) -> list[str]:
    """Generate documentation for a struct."""
    lines = []
    name = struct.get("name", "Unknown")
    docs = struct.get("docs", "")
    struct_info = struct.get("inner", {}).get("struct", {})
    struct_id = struct.get("id")

    lines.append(f"### `{name}`")
    lines.append("")
    if docs:
        lines.append(docs)
        lines.append("")

    # Document fields
    kind = struct_info.get("kind", {})
    if "plain" in kind:
        field_ids = kind["plain"].get("fields", [])
        if field_ids:
            lines.append("**Fields:**")
            lines.append("")
            for fid in field_ids:
                field = index.get(str(fid), {})
                if field.get("visibility") != "public":
                    continue
                field_name = field.get("name", "?")
                field_docs = field.get("docs", "")
                field_type = field.get("inner", {}).get("struct_field", {})
                type_str = format_type(field_type, index)
                lines.append(f"- `{field_name}`: `{type_str}`")
                if field_docs:
                    lines.append(f"  - {field_docs.split(chr(10))[0]}")
            lines.append("")

    # Document impl methods
    impl_ids = struct_info.get("impls", [])
    methods = []

    for impl_id in impl_ids:
        impl_item = index.get(str(impl_id), {})
        impl_info = impl_item.get("inner", {}).get("impl", {})

        # Skip trait impls (only document inherent methods)
        if impl_info.get("trait") is not None:
            continue

        for method_id in impl_info.get("items", []):
            method = index.get(str(method_id), {})
            if method.get("visibility") != "public":
                continue
            if "function" in method.get("inner", {}):
                methods.append(method)

    if methods:
        lines.append("**Methods:**")
        lines.append("")
        for method in sorted(methods, key=lambda x: x.get("name", "")):
            lines.extend(document_method(method, index))

    return lines


def document_enum(enum: dict, index: dict, crate_id_filter: int | None) -> list[str]:
    """Generate documentation for an enum."""
    lines = []
    name = enum.get("name", "Unknown")
    docs = enum.get("docs", "")
    enum_info = enum.get("inner", {}).get("enum", {})

    lines.append(f"### `{name}`")
    lines.append("")
    if docs:
        lines.append(docs)
        lines.append("")

    # Document variants
    variant_ids = enum_info.get("variants", [])
    if variant_ids:
        lines.append("**Variants:**")
        lines.append("")
        for vid in variant_ids:
            variant = index.get(str(vid), {})
            variant_name = variant.get("name", "?")
            variant_docs = variant.get("docs", "")
            variant_info = variant.get("inner", {}).get("variant", {})
            kind = variant_info.get("kind", {})

            # Format variant kind
            if "plain" in kind:
                lines.append(f"- `{variant_name}`")
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
                lines.append(f"- `{variant_name}({', '.join(fields)})`")
            elif "struct" in kind:
                lines.append(f"- `{variant_name} {{ ... }}`")

            if variant_docs:
                lines.append(f"  - {variant_docs.split(chr(10))[0]}")
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
        lines.append("**Methods:**")
        lines.append("")
        for method in sorted(methods, key=lambda x: x.get("name", "")):
            lines.extend(document_method(method, index))

    return lines


def document_method(method: dict, index: dict) -> list[str]:
    """Generate documentation for a method."""
    lines = []
    name = method.get("name", "Unknown")
    docs = method.get("docs", "")
    func = method.get("inner", {}).get("function", {})

    sig = format_function_signature(func, index, name)
    lines.append(f"#### `{name}`")
    lines.append("")
    lines.append(f"```rust")
    lines.append(sig)
    lines.append("```")
    lines.append("")

    if docs:
        lines.append(docs)
        lines.append("")

    return lines


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)

    input_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2]) if len(sys.argv) > 2 else input_path.with_suffix(".md")

    print(f"Loading {input_path}...")
    with open(input_path) as f:
        data = json.load(f)

    print("Generating documentation...")
    markdown = generate_megadoc(data)

    print(f"Writing {output_path}...")
    with open(output_path, "w") as f:
        f.write(markdown)

    print("Done!")


if __name__ == "__main__":
    main()
