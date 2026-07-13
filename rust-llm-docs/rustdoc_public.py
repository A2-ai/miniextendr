#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.8"
# dependencies = [
#   "requests>=2.31.0",
#   "zstandard>=0.22.0",
# ]
# ///
"""
Generate documentation from public crates on docs.rs.

Downloads rustdoc JSON from docs.rs and generates comprehensive,
hierarchical markdown documentation suitable for LLM consumption.

Usage:
    uv run rustdoc_public.py <crate> [--version VERSION] [--output-dir DIR] [--no-rexports]

Options:
    --version VERSION       Crate version (default: latest)
    --output-dir DIR        Output directory (default: docs)
    --no-rexports           Exclude re-exported items from other crates

Environment Variables:
    NO_REXPORTS             Set to 1, true, or yes to exclude re-exported items

Examples:
    uv run rustdoc_public.py axum
    uv run rustdoc_public.py axum --version 0.7.0 --output-dir ./docs
    uv run rustdoc_public.py axum --no-rexports
    NO_REXPORTS=1 uv run rustdoc_public.py axum
"""

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Any

import requests
import zstandard as zstd

from rustdoc_common import format_type, format_function_signature
from rustdoc_megadoc import generate_megadoc


def get_rexport_note(item: dict) -> str:
    """Get a re-export notation if item is from another crate."""
    source = item.get("_source", "")
    if source:
        return f" *(re-exported from `{source}`)*"
    crate_id = item.get("crate_id", 0)
    if crate_id != 0:
        return " *(re-exported)*"
    return ""


def download_crate_json(crate_name: str, version: str = "latest") -> dict:
    """
    Download rustdoc JSON from docs.rs for a public crate.

    Args:
        crate_name: Crate name (e.g., "axum")
        version: Version string (default: "latest")

    Returns:
        Parsed rustdoc JSON as dict
    """
    # Create cache directory
    cache_dir = Path(".cache/rustdoc")
    cache_dir.mkdir(parents=True, exist_ok=True)

    cache_file = cache_dir / f"{crate_name}-{version}.json"

    # Return from cache if available
    if cache_file.exists():
        print(f"Loading from cache: {cache_file}")
        with open(cache_file) as f:
            return json.load(f)

    # Download from docs.rs
    print(f"Downloading {crate_name} v{version} from docs.rs...")
    url = f"https://docs.rs/crate/{crate_name}/{version}/json"

    try:
        response = requests.get(url, timeout=30)
        response.raise_for_status()
    except requests.RequestException as e:
        print(f"Error downloading from {url}: {e}")
        sys.exit(1)

    # Handle zstandard compression
    try:
        # Use streaming decompression for zstandard
        dctx = zstd.ZstdDecompressor()
        # Decompress with a large max_output_size (1GB)
        decompressed = dctx.decompress(response.content, max_output_size=1024 * 1024 * 1024)
        parsed = json.loads(decompressed)
    except Exception as e:
        print(f"Error decompressing/parsing JSON: {e}")
        print(f"Response length: {len(response.content)}")
        sys.exit(1)

    # Cache the result
    with open(cache_file, "w") as f:
        json.dump(parsed, f)
    print(f"Cached to: {cache_file}")

    return parsed


def find_root_module(data: dict) -> dict:
    """Find the root module (crate root)."""
    index = data.get("index", {})
    root_id = data.get("root")
    root = index.get(str(root_id), {})
    return root


def get_module_items(module_id: int | str, index: dict) -> dict[str, list[dict]]:
    """
    Categorize items within a module by type.

    Returns dict with keys: modules, structs, enums, traits, functions, type_aliases, constants
    """
    module = index.get(str(module_id), {})
    module_info = module.get("inner", {}).get("module", {})
    item_ids = module_info.get("items", [])

    result = {
        "modules": [],
        "structs": [],
        "enums": [],
        "traits": [],
        "functions": [],
        "type_aliases": [],
        "constants": [],
    }

    for item_id in item_ids:
        item = index.get(str(item_id), {})
        if not item or item.get("visibility") != "public":
            continue

        inner = item.get("inner", {})
        if "module" in inner:
            result["modules"].append(item)
        elif "struct" in inner:
            result["structs"].append(item)
        elif "enum" in inner:
            result["enums"].append(item)
        elif "trait" in inner:
            result["traits"].append(item)
        elif "function" in inner:
            result["functions"].append(item)
        elif "type_alias" in inner:
            result["type_aliases"].append(item)
        elif "constant" in inner or "associated_const" in inner:
            result["constants"].append(item)

    return result


def get_module_path(item_id: int | str, index: dict) -> str:
    """
    Reconstruct the full module path (e.g., "axum::routing").

    For now, use the item name. A more sophisticated approach would
    traverse parents, but that requires additional index information.
    """
    item = index.get(str(item_id), {})
    return item.get("name", "unknown")


# Traits whose methods are noise in an API reference: auto-derives, markers,
# std conversions / formatting / hashing. Skipping these keeps *domain* trait
# impls -- most importantly candle's `Module` (forward) -- visible instead of
# being buried under Clone/Debug/From/etc.
SKIP_TRAIT_METHODS = {
    "Clone", "Copy", "Debug", "Default", "Display", "PartialEq", "Eq",
    "Hash", "PartialOrd", "Ord", "From", "Into", "TryFrom", "TryInto",
    "AsRef", "AsMut", "Borrow", "BorrowMut", "Deref", "DerefMut", "Drop",
    "ToOwned", "ToString", "Serialize", "Deserialize", "Send", "Sync",
    "Unpin", "RefUnwindSafe", "UnwindSafe", "Freeze", "Any", "Sized",
    "Pointer", "LowerHex", "UpperHex", "Binary", "Octal", "LowerExp", "UpperExp",
}


