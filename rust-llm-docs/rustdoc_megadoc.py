#!/usr/bin/env python3
"""
Generate a single comprehensive markdown file from rustdoc JSON.

Covers every public item kind in the crate: structs, enums, traits,
module-level functions, macros (macro_rules! and proc-macros), constants,
statics, and type aliases. Item headings are module-qualified so same-named
items in different modules stay distinguishable, and markdown headings inside
doc comments are demoted below the containing section's level.

Usage:
    python rustdoc_megadoc.py <input.json> [output.md]
"""

import json
import sys
from pathlib import Path

from rustdoc_common import (
    demote_headings,
    format_function_signature,
    format_type,
)

# Heading level used for individual item entries (### `module::Name`).
ITEM_LEVEL = 3
# Heading level used for members of an item (#### `method`).
MEMBER_LEVEL = 4


def qualified_name(item: dict, paths: dict) -> str:
    """Module-qualified item name (crate segment dropped), e.g. `list::List`.

    Falls back to the bare name for items absent from the `paths` table
    (e.g. impl members).
    """
    entry = paths.get(str(item.get("id", "")), {})
    segments = entry.get("path")
    if segments and len(segments) > 1:
        return "::".join(segments[1:])
    return item.get("name", "Unknown")


def append_docs(lines: list, docs: str, base_level: int) -> None:
    if docs:
        lines.append(demote_headings(docs, base_level))
        lines.append("")


def collect_member_ids(index: dict) -> set:
    """IDs of items that live inside an impl block or a trait definition.

    Used to tell module-level functions apart from methods and trait members.
    """
    members = set()
    for item in index.values():
        inner = item.get("inner", {})
        container = inner.get("impl") or inner.get("trait")
        if container:
            members.update(str(mid) for mid in container.get("items", []) or [])
    return members


def generate_megadoc(data: dict, crate_only: bool = True) -> str:
    """Generate a comprehensive markdown document from rustdoc JSON."""
    index = data.get("index", {})
    paths = data.get("paths", {})
    root_id = data.get("root")
    crate_version = data.get("crate_version", "unknown")

    # Get crate info
    root = index.get(str(root_id), {})
    crate_name = root.get("name", "unknown")
    crate_docs = root.get("docs", "")

    lines = []
    lines.append(f"# {crate_name} v{crate_version}")
    lines.append("")
    append_docs(lines, crate_docs, base_level=1)

    # Collect items by kind
    kinds = {
        "structs": [],
        "enums": [],
        "traits": [],
        "functions": [],
        "macros": [],
        "constants": [],
        "statics": [],
        "type_aliases": [],
    }
    member_ids = collect_member_ids(index)
    crate_id_filter = 0 if crate_only else None

    for key, item in index.items():
        if crate_id_filter is not None and item.get("crate_id") != crate_id_filter:
            continue
        if item.get("visibility") != "public":
            continue

        inner = item.get("inner", {})
        if "struct" in inner:
            kinds["structs"].append(item)
        elif "enum" in inner:
            kinds["enums"].append(item)
        elif "trait" in inner:
            kinds["traits"].append(item)
        elif "function" in inner:
            if key not in member_ids:
                kinds["functions"].append(item)
        elif "macro" in inner or "proc_macro" in inner:
            kinds["macros"].append(item)
        elif "constant" in inner:
            kinds["constants"].append(item)
        elif "static" in inner:
            kinds["statics"].append(item)
        elif "type_alias" in inner:
            kinds["type_aliases"].append(item)

    def emit_section(title: str, items: list, document) -> None:
        if not items:
            return
        lines.append("---")
        lines.append("")
        lines.append(f"## {title}")
        lines.append("")
        for item in sorted(items, key=lambda x: qualified_name(x, paths)):
            lines.extend(document(item, index, paths))

    emit_section("Structs", kinds["structs"], document_struct)
    emit_section("Enums", kinds["enums"], document_enum)
    emit_section("Traits", kinds["traits"], document_trait)
    emit_section("Functions", kinds["functions"], document_function)
    emit_section("Macros", kinds["macros"], document_macro)
    emit_section("Constants", kinds["constants"], document_constant)
    emit_section("Statics", kinds["statics"], document_static)
    emit_section("Type aliases", kinds["type_aliases"], document_type_alias)

    return "\n".join(lines)


def item_heading(item: dict, paths: dict) -> list:
    return [f"{'#' * ITEM_LEVEL} `{qualified_name(item, paths)}`", ""]


def inherent_methods(type_info: dict, index: dict) -> list:
    """Public methods from inherent (non-trait) impl blocks."""
    methods = []
    for impl_id in type_info.get("impls", []):
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
    return methods


def document_struct(struct: dict, index: dict, paths: dict) -> list:
    """Generate documentation for a struct."""
    lines = item_heading(struct, paths)
    append_docs(lines, struct.get("docs", ""), ITEM_LEVEL)
    struct_info = struct.get("inner", {}).get("struct", {})

    # Document fields
    kind = struct_info.get("kind", {})
    if "plain" in kind:
        field_ids = kind["plain"].get("fields", [])
        if field_ids:
            field_lines = []
            for fid in field_ids:
                field = index.get(str(fid), {})
                if field.get("visibility") != "public":
                    continue
                field_name = field.get("name", "?")
                field_docs = field.get("docs", "")
                field_type = field.get("inner", {}).get("struct_field", {})
                type_str = format_type(field_type, index)
                field_lines.append(f"- `{field_name}`: `{type_str}`")
                if field_docs:
                    field_lines.append(f"  - {field_docs.split(chr(10))[0]}")
            if field_lines:
                lines.append("**Fields:**")
                lines.append("")
                lines.extend(field_lines)
                lines.append("")

    methods = inherent_methods(struct_info, index)
    if methods:
        lines.append("**Methods:**")
        lines.append("")
        for method in sorted(methods, key=lambda x: x.get("name", "")):
            lines.extend(document_method(method, index))

    return lines


