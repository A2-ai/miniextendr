#!/usr/bin/env python3
"""
Shared utilities for rustdoc JSON processing.

Used by: rustdoc_megadoc.py, rustdoc_split.py, rustdoc_public.py
"""

from typing import Any


def format_type(ty: Any, index: dict) -> str:
    """
    Recursively format a rustdoc JSON type into a readable string.

    Handles: primitive, generic, resolved_path, borrowed_ref, impl_trait,
    tuple, slice, array, raw_pointer, qualified_path, dyn_trait, fn_pointer
    """
    if ty is None:
        return "()"

    if isinstance(ty, str):
        return ty

    if not isinstance(ty, dict):
        return str(ty)

    # Primitive types (bool, u16, i32, etc.)
    if "primitive" in ty:
        return ty["primitive"]

    # Generic type parameters (Self, T, etc.)
    if "generic" in ty:
        return ty["generic"]

    # Resolved path (named types like String, Vec<T>, Result<T, E>)
    if "resolved_path" in ty:
        rp = ty["resolved_path"]
        name = rp.get("path", rp.get("name", "Unknown"))
        args = rp.get("args")
        if args and "angle_bracketed" in args:
            ab = args["angle_bracketed"]
            type_args = []
            for arg in ab.get("args", []):
                if "type" in arg:
                    type_args.append(format_type(arg["type"], index))
                elif "lifetime" in arg:
                    type_args.append(f"'{arg['lifetime']}")
                elif "const" in arg:
                    type_args.append(str(arg["const"]))
            if type_args:
                return f"{name}<{', '.join(type_args)}>"
        return name

    # Borrowed reference (&T, &mut T, &'a T)
    if "borrowed_ref" in ty:
        br = ty["borrowed_ref"]
        lifetime = br.get("lifetime")
        is_mutable = br.get("is_mutable", False)
        inner_type = format_type(br.get("type"), index)

        parts = ["&"]
        if lifetime:
            parts.append(f"'{lifetime} ")
        if is_mutable:
            parts.append("mut ")
        parts.append(inner_type)
        return "".join(parts)

    # impl Trait
    if "impl_trait" in ty:
        bounds = ty["impl_trait"]
        bound_strs = []
        for bound in bounds:
            if "trait_bound" in bound:
                tb = bound["trait_bound"]
                trait = tb.get("trait", {})
                trait_name = trait.get("path", "Unknown")
                args = trait.get("args")
                if args and "angle_bracketed" in args:
                    ab = args["angle_bracketed"]
                    type_args = []
                    for arg in ab.get("args", []):
                        if "type" in arg:
                            type_args.append(format_type(arg["type"], index))
                    if type_args:
                        trait_name = f"{trait_name}<{', '.join(type_args)}>"
                bound_strs.append(trait_name)
        return f"impl {' + '.join(bound_strs)}"

    # Tuple types ((A, B, C))
    if "tuple" in ty:
        elements = ty["tuple"]
        if not elements:
            return "()"
        formatted = [format_type(e, index) for e in elements]
        return f"({', '.join(formatted)})"

    # Slice types ([T])
    if "slice" in ty:
        inner = format_type(ty["slice"], index)
        return f"[{inner}]"

    # Array types ([T; N])
    if "array" in ty:
        arr = ty["array"]
        inner = format_type(arr.get("type"), index)
        length = arr.get("len", "N")
        return f"[{inner}; {length}]"

    # Raw pointer (*const T, *mut T)
    if "raw_pointer" in ty:
        rp = ty["raw_pointer"]
        is_mutable = rp.get("is_mutable", False)
        inner = format_type(rp.get("type"), index)
        prefix = "*mut" if is_mutable else "*const"
        return f"{prefix} {inner}"

    # Qualified path (associated types like <T as Trait>::Item)
    if "qualified_path" in ty:
        qp = ty["qualified_path"]
        name = qp.get("name", "Unknown")
        self_type = format_type(qp.get("self_type"), index)
        trait_info = qp.get("trait")
        if trait_info:
            trait_name = trait_info.get("path", "Trait")
            return f"<{self_type} as {trait_name}>::{name}"
        return f"{self_type}::{name}"

    # dyn Trait
    if "dyn_trait" in ty:
        dt = ty["dyn_trait"]
        traits = dt.get("traits", [])
        bound_strs = []
        for poly in traits:
            # rustdoc JSON (format_version 57) represents each bound as a
            # PolyTrait whose "trait" is a Path; older shapes nested it under a
            # "trait_bound" key. Handle both so the trait name isn't dropped
            # (the bug that rendered `Box<dyn CustomOp1>` as `Box<dyn >`).
            tb = poly.get("trait_bound", poly)
            trait_info = tb.get("trait", {})
            path = trait_info.get("path")
            if path:
                bound_strs.append(path)
        lifetime = dt.get("lifetime")
        result = f"dyn {' + '.join(bound_strs)}" if bound_strs else "dyn _"
        if lifetime:
            result += f" + '{lifetime}"
        return result

    # Function pointer (fn(A, B) -> C)
    if "fn_pointer" in ty:
        fp = ty["fn_pointer"]
        sig = fp.get("sig", {})
        inputs = sig.get("inputs", [])
        output = sig.get("output")

        param_strs = []
        for name, param_type in inputs:
            param_strs.append(format_type(param_type, index))

        result = f"fn({', '.join(param_strs)})"
        if output:
            result += f" -> {format_type(output, index)}"
        return result

    # Fallback for unknown types
    return str(ty)


def format_function_signature(func: dict, index: dict, name: str = "") -> str:
    """Format a function's full signature."""
    sig = func.get("sig", {})
    header = func.get("header", {})
    generics = func.get("generics", {})

    parts = []

    # Async/const/unsafe modifiers
    if header.get("is_async"):
        parts.append("async ")
    if header.get("is_const"):
        parts.append("const ")
    if header.get("is_unsafe"):
        parts.append("unsafe ")

    parts.append(name if name else "fn")

    # Generic parameters
    params = generics.get("params", [])
    type_params = []
    for param in params:
        if not param.get("kind", {}).get("type", {}).get("is_synthetic", False):
            type_params.append(param.get("name", "?"))
    if type_params:
        parts.append(f"<{', '.join(type_params)}>")

    # Input parameters
    inputs = sig.get("inputs", [])
    param_strs = []
    for name, param_type in inputs:
        formatted_type = format_type(param_type, index)
        param_strs.append(f"{name}: {formatted_type}")

    parts.append(f"({', '.join(param_strs)})")

    # Return type
    output = sig.get("output")
    if output:
        parts.append(f" -> {format_type(output, index)}")

    return "".join(parts)


def is_public_crate_item(item: dict) -> bool:
    """Check if an item is public and from the current crate."""
    return item.get("visibility") == "public" and item.get("crate_id") == 0


def get_item_by_id(item_id: int | str, index: dict) -> dict | None:
    """Get an item from the index by ID."""
    return index.get(str(item_id))