def collect_trait_methods(type_info: dict, index: dict) -> list:
    """Collect methods contributed by *trait* implementations of a struct/enum.

    Returns a list of ``(trait_name, [method_dict, ...])`` sorted by trait name,
    skipping the auto-derive / marker / std traits in ``SKIP_TRAIT_METHODS`` so
    that domain traits such as candle's ``Module::forward`` surface clearly.

    Note: trait-impl method items carry "default" visibility (they inherit it
    from the trait), so -- unlike inherent methods -- they are NOT filtered on
    ``visibility == "public"``.
    """
    groups = []
    for impl_id in type_info.get("impls", []):
        impl_item = index.get(str(impl_id), {})
        impl_info = impl_item.get("inner", {}).get("impl", {})
        trait = impl_info.get("trait")
        if not trait or impl_info.get("is_negative"):
            continue
        # Skip synthetic auto-trait impls (Send/Sync/Unpin/...) and blanket impls
        # (From/Into/Borrow/CloneToUninit/VZip/Pointable/... pulled in from deps);
        # both are pure noise here. Concrete impls like `Module` are kept.
        if impl_info.get("is_synthetic") or impl_info.get("blanket_impl") is not None:
            continue
        trait_name = trait.get("path", "").split("::")[-1]
        if not trait_name or trait_name in SKIP_TRAIT_METHODS:
            continue
        methods = [
            index.get(str(mid), {})
            for mid in impl_info.get("items", [])
            if "function" in index.get(str(mid), {}).get("inner", {})
        ]
        if methods:
            groups.append((trait_name, methods))
    groups.sort(key=lambda g: g[0])
    return groups


def render_trait_methods(type_info: dict, index: dict, style: str) -> list:
    """Markdown lines documenting trait-impl methods (empty list if none).

    style="list": markdown bullets, used in the inline module-file format.
    style="code": bold trait name + a ```rust``` block of signatures, used in
    the reference / individual-file formats.
    """
    groups = collect_trait_methods(type_info, index)
    if not groups:
        return []
    lines = ["**Trait implementations:**", ""]
    for trait_name, methods in groups:
        sigs = []
        for method in sorted(methods, key=lambda x: x.get("name", "")):
            mname = method.get("name", "?")
            func = method.get("inner", {}).get("function", {})
            sigs.append(format_function_signature(func, index, mname))
        if style == "code":
            lines.append(f"- `{trait_name}`")
            lines.append("```rust")
            lines.extend(sigs)
            lines.append("```")
        elif len(sigs) == 1:
            lines.append(f"- **{trait_name}**: `{sigs[0]}`")
        else:
            lines.append(f"- **{trait_name}**:")
            lines.extend(f"  - `{s}`" for s in sigs)
    lines.append("")
    return lines


def document_struct(struct: dict, index: dict) -> str:
    """Generate documentation for a struct (compact format)."""
    lines = []
    name = struct.get("name", "Unknown")
    docs = struct.get("docs", "")
    struct_info = struct.get("inner", {}).get("struct", {})
    rexport_note = get_rexport_note(struct)

    lines.append(f"### Struct: `{name}`{rexport_note}")
    lines.append("")

    if docs:
        lines.append(docs)
        lines.append("")

    # Fields table
    kind = struct_info.get("kind", {})
    if "plain" in kind:
        field_ids = kind["plain"].get("fields", [])
        if field_ids:
            lines.append("**Fields:**")
            lines.append("")
            lines.append("| Name | Type |")
            lines.append("|------|------|")
            for fid in field_ids:
                field = index.get(str(fid), {})
                if field.get("visibility") != "public":
                    continue
                field_name = field.get("name", "?")
                field_type = field.get("inner", {}).get("struct_field", {})
                type_str = format_type(field_type, index).replace("|", "\\|")
                lines.append(f"| `{field_name}` | `{type_str}` |")
            lines.append("")

    # Methods
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
        lines.append("**Methods:**")
        lines.append("")
        for method in sorted(methods, key=lambda x: x.get("name", "")):
            name = method.get("name", "?")
            method_docs = method.get("docs", "")
            func = method.get("inner", {}).get("function", {})
            sig = format_function_signature(func, index, name)
            lines.append(f"- `{sig}`")
            if method_docs:
                first_line = method_docs.split("\n")[0]
                lines.append(f"  {first_line}")
        lines.append("")

    lines.extend(render_trait_methods(struct_info, index, "list"))

    return "\n".join(lines)


def document_enum(enum: dict, index: dict) -> str:
    """Generate documentation for an enum (compact format)."""
    lines = []
    name = enum.get("name", "Unknown")
    docs = enum.get("docs", "")
    enum_info = enum.get("inner", {}).get("enum", {})
    rexport_note = get_rexport_note(enum)

    lines.append(f"### Enum: `{name}`{rexport_note}")
    lines.append("")

    if docs:
        lines.append(docs)
        lines.append("")

    # Variants
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
                first_line = variant_docs.split("\n")[0]
                lines.append(f"  {first_line}")
        lines.append("")

    # Methods
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
            name = method.get("name", "?")
            method_docs = method.get("docs", "")
            func = method.get("inner", {}).get("function", {})
            sig = format_function_signature(func, index, name)
            lines.append(f"- `{sig}`")
            if method_docs:
                first_line = method_docs.split("\n")[0]
                lines.append(f"  {first_line}")
        lines.append("")

    lines.extend(render_trait_methods(enum_info, index, "list"))

    return "\n".join(lines)