def document_enum(enum: dict, index: dict, paths: dict) -> list:
    """Generate documentation for an enum."""
    lines = item_heading(enum, paths)
    append_docs(lines, enum.get("docs", ""), ITEM_LEVEL)
    enum_info = enum.get("inner", {}).get("enum", {})

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

    methods = inherent_methods(enum_info, index)
    if methods:
        lines.append("**Methods:**")
        lines.append("")
        for method in sorted(methods, key=lambda x: x.get("name", "")):
            lines.extend(document_method(method, index))

    return lines


def document_trait(trait: dict, index: dict, paths: dict) -> list:
    """Generate documentation for a trait: docs plus required/provided members."""
    lines = item_heading(trait, paths)
    trait_info = trait.get("inner", {}).get("trait", {})
    qualifiers = []
    if trait_info.get("is_unsafe"):
        qualifiers.append("unsafe")
    if trait_info.get("is_auto"):
        qualifiers.append("auto")
    if qualifiers:
        lines.append(f"*({' '.join(qualifiers)} trait)*")
        lines.append("")
    append_docs(lines, trait.get("docs", ""), ITEM_LEVEL)

    required, provided, assoc = [], [], []
    for mid in trait_info.get("items", []) or []:
        member = index.get(str(mid), {})
        inner = member.get("inner", {})
        name = member.get("name", "?")
        if "function" in inner:
            func = inner["function"]
            sig = format_function_signature(func, index, name)
            docs = member.get("docs", "")
            first_line = f"  - {docs.split(chr(10))[0]}" if docs else None
            target = provided if func.get("has_body") else required
            target.append((sig, first_line))
        elif "assoc_type" in inner:
            assoc.append(f"- associated type `{name}`")
        elif "assoc_const" in inner:
            ty = format_type(inner["assoc_const"].get("type"), index)
            assoc.append(f"- associated const `{name}: {ty}`")

    for title, group in (("Required methods", required), ("Provided methods", provided)):
        if group:
            lines.append(f"**{title}:**")
            lines.append("")
            for sig, doc_line in group:
                lines.append(f"- `{sig}`")
                if doc_line:
                    lines.append(doc_line)
            lines.append("")
    if assoc:
        lines.append("**Associated items:**")
        lines.append("")
        lines.extend(assoc)
        lines.append("")

    return lines


def document_function(func_item: dict, index: dict, paths: dict) -> list:
    """Generate documentation for a module-level function."""
    lines = item_heading(func_item, paths)
    func = func_item.get("inner", {}).get("function", {})
    sig = format_function_signature(func, index, func_item.get("name", "?"))
    lines.append("```rust")
    lines.append(sig)
    lines.append("```")
    lines.append("")
    append_docs(lines, func_item.get("docs", ""), ITEM_LEVEL)
    return lines


def document_macro(macro: dict, index: dict, paths: dict) -> list:
    """Generate documentation for a macro_rules! or proc-macro."""
    name = macro.get("name", "?")
    inner = macro.get("inner", {})
    if "proc_macro" in inner:
        kind = inner["proc_macro"].get("kind", "bang")
        label = {"attr": f"#[{name}]", "derive": f"#[derive({name})]"}.get(
            kind, f"{name}!"
        )
    else:
        label = f"{name}!"
    lines = [f"{'#' * ITEM_LEVEL} `{label}`", ""]
    append_docs(lines, macro.get("docs", ""), ITEM_LEVEL)
    return lines


def _value_entry(item: dict, type_key: str, paths: dict, index: dict) -> list:
    """Shared renderer for constants and statics: name, type, docs."""
    info = item.get("inner", {}).get(type_key, {})
    ty = format_type(info.get("type"), index)
    lines = [f"{'#' * ITEM_LEVEL} `{qualified_name(item, paths)}: {ty}`", ""]
    append_docs(lines, item.get("docs", ""), ITEM_LEVEL)
    return lines


def document_constant(item: dict, index: dict, paths: dict) -> list:
    return _value_entry(item, "constant", paths, index)


def document_static(item: dict, index: dict, paths: dict) -> list:
    return _value_entry(item, "static", paths, index)


def document_type_alias(item: dict, index: dict, paths: dict) -> list:
    info = item.get("inner", {}).get("type_alias", {})
    ty = format_type(info.get("type"), index)
    lines = [
        f"{'#' * ITEM_LEVEL} `{qualified_name(item, paths)}`",
        "",
        f"`type {item.get('name', '?')} = {ty}`",
        "",
    ]
    append_docs(lines, item.get("docs", ""), ITEM_LEVEL)
    return lines


def document_method(method: dict, index: dict) -> list:
    """Generate documentation for a method."""
    lines = []
    name = method.get("name", "Unknown")
    docs = method.get("docs", "")
    func = method.get("inner", {}).get("function", {})

    sig = format_function_signature(func, index, name)
    lines.append(f"{'#' * MEMBER_LEVEL} `{name}`")
    lines.append("")
    lines.append("```rust")
    lines.append(sig)
    lines.append("```")
    lines.append("")
    append_docs(lines, docs, MEMBER_LEVEL)

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