def document_trait(trait: dict, index: dict) -> str:
    """Generate documentation for a trait."""
    lines = []
    name = trait.get("name", "Unknown")
    docs = trait.get("docs", "")
    trait_info = trait.get("inner", {}).get("trait", {})
    rexport_note = get_rexport_note(trait)

    lines.append(f"### Trait: `{name}`{rexport_note}")
    lines.append("")

    if docs:
        lines.append(docs)
        lines.append("")

    # Separate required and provided methods
    method_ids = trait_info.get("items", [])
    required_methods = []
    provided_methods = []

    for method_id in method_ids:
        method = index.get(str(method_id), {})
        # Accept both "public" and "default" visibility (default means public for trait items)
        vis = method.get("visibility")
        if vis not in ("public", "default"):
            continue
        func = method.get("inner", {}).get("function", {})
        # Skip items without function data (e.g., associated types)
        if not func:
            continue
        # Distinguish by has_body: False = required, True = provided
        if func.get("has_body", False):
            provided_methods.append(method)
        else:
            required_methods.append(method)

    # Document required methods
    if required_methods:
        lines.append("**Required Methods:**")
        lines.append("")
        for method in required_methods:
            name = method.get("name", "?")
            method_docs = method.get("docs", "")
            func = method.get("inner", {}).get("function", {})
            sig = format_function_signature(func, index, name)
            lines.append(f"- `{sig}`")
            if method_docs:
                first_line = method_docs.split("\n")[0]
                lines.append(f"  {first_line}")
        lines.append("")

    # Document provided methods
    if provided_methods:
        lines.append("**Provided Methods:**")
        lines.append("")
        for method in provided_methods:
            name = method.get("name", "?")
            method_docs = method.get("docs", "")
            func = method.get("inner", {}).get("function", {})
            sig = format_function_signature(func, index, name)
            lines.append(f"- `{sig}`")
            if method_docs:
                first_line = method_docs.split("\n")[0]
                lines.append(f"  {first_line}")
        lines.append("")

    return "\n".join(lines)


def document_function(func: dict, index: dict) -> str:
    """Generate documentation for a function."""
    lines = []
    name = func.get("name", "Unknown")
    docs = func.get("docs", "")
    func_info = func.get("inner", {}).get("function", {})

    sig = format_function_signature(func_info, index, name)
    lines.append(f"#### `{sig}`")
    lines.append("")

    if docs:
        lines.append(docs)
        lines.append("")

    return "\n".join(lines)


def document_module(module_id: int | str, index: dict, crate_name: str) -> str:
    """Generate comprehensive documentation for a module."""
    module = index.get(str(module_id), {})
    name = module.get("name", "unknown")
    docs = module.get("docs", "")

    lines = []

    # Header
    module_path = get_module_path(module_id, index)
    lines.append(f"# {crate_name}::{module_path}")
    lines.append("")

    # Description
    if docs:
        lines.append(docs)
        lines.append("")

    # Get categorized items
    items = get_module_items(module_id, index)

    # Submodules (just links)
    if items["modules"]:
        lines.append("## Submodules")
        lines.append("")
        for mod in sorted(items["modules"], key=lambda x: x.get("name", "")):
            mod_name = mod.get("name", "?")
            lines.append(f"- `{mod_name}` - See separate documentation")
        lines.append("")

    # Type Aliases
    if items["type_aliases"]:
        lines.append("## Type Aliases")
        lines.append("")
        for ta in sorted(items["type_aliases"], key=lambda x: x.get("name", "")):
            ta_name = ta.get("name", "?")
            ta_docs = ta.get("docs", "")
            lines.append(f"### `{ta_name}`")
            lines.append("")
            if ta_docs:
                lines.append(ta_docs)
                lines.append("")
        lines.append("")

    # Constants
    if items["constants"]:
        lines.append("## Constants")
        lines.append("")
        for const in sorted(items["constants"], key=lambda x: x.get("name", "")):
            const_name = const.get("name", "?")
            const_docs = const.get("docs", "")
            lines.append(f"### `{const_name}`")
            lines.append("")
            if const_docs:
                lines.append(const_docs)
                lines.append("")
        lines.append("")

    # Functions
    if items["functions"]:
        lines.append("## Functions")
        lines.append("")
        for func in sorted(items["functions"], key=lambda x: x.get("name", "")):
            lines.append(document_function(func, index))
        lines.append("")

    # Traits
    if items["traits"]:
        lines.append("## Traits")
        lines.append("")
        for trait in sorted(items["traits"], key=lambda x: x.get("name", "")):
            lines.append(document_trait(trait, index))
        lines.append("")

    # Structs
    if items["structs"]:
        lines.append("## Types: Structs")
        lines.append("")
        for struct in sorted(items["structs"], key=lambda x: x.get("name", "")):
            lines.append(document_struct(struct, index))
        lines.append("")

    # Enums
    if items["enums"]:
        lines.append("## Types: Enums")
        lines.append("")
        for enum in sorted(items["enums"], key=lambda x: x.get("name", "")):
            lines.append(document_enum(enum, index))
        lines.append("")

    return "\n".join(lines)


def collect_rexports_from_root(data: dict, item_type: str) -> list[dict]:
    """Collect re-exported traits/structs/enums from root module use statements.

    Args:
        data: The rustdoc JSON data
        item_type: Type to collect ("trait", "struct", "enum", "function")

    Returns:
        List of re-exported items with _source field added
    """
    index = data.get("index", {})
    root_id = data.get("root")
    root = index.get(str(root_id), {})
    root_module = root.get("inner", {}).get("module", {})

    rexports = []

    for item_id in root_module.get("items", []):
        item = index.get(str(item_id), {})
        if not item:
            continue

        # Check if this is a use statement
        if "use" not in item.get("inner", {}):
            continue

        use_info = item.get("inner", {}).get("use", {})
        source = use_info.get("source", "")

        # Create a synthetic item for this re-export
        rexport_item = {
            "name": use_info.get("name"),
            "docs": "",  # Use statements don't have docs
            "visibility": "public",
            "crate_id": 1,  # Mark as external
            "_source": source,  # Store source for notation
            "inner": {},
        }

        # We can't validate the type since the external item data is not in the index
        # So we'll add all re-exports with the assumption they might be of the requested type
        # Users will see them and if they're not the right type, they can file an issue
        rexports.append(rexport_item)

    return rexports


def collect_all_traits(index: dict, include_rexports: bool = True, rexport_items: list[dict] = None) -> list[dict]:
    """Collect all public traits from the index.

    Args:
        index: The rustdoc index
        include_rexports: If True, include re-exported traits from other crates
        rexport_items: Pre-collected re-export items (from collect_rexports_from_root)
    """
    traits = []
    for item in index.values():
        if item.get("visibility") != "public":
            continue
        if "trait" not in item.get("inner", {}):
            continue

        # Filter by crate_id
        crate_id = item.get("crate_id", 0)
        if crate_id == 0:
            # Locally defined trait
            traits.append(item)
        elif include_rexports:
            # Re-exported trait from another crate
            traits.append(item)

    # Add re-exports from use statements (if available and enabled)
    if include_rexports and rexport_items:
        for rexport in rexport_items:
            if rexport.get("name") not in [t.get("name") for t in traits]:
                # Avoid duplicates
                traits.append(rexport)

    return sorted(traits, key=lambda x: x.get("name", ""))


def collect_all_types(index: dict, include_rexports: bool = True) -> tuple[list[dict], list[dict]]:
    """Collect all public structs and enums from the index.

    Args:
        index: The rustdoc index
        include_rexports: If True, include re-exported types from other crates
    """
    structs = []
    enums = []
    for item in index.values():
        if item.get("visibility") != "public":
            continue

        # Filter by crate_id
        crate_id = item.get("crate_id", 0)
        if crate_id != 0 and not include_rexports:
            continue

        inner = item.get("inner", {})
        if "struct" in inner:
            structs.append(item)
        elif "enum" in inner:
            enums.append(item)

    return sorted(structs, key=lambda x: x.get("name", "")), sorted(enums, key=lambda x: x.get("name", ""))


def collect_all_functions(index: dict, include_rexports: bool = True) -> list[dict]:
    """Collect all standalone public functions from the index (not methods).

    Args:
        index: The rustdoc index
        include_rexports: If True, include re-exported functions from other crates
    """
    # First, collect all item IDs that are methods (inside impl or trait blocks)
    method_ids = set()
    for item in index.values():
        if "impl" in item.get("inner", {}):
            impl_info = item.get("inner", {}).get("impl", {})
            for method_id in impl_info.get("items", []):
                method_ids.add(str(method_id))
        elif "trait" in item.get("inner", {}):
            trait_info = item.get("inner", {}).get("trait", {})
            for method_id in trait_info.get("items", []):
                method_ids.add(str(method_id))

    # Now collect only standalone functions (not methods)
    functions = []
    for item_id, item in index.items():
        if item_id in method_ids:
            continue
        if item.get("visibility") != "public":
            continue

        # Filter by crate_id
        crate_id = item.get("crate_id", 0)
        if crate_id != 0 and not include_rexports:
            continue

        if "function" in item.get("inner", {}):
            functions.append(item)

    return sorted(functions, key=lambda x: x.get("name", ""))


def generate_traits_reference(traits: list[dict], index: dict, crate_name: str) -> str:
    """Generate a comprehensive traits reference document."""
    lines = [f"# {crate_name}: Traits Reference"]
    lines.append("")
    lines.append(f"Comprehensive reference of all {len(traits)} public traits in {crate_name}.")
    lines.append("")

    if not traits:
        lines.append("No public traits found.")
        return "\n".join(lines)

    # Table of contents
    lines.append("## Index")
    lines.append("")
    for trait in traits:
        trait_name = trait.get("name", "?")
        lines.append(f"- [`{trait_name}`](#{trait_name.lower()})")
    lines.append("")

    # Detailed trait documentation
    lines.append("---")
    lines.append("")

    for trait in traits:
        trait_name = trait.get("name", "?")
        trait_docs = trait.get("docs", "")
        trait_info = trait.get("inner", {}).get("trait", {})

        lines.append(f"## {trait_name}")
        lines.append("")

        if trait_docs:
            lines.append(trait_docs)
            lines.append("")

        # Separate required and provided methods
        method_ids = trait_info.get("items", [])
        required_methods = []
        provided_methods = []

        for method_id in method_ids:
            method = index.get(str(method_id), {})
            # Accept both "public" and "default" visibility
            vis = method.get("visibility")
            if vis not in ("public", "default"):
                continue
            func = method.get("inner", {}).get("function", {})
            # Skip items without function data (e.g., associated types)
            if not func:
                continue
            # Distinguish by has_body: False = required, True = provided
            if func.get("has_body", False):
                provided_methods.append(method)
            else:
                required_methods.append(method)

        # Document required methods
        if required_methods:
            lines.append("### Required Methods")
            lines.append("")
            for method in required_methods:
                name = method.get("name", "?")
                method_docs = method.get("docs", "")
                func = method.get("inner", {}).get("function", {})
                sig = format_function_signature(func, index, name)
                lines.append(f"```rust")
                lines.append(sig)
                lines.append("```")
                if method_docs:
                    lines.append(method_docs)
                lines.append("")

        # Document provided methods
        if provided_methods:
            lines.append("### Provided Methods")
            lines.append("")
            for method in provided_methods:
                name = method.get("name", "?")
                method_docs = method.get("docs", "")
                func = method.get("inner", {}).get("function", {})
                sig = format_function_signature(func, index, name)
                lines.append(f"```rust")
                lines.append(sig)
                lines.append("```")
                if method_docs:
                    lines.append(method_docs)
                lines.append("")

        lines.append("---")
        lines.append("")

    return "\n".join(lines)


def generate_types_reference(structs: list[dict], enums: list[dict], index: dict, crate_name: str) -> str:
    """Generate a comprehensive types reference document with consolidated duplicates."""
    lines = [f"# {crate_name}: Types Reference"]
    lines.append("")
    lines.append(f"Comprehensive reference of all {len(structs)} structs and {len(enums)} enums in {crate_name}.")
    lines.append("")

    # Group structs by name
    structs_by_name = {}
    for struct in structs:
        name = struct.get("name", "?")
        if name not in structs_by_name:
            structs_by_name[name] = []
        structs_by_name[name].append(struct)

    # Group enums by name
    enums_by_name = {}
    for enum in enums:
        name = enum.get("name", "?")
        if name not in enums_by_name:
            enums_by_name[name] = []
        enums_by_name[name].append(enum)

    # Index
    lines.append("## Index")
    lines.append("")

    if structs_by_name:
        lines.append("### Structs")
        lines.append("")
        for name in sorted(structs_by_name.keys()):
            lines.append(f"- [`{name}`](#{name.lower()})")
        lines.append("")

    if enums_by_name:
        lines.append("### Enums")
        lines.append("")
        for name in sorted(enums_by_name.keys()):
            lines.append(f"- [`{name}`](#{name.lower()})")
        lines.append("")

    lines.append("---")
    lines.append("")

    # Structs
    if structs_by_name:
        lines.append("## Structs")
        lines.append("")

        for struct_name in sorted(structs_by_name.keys()):
            struct_list = structs_by_name[struct_name]

            lines.append(f"### {struct_name}")
            lines.append("")

            # If multiple structs with same name, show as variants
            if len(struct_list) > 1:
                lines.append(f"*Note: {len(struct_list)} variant(s) of `{struct_name}` exist.*")
                lines.append("")

                for i, struct in enumerate(struct_list, 1):
                    struct_docs = struct.get("docs", "")
                    if struct_docs:
                        lines.append(f"#### Variant {i}")
                        lines.append("")
                        lines.append(struct_docs)
                        lines.append("")

                lines.append("---")
                lines.append("")
            else:
                # Single struct
                struct = struct_list[0]
                struct_docs = struct.get("docs", "")
                struct_info = struct.get("inner", {}).get("struct", {})

                if struct_docs:
                    lines.append(struct_docs)
                    lines.append("")

                # Fields
                kind = struct_info.get("kind", {})
                if "plain" in kind:
                    field_ids = kind["plain"].get("fields", [])
                    if field_ids:
                        lines.append("**Fields:**")
                        lines.append("")
                        lines.append("| Name | Type |")
                        lines.append("|------|------|")
                        for fid in field_ids:
                            field = index.get(str(fid), {})
                            if field.get("visibility") != "public":
                                continue
                            field_name = field.get("name", "?")
                            field_type = field.get("inner", {}).get("struct_field", {})
                            type_str = format_type(field_type, index).replace("|", "\\|")
                            lines.append(f"| `{field_name}` | `{type_str}` |")
                        lines.append("")

                # Methods summary
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
                    lines.append("**Methods:**")
                    lines.append("")
                    for method in sorted(methods, key=lambda x: x.get("name", "")):
                        name = method.get("name", "?")
                        func = method.get("inner", {}).get("function", {})
                        sig = format_function_signature(func, index, name)
                        lines.append(f"```rust")
                        lines.append(sig)
                        lines.append("```")
                    lines.append("")

                lines.append("---")
                lines.append("")

    # Enums
    if enums_by_name:
        lines.append("## Enums")
        lines.append("")

        for enum_name in sorted(enums_by_name.keys()):
            enum_list = enums_by_name[enum_name]

            lines.append(f"### {enum_name}")
            lines.append("")

            # If multiple enums with same name, show as variants
            if len(enum_list) > 1:
                lines.append(f"*Note: {len(enum_list)} variant(s) of `{enum_name}` exist.*")
                lines.append("")

                for i, enum in enumerate(enum_list, 1):
                    enum_docs = enum.get("docs", "")
                    if enum_docs:
                        lines.append(f"#### Variant {i}")
                        lines.append("")
                        lines.append(enum_docs)
                        lines.append("")

                lines.append("---")
                lines.append("")
            else:
                # Single enum
                enum = enum_list[0]
                enum_docs = enum.get("docs", "")
                enum_info = enum.get("inner", {}).get("enum", {})

                if enum_docs:
                    lines.append(enum_docs)
                    lines.append("")

                # Variants
                variant_ids = enum_info.get("variants", [])
                if variant_ids:
                    lines.append("**Variants:**")
                    lines.append("")
                    for vid in variant_ids:
                        variant = index.get(str(vid), {})
                        variant_name = variant.get("name", "?")
                        variant_info = variant.get("inner", {}).get("variant", {})
                        kind = variant_info.get("kind", {})

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
                    lines.append("")

                # Methods summary
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
                        name = method.get("name", "?")
                        func = method.get("inner", {}).get("function", {})
                        sig = format_function_signature(func, index, name)
                        lines.append(f"```rust")
                        lines.append(sig)
                        lines.append("```")
                    lines.append("")

                lines.append("---")
                lines.append("")

    return "\n".join(lines)


def generate_structs_reference(structs: list[dict], index: dict, crate_name: str) -> str:
    """Generate a comprehensive structs-only reference document."""
    lines = [f"# {crate_name}: Structs Reference"]
    lines.append("")
    lines.append(f"Complete reference of all {len(structs)} public structs in {crate_name}.")
    lines.append("")

    if not structs:
        lines.append("No public structs found.")
        return "\n".join(lines)

    # Group structs by name to handle duplicates (e.g., multiple ResponseFuture)
    structs_by_name = {}
    for struct in structs:
        name = struct.get("name", "?")
        if name not in structs_by_name:
            structs_by_name[name] = []
        structs_by_name[name].append(struct)

    # Table of contents
    lines.append("## Index")
    lines.append("")
    for name in sorted(structs_by_name.keys()):
        lines.append(f"- [`{name}`](#{name.lower()})")
    lines.append("")
    lines.append("---")
    lines.append("")

    # Detailed struct documentation
    for name in sorted(structs_by_name.keys()):
        struct_list = structs_by_name[name]

        lines.append(f"## {name}")
        lines.append("")

        # If there are multiple structs with the same name, group them
        if len(struct_list) > 1:
            lines.append(f"*Note: {len(struct_list)} variant(s) of `{name}` exist.*")
            lines.append("")

            for i, struct in enumerate(struct_list, 1):
                struct_docs = struct.get("docs", "")

                if struct_docs:
                    lines.append(f"### Variant {i}")
                    lines.append("")
                    lines.append(struct_docs)
                    lines.append("")

            lines.append("---")
            lines.append("")
        else:
            # Single struct with this name
            struct = struct_list[0]
            struct_docs = struct.get("docs", "")
            struct_info = struct.get("inner", {}).get("struct", {})

            if struct_docs:
                lines.append(struct_docs)
                lines.append("")

            # Fields
            kind = struct_info.get("kind", {})
            if "plain" in kind:
                field_ids = kind["plain"].get("fields", [])
                if field_ids:
                    lines.append("### Fields")
                    lines.append("")
                    lines.append("| Name | Type |")
                    lines.append("|------|------|")
                    for fid in field_ids:
                        field = index.get(str(fid), {})
                        if field.get("visibility") != "public":
                            continue
                        field_name = field.get("name", "?")
                        field_type = field.get("inner", {}).get("struct_field", {})
                        type_str = format_type(field_type, index).replace("|", "\\|")
                        lines.append(f"| `{field_name}` | `{type_str}` |")
                    lines.append("")

            # Methods
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
                lines.append("### Methods")
                lines.append("")
                lines.append("```rust")
                for method in sorted(methods, key=lambda x: x.get("name", "")):
                    name = method.get("name", "?")
                    func = method.get("inner", {}).get("function", {})
                    sig = format_function_signature(func, index, name)
                    lines.append(sig)
                lines.append("```")
                lines.append("")

            lines.extend(render_trait_methods(struct_info, index, "code"))

            lines.append("---")
            lines.append("")

    return "\n".join(lines)


def generate_enums_reference(enums: list[dict], index: dict, crate_name: str) -> str:
    """Generate a comprehensive enums-only reference document."""
    lines = [f"# {crate_name}: Enums Reference"]
    lines.append("")
    lines.append(f"Complete reference of all {len(enums)} public enums in {crate_name}.")
    lines.append("")

    if not enums:
        lines.append("No public enums found.")
        return "\n".join(lines)

    # Group enums by name to handle duplicates
    enums_by_name = {}
    for enum in enums:
        name = enum.get("name", "?")
        if name not in enums_by_name:
            enums_by_name[name] = []
        enums_by_name[name].append(enum)

    # Table of contents
    lines.append("## Index")
    lines.append("")
    for name in sorted(enums_by_name.keys()):
        lines.append(f"- [`{name}`](#{name.lower()})")
    lines.append("")
    lines.append("---")
    lines.append("")

    # Detailed enum documentation
    for name in sorted(enums_by_name.keys()):
        enum_list = enums_by_name[name]

        lines.append(f"## {name}")
        lines.append("")

        # If there are multiple enums with the same name, group them
        if len(enum_list) > 1:
            lines.append(f"*Note: {len(enum_list)} variant(s) of `{name}` exist.*")
            lines.append("")

            for i, enum in enumerate(enum_list, 1):
                enum_docs = enum.get("docs", "")

                if enum_docs:
                    lines.append(f"### Variant {i}")
                    lines.append("")
                    lines.append(enum_docs)
                    lines.append("")

            lines.append("---")
            lines.append("")
        else:
            # Single enum with this name
            enum = enum_list[0]
            enum_docs = enum.get("docs", "")
            enum_info = enum.get("inner", {}).get("enum", {})

            if enum_docs:
                lines.append(enum_docs)
                lines.append("")

            # Variants
            variant_ids = enum_info.get("variants", [])
            if variant_ids:
                lines.append("### Variants")
                lines.append("")
                for vid in variant_ids:
                    variant = index.get(str(vid), {})
                    variant_name = variant.get("name", "?")
                    variant_docs = variant.get("docs", "")
                    variant_info = variant.get("inner", {}).get("variant", {})
                    kind = variant_info.get("kind", {})

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
                        first_line = variant_docs.split("\n")[0]
                        lines.append(f"  {first_line}")
                lines.append("")

            # Methods
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
                lines.append("### Methods")
                lines.append("")
                lines.append("```rust")
                for method in sorted(methods, key=lambda x: x.get("name", "")):
                    name = method.get("name", "?")
                    func = method.get("inner", {}).get("function", {})
                    sig = format_function_signature(func, index, name)
                    lines.append(sig)
                lines.append("```")
                lines.append("")

            lines.extend(render_trait_methods(enum_info, index, "code"))

            lines.append("---")
            lines.append("")

    return "\n".join(lines)


def generate_functions_reference(functions: list[dict], index: dict, crate_name: str) -> str:
    """Generate a comprehensive functions-only reference document."""
    lines = [f"# {crate_name}: Functions Reference"]
    lines.append("")
    lines.append(f"Complete reference of all {len(functions)} public functions in {crate_name}.")
    lines.append("")

    if not functions:
        lines.append("No public functions found.")
        return "\n".join(lines)

    # Table of contents
    lines.append("## Index")
    lines.append("")
    for func in functions:
        func_name = func.get("name", "?")
        lines.append(f"- [`{func_name}`](#{func_name.lower()})")
    lines.append("")
    lines.append("---")
    lines.append("")

    # Detailed function documentation
    for func in functions:
        func_name = func.get("name", "?")
        func_docs = func.get("docs", "")
        func_info = func.get("inner", {}).get("function", {})

        lines.append(f"## {func_name}")
        lines.append("")

        sig = format_function_signature(func_info, index, func_name)
        lines.append("```rust")
        lines.append(sig)
        lines.append("```")
        lines.append("")

        if func_docs:
            lines.append(func_docs)
            lines.append("")

        lines.append("---")
        lines.append("")

    return "\n".join(lines)


def generate_individual_struct_file(struct: dict, index: dict, crate_name: str) -> str:
    """Generate documentation for a single struct as an individual file."""
    lines = []
    struct_name = struct.get("name", "Unknown")
    struct_docs = struct.get("docs", "")
    struct_info = struct.get("inner", {}).get("struct", {})

    lines.append(f"# {crate_name}::{struct_name}")
    lines.append("")

    if struct_docs:
        lines.append(struct_docs)
        lines.append("")

    # Fields
    kind = struct_info.get("kind", {})
    if "plain" in kind:
        field_ids = kind["plain"].get("fields", [])
        if field_ids:
            lines.append("## Fields")
            lines.append("")
            lines.append("| Name | Type |")
            lines.append("|------|------|")
            for fid in field_ids:
                field = index.get(str(fid), {})
                if field.get("visibility") != "public":
                    continue
                field_name = field.get("name", "?")
                field_type = field.get("inner", {}).get("struct_field", {})
                type_str = format_type(field_type, index).replace("|", "\\|")
                lines.append(f"| `{field_name}` | `{type_str}` |")
            lines.append("")

    # Methods
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
        lines.append("```rust")
        for method in sorted(methods, key=lambda x: x.get("name", "")):
            name = method.get("name", "?")
            func = method.get("inner", {}).get("function", {})
            sig = format_function_signature(func, index, name)
            lines.append(sig)
        lines.append("```")
        lines.append("")

    lines.extend(render_trait_methods(struct_info, index, "code"))

    return "\n".join(lines)


def generate_individual_trait_file(trait: dict, index: dict, crate_name: str) -> str:
    """Generate documentation for a single trait as an individual file."""
    lines = []
    trait_name = trait.get("name", "Unknown")
    trait_docs = trait.get("docs", "")
    trait_info = trait.get("inner", {}).get("trait", {})
    rexport_note = get_rexport_note(trait)

    lines.append(f"# {crate_name}::{trait_name}{rexport_note}")
    lines.append("")

    # Show source for re-exported items
    source = trait.get("_source", "")
    if source:
        lines.append(f"> **Re-exported from:** `{source}`")
        lines.append("")

    if trait_docs:
        lines.append(trait_docs)
        lines.append("")

    # Separate required and provided methods
    method_ids = trait_info.get("items", [])
    required_methods = []
    provided_methods = []

    for method_id in method_ids:
        method = index.get(str(method_id), {})
        # Accept both "public" and "default" visibility (default means public for trait items)
        vis = method.get("visibility")
        if vis not in ("public", "default"):
            continue
        func = method.get("inner", {}).get("function", {})
        # Skip items without function data (e.g., associated types)
        if not func:
            continue
        # Distinguish by has_body: False = required, True = provided
        if func.get("has_body", False):
            provided_methods.append(method)
        else:
            required_methods.append(method)

    # Document required methods
    if required_methods:
        lines.append("## Required Methods")
        lines.append("")
        lines.append("```rust")
        for method in required_methods:
            name = method.get("name", "?")
            func = method.get("inner", {}).get("function", {})
            sig = format_function_signature(func, index, name)
            lines.append(sig)
        lines.append("```")
        lines.append("")

        for method in required_methods:
            name = method.get("name", "?")
            method_docs = method.get("docs", "")
            if method_docs:
                lines.append(f"**`{name}`**: {method_docs}")
                lines.append("")

    # Document provided methods
    if provided_methods:
        lines.append("## Provided Methods")
        lines.append("")
        lines.append("```rust")
        for method in provided_methods:
            name = method.get("name", "?")
            func = method.get("inner", {}).get("function", {})
            sig = format_function_signature(func, index, name)
            lines.append(sig)
        lines.append("```")
        lines.append("")

        for method in provided_methods:
            name = method.get("name", "?")
            method_docs = method.get("docs", "")
            if method_docs:
                lines.append(f"**`{name}`**: {method_docs}")
                lines.append("")

    return "\n".join(lines)


def generate_docs(data: dict, output_dir: Path, include_rexports: bool = True):
    """Generate complete documentation structure.

    Args:
        data: The rustdoc JSON data
        output_dir: Output directory path
        include_rexports: If True, include re-exported items from other crates
    """
    index = data.get("index", {})
    root = find_root_module(data)
    crate_name = root.get("name", "unknown")
    root_id = root.get("id")

    print(f"\nGenerating documentation for {crate_name}...")
    if include_rexports:
        print("  (including re-exported items from other crates)")
    else:
        print("  (local items only)")

    # Create output directories
    modules_dir = output_dir / "modules"
    structs_dir = output_dir / "structs"
    traits_dir = output_dir / "traits"
    modules_dir.mkdir(parents=True, exist_ok=True)
    structs_dir.mkdir(parents=True, exist_ok=True)
    traits_dir.mkdir(parents=True, exist_ok=True)

    # The hierarchical/split views below stay intentionally optimized for
    # browsing types and traits. API.md is the lossless public-item view.
    api_path = output_dir / "API.md"
    api_path.write_text(
        generate_megadoc(data, crate_only=True, include_reexports=include_rexports)
    )
    print(f"  Created: {api_path}")

    # Generate module documentation
    items = get_module_items(root_id, index)
    module_count = 0

    for module in items["modules"]:
        module_id = module.get("id")
        module_name = module.get("name", "unknown")

        content = document_module(module_id, index, crate_name)
        filepath = modules_dir / f"{module_name}.md"

        with open(filepath, "w") as f:
            f.write(content)

        print(f"  Created: {filepath}")
        module_count += 1

    # Collect re-exported items from root module use statements (if enabled)
    rexport_traits = collect_rexports_from_root(data, "trait") if include_rexports else []

    # Generate comprehensive reference documents
    traits = collect_all_traits(index, include_rexports=include_rexports, rexport_items=rexport_traits)
    structs, enums = collect_all_types(index, include_rexports=include_rexports)
    functions = collect_all_functions(index, include_rexports=include_rexports)

    if traits:
        traits_content = generate_traits_reference(traits, index, crate_name)
        traits_path = output_dir / "TRAITS.md"
        with open(traits_path, "w") as f:
            f.write(traits_content)
        print(f"  Created: {traits_path}")

    if structs:
        structs_content = generate_structs_reference(structs, index, crate_name)
        structs_path = output_dir / "STRUCTS.md"
        with open(structs_path, "w") as f:
            f.write(structs_content)
        print(f"  Created: {structs_path}")

    if enums:
        enums_content = generate_enums_reference(enums, index, crate_name)
        enums_path = output_dir / "ENUMS.md"
        with open(enums_path, "w") as f:
            f.write(enums_content)
        print(f"  Created: {enums_path}")

    if functions:
        functions_content = generate_functions_reference(functions, index, crate_name)
        functions_path = output_dir / "FUNCTIONS.md"
        with open(functions_path, "w") as f:
            f.write(functions_content)
        print(f"  Created: {functions_path}")

    # Generate individual struct files
    for struct in structs:
        struct_name = struct.get("name", "Unknown")
        content = generate_individual_struct_file(struct, index, crate_name)
        filepath = structs_dir / f"{struct_name}.md"
        with open(filepath, "w") as f:
            f.write(content)
        print(f"  Created: {filepath}")

    # Generate individual trait files
    for trait in traits:
        trait_name = trait.get("name", "Unknown")
        content = generate_individual_trait_file(trait, index, crate_name)
        filepath = traits_dir / f"{trait_name}.md"
        with open(filepath, "w") as f:
            f.write(content)
        print(f"  Created: {filepath}")

    # Generate README with root module content
    root_docs = root.get("docs", "")
    readme_lines = [f"# {crate_name}"]
    readme_lines.append("")

    if root_docs:
        readme_lines.append(root_docs)
        readme_lines.append("")

    # Add reference document links
    readme_lines.append("## Reference Documents")
    readme_lines.append("")
    readme_lines.append("Quick-reference guides:")
    readme_lines.append("")
    readme_lines.append("- **[Complete API](API.md)** - All public rustdoc item kinds")
    if traits:
        readme_lines.append(f"- **[Traits](TRAITS.md)** - {len(traits)} traits (consolidated reference)")
        readme_lines.append(f"- **[traits/](traits/)** - Individual trait files")
    if structs:
        readme_lines.append(f"- **[Structs](STRUCTS.md)** - {len(structs)} structs (consolidated reference)")
        readme_lines.append(f"- **[structs/](structs/)** - Individual struct files")
    if enums:
        readme_lines.append(f"- **[Enums](ENUMS.md)** - {len(enums)} enums (consolidated reference)")
    if functions:
        readme_lines.append(f"- **[Functions](FUNCTIONS.md)** - {len(functions)} functions")
    readme_lines.append("")

    # Add module index
    readme_lines.append("## Modules")
    readme_lines.append("")
    for module in sorted(items["modules"], key=lambda x: x.get("name", "")):
        mod_name = module.get("name", "?")
        mod_docs = module.get("docs", "")
        summary = mod_docs.split("\n")[0] if mod_docs else ""
        readme_lines.append(f"- **[{mod_name}](modules/{mod_name}.md)** - {summary}")
    readme_lines.append("")

    readme_path = output_dir / "README.md"
    with open(readme_path, "w") as f:
        f.write("\n".join(readme_lines))
    print(f"  Created: {readme_path}")

    print(f"\nDone! Generated documentation:")
    print(f"  - {module_count} modules (modules/*.md)")
    print(f"  - {len(traits)} traits (TRAITS.md + traits/*.md)")
    print(f"  - {len(structs)} structs (STRUCTS.md + structs/*.md)")
    print(f"  - {len(enums)} enums (ENUMS.md)")
    print(f"  - {len(functions)} functions (FUNCTIONS.md)")
    print(f"  - Complete API: API.md")
    print(f"  - Split references: TRAITS.md, STRUCTS.md, ENUMS.md, FUNCTIONS.md")


def main():
    parser = argparse.ArgumentParser(
        description="Generate markdown documentation from public Rust crates"
    )
    parser.add_argument("crate", help="Crate name (e.g., axum)")
    parser.add_argument(
        "--version", default="latest", help="Crate version (default: latest)"
    )
    parser.add_argument(
        "--output-dir", "-o", default="docs", help="Output directory (default: docs)"
    )
    parser.add_argument(
        "--no-rexports",
        action="store_true",
        help="Exclude re-exported items from other crates",
    )

    args = parser.parse_args()

    # Check environment variable (takes precedence over CLI arg)
    no_rexports_env = os.environ.get("NO_REXPORTS", "").lower() in ("1", "true", "yes")
    include_rexports = not (args.no_rexports or no_rexports_env)

    # Download JSON
    data = download_crate_json(args.crate, args.version)

    # Generate documentation
    output_dir = Path(args.output_dir)
    generate_docs(data, output_dir, include_rexports=include_rexports)

    print(f"\nDocumentation saved to: {output_dir}")


if __name__ == "__main__":
    main()
