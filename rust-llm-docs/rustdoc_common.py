#!/usr/bin/env python3
"""
Shared utilities for rustdoc JSON processing.

Used by: rustdoc_megadoc.py, rustdoc_split.py, rustdoc_public.py
"""

from typing import Any


def format_const_arg(const: Any) -> str:
    """Render a const-generic argument from rustdoc JSON.

    rustdoc encodes const args as ``{"expr": "N", "value": ..., "is_literal": ...}``;
    naive ``str()`` leaks the Python dict repr into rendered types.
    """
    if isinstance(const, dict):
        if const.get("is_literal") and const.get("value") is not None:
            return str(const["value"])
        expr = const.get("expr")
        if expr is not None:
            return str(expr)
    return str(const)


def format_term(term: Any, index: dict) -> str:
    """Render a rustdoc ``Term`` (a type or constant)."""
    if not isinstance(term, dict):
        return str(term)
    if "type" in term:
        return format_type(term["type"], index)
    if "constant" in term:
        return format_const_arg(term["constant"])
    raise ValueError(f"unsupported rustdoc term shape: {sorted(term)}")


def format_generic_param(param: dict, index: dict) -> str:
    """Render one generic parameter definition."""
    name = param.get("name", "?")
    kind = param.get("kind", {})

    if "lifetime" in kind:
        outlives = kind["lifetime"].get("outlives", []) or []
        return f"{name}: {' + '.join(outlives)}" if outlives else name

    if "type" in kind:
        info = kind["type"]
        if info.get("is_synthetic", False):
            return ""
        bounds = [format_generic_bound(bound, index) for bound in info.get("bounds", [])]
        rendered = name
        if bounds:
            rendered += f": {' + '.join(bounds)}"
        if info.get("default") is not None:
            rendered += f" = {format_type(info['default'], index)}"
        return rendered

    if "const" in kind:
        info = kind["const"]
        rendered = f"const {name}: {format_type(info.get('type'), index)}"
        if info.get("default") is not None:
            rendered += f" = {info['default']}"
        return rendered

    return name


def format_generic_bound(bound: Any, index: dict) -> str:
    """Render a trait, lifetime, or precise-capturing bound."""
    if not isinstance(bound, dict):
        return str(bound)

    if "trait_bound" in bound:
        info = bound["trait_bound"]
        rendered = format_path(info.get("trait", {}), index)
        params = [
            format_generic_param(param, index)
            for param in info.get("generic_params", []) or []
        ]
        params = [param for param in params if param]
        if params:
            rendered = f"for<{', '.join(params)}> {rendered}"
        modifier = info.get("modifier", "none")
        if modifier == "maybe":
            rendered = f"?{rendered}"
        elif modifier == "maybe_const":
            rendered = f"~const {rendered}"
        return rendered

    if "outlives" in bound:
        return bound["outlives"]

    if "use" in bound:
        args = []
        for arg in bound["use"]:
            if "lifetime" in arg:
                args.append(arg["lifetime"])
            elif "param" in arg:
                args.append(arg["param"])
        return f"use<{', '.join(args)}>"

    raise ValueError(f"unsupported rustdoc generic bound: {sorted(bound)}")


def format_generic_args(args: Any, index: dict) -> str:
    """Render angle-bracketed, parenthesized, or return-type generic args."""
    if not args:
        return ""

    if "angle_bracketed" in args:
        info = args["angle_bracketed"]
        rendered = []
        for arg in info.get("args", []) or []:
            if "type" in arg:
                rendered.append(format_type(arg["type"], index))
            elif "lifetime" in arg:
                rendered.append(arg["lifetime"])
            elif "const" in arg:
                rendered.append(format_const_arg(arg["const"]))
            elif "infer" in arg:
                rendered.append("_")

        for constraint in info.get("constraints", []) or []:
            name = constraint.get("name", "?")
            name += format_generic_args(constraint.get("args"), index)
            binding = constraint.get("binding", {})
            if "equality" in binding:
                rendered.append(f"{name} = {format_term(binding['equality'], index)}")
            elif "constraint" in binding:
                bounds = [
                    format_generic_bound(bound, index)
                    for bound in binding["constraint"]
                ]
                rendered.append(f"{name}: {' + '.join(bounds)}")
        return f"<{', '.join(rendered)}>" if rendered else ""

    if "parenthesized" in args:
        info = args["parenthesized"]
        inputs = ", ".join(format_type(ty, index) for ty in info.get("inputs", []))
        rendered = f"({inputs})"
        if info.get("output") is not None:
            rendered += f" -> {format_type(info['output'], index)}"
        return rendered

    if "return_type_notation" in args:
        return "(..)"

    raise ValueError(f"unsupported rustdoc generic args: {sorted(args)}")


def format_path(path: Any, index: dict) -> str:
    """Render a rustdoc path with its generic arguments."""
    if not isinstance(path, dict):
        return str(path)
    name = path.get("path", path.get("name", "Unknown"))
    return f"{name}{format_generic_args(path.get('args'), index)}"


def format_where_predicate(predicate: dict, index: dict) -> str:
    """Render one ``where`` predicate."""
    if "bound_predicate" in predicate:
        info = predicate["bound_predicate"]
        params = [
            format_generic_param(param, index)
            for param in info.get("generic_params", []) or []
        ]
        params = [param for param in params if param]
        prefix = f"for<{', '.join(params)}> " if params else ""
        bounds = " + ".join(
            format_generic_bound(bound, index) for bound in info.get("bounds", [])
        )
        return f"{prefix}{format_type(info.get('type'), index)}: {bounds}"

    if "lifetime_predicate" in predicate:
        info = predicate["lifetime_predicate"]
        return f"{info.get('lifetime', '?')}: {' + '.join(info.get('outlives', []))}"

    if "eq_predicate" in predicate:
        info = predicate["eq_predicate"]
        return (
            f"{format_type(info.get('lhs'), index)} = "
            f"{format_term(info.get('rhs'), index)}"
        )

    raise ValueError(f"unsupported rustdoc where predicate: {sorted(predicate)}")


def format_generics(generics: dict, index: dict) -> tuple[str, str]:
    """Return rendered ``(<params>, where-clause)`` fragments."""
    params = [
        format_generic_param(param, index)
        for param in generics.get("params", []) or []
    ]
    params = [param for param in params if param]
    params_text = f"<{', '.join(params)}>" if params else ""

    predicates = [
        format_where_predicate(predicate, index)
        for predicate in generics.get("where_predicates", []) or []
    ]
    where_text = f" where {', '.join(predicates)}" if predicates else ""
    return params_text, where_text


def format_abi(abi: Any) -> str:
    """Render a rustdoc ABI value as an ``extern`` qualifier."""
    if not abi or abi == "Rust":
        return ""
    if isinstance(abi, str):
        return f'extern "{abi}" '
    if not isinstance(abi, dict) or not abi:
        return ""

    name, value = next(iter(abi.items()))
    spelling = {
        "C": "C",
        "Cdecl": "cdecl",
        "Stdcall": "stdcall",
        "Fastcall": "fastcall",
        "Aapcs": "aapcs",
        "Win64": "win64",
        "SysV64": "sysv64",
        "System": "system",
    }.get(name, value if name == "Other" and isinstance(value, str) else name)
    if isinstance(value, dict) and value.get("unwind"):
        spelling += "-unwind"
    return f'extern "{spelling}" '


def format_function_header(header: dict) -> str:
    """Render qualifiers that precede ``fn``."""
    parts = []
    if header.get("is_const"):
        parts.append("const ")
    if header.get("is_async"):
        parts.append("async ")
    if header.get("is_unsafe"):
        parts.append("unsafe ")
    parts.append(format_abi(header.get("abi")))
    return "".join(parts)


def demote_headings(docs: str, base_level: int) -> str:
    """Shift markdown headings inside a doc string below ``base_level``.

    Item docs are injected verbatim into a document whose own structure uses
    ``#``..``###`` headings; doc comments routinely contain ``# Safety`` /
    ``## Examples`` sections that would otherwise collide with (and outrank)
    the section that contains them. Headings inside fenced code blocks are
    left untouched.
    """
    lines = docs.split("\n")
    in_fence = False
    min_level = None
    for line in lines:
        stripped = line.lstrip()
        if stripped.startswith("```") or stripped.startswith("~~~"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        if line.startswith("#"):
            level = len(line) - len(line.lstrip("#"))
            if 1 <= level <= 6 and line[level : level + 1] in (" ", ""):
                min_level = level if min_level is None else min(min_level, level)
    if min_level is None or min_level > base_level:
        return docs
    delta = base_level + 1 - min_level
    out = []
    in_fence = False
    for line in lines:
        stripped = line.lstrip()
        if stripped.startswith("```") or stripped.startswith("~~~"):
            in_fence = not in_fence
            out.append(line)
            continue
        if not in_fence and line.startswith("#"):
            level = len(line) - len(line.lstrip("#"))
            if 1 <= level <= 6 and line[level : level + 1] in (" ", ""):
                out.append("#" * min(6, level + delta) + line[level:])
                continue
        out.append(line)
    return "\n".join(out)


def format_type(ty: Any, index: dict) -> str:
    """
    Recursively format a rustdoc JSON type into a readable string.

    Handles every ``Type`` variant in rustdoc JSON format 57.
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
        return format_path(ty["resolved_path"], index)

    # Borrowed reference (&T, &mut T, &'a T)
    if "borrowed_ref" in ty:
        br = ty["borrowed_ref"]
        lifetime = br.get("lifetime")
        is_mutable = br.get("is_mutable", False)
        inner_type = format_type(br.get("type"), index)

        parts = ["&"]
        if lifetime:
            # rustdoc JSON lifetimes already carry the leading tick
            parts.append(f"{lifetime} ")
        if is_mutable:
            parts.append("mut ")
        parts.append(inner_type)
        return "".join(parts)

    # impl Trait
    if "impl_trait" in ty:
        bounds = [format_generic_bound(bound, index) for bound in ty["impl_trait"]]
        return f"impl {' + '.join(bounds)}"

    # Tuple types ((A, B, C))
    if "tuple" in ty:
        elements = ty["tuple"]
        if not elements:
            return "()"
        formatted = [format_type(e, index) for e in elements]
        if len(formatted) == 1:
            return f"({formatted[0]},)"
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

    # Pattern type (unstable, e.g. `u32 is 1..`)
    if "pat" in ty:
        pat = ty["pat"]
        base = format_type(pat.get("type"), index)
        pattern = pat.get("__pat_unstable_do_not_use", "_")
        return f"{base} is {pattern}"

    # Inferred type (`_`)
    if "infer" in ty:
        return "_"

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
        args = format_generic_args(qp.get("args"), index)
        self_type = format_type(qp.get("self_type"), index)
        trait_info = qp.get("trait")
        if trait_info:
            trait_name = format_path(trait_info, index)
            return f"<{self_type} as {trait_name}>::{name}{args}"
        return f"{self_type}::{name}{args}"

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
            path = format_path(trait_info, index)
            params = [
                format_generic_param(param, index)
                for param in tb.get("generic_params", []) or []
            ]
            params = [param for param in params if param]
            if params:
                path = f"for<{', '.join(params)}> {path}"
            if path:
                bound_strs.append(path)
        lifetime = dt.get("lifetime")
        result = f"dyn {' + '.join(bound_strs)}" if bound_strs else "dyn _"
        if lifetime:
            # rustdoc JSON lifetimes already carry the leading tick
            result += f" + {lifetime}"
        return result

    # Function pointer (fn(A, B) -> C). Format 57 calls this
    # `function_pointer`; accept the older `fn_pointer` spelling as well.
    if "function_pointer" in ty or "fn_pointer" in ty:
        fp = ty.get("function_pointer", ty.get("fn_pointer"))
        sig = fp.get("sig", {})
        inputs = sig.get("inputs", [])
        output = sig.get("output")

        params = [
            format_generic_param(param, index)
            for param in fp.get("generic_params", []) or []
        ]
        params = [param for param in params if param]
        prefix = f"for<{', '.join(params)}> " if params else ""
        param_strs = [format_type(param_type, index) for _, param_type in inputs]
        if sig.get("is_c_variadic"):
            param_strs.append("...")

        result = f"{prefix}{format_function_header(fp.get('header', {}))}fn({', '.join(param_strs)})"
        if output:
            result += f" -> {format_type(output, index)}"
        return result

    raise ValueError(f"unsupported rustdoc type shape: {sorted(ty)}")


def format_function_signature(func: dict, index: dict, name: str = "") -> str:
    """Format a function's full signature."""
    sig = func.get("sig", {})
    header = func.get("header", {})
    generics = func.get("generics", {})

    params, where_clause = format_generics(generics, index)
    parts = [format_function_header(header), "fn"]
    if name:
        parts.append(f" {name}")
    parts.append(params)

    # Input parameters
    inputs = sig.get("inputs", [])
    param_strs = []
    for name, param_type in inputs:
        formatted_type = format_type(param_type, index)
        param_strs.append(f"{name}: {formatted_type}")
    if sig.get("is_c_variadic"):
        param_strs.append("...")

    parts.append(f"({', '.join(param_strs)})")

    # Return type
    output = sig.get("output")
    if output:
        parts.append(f" -> {format_type(output, index)}")

    parts.append(where_clause)

    return "".join(parts)


def is_public_crate_item(item: dict) -> bool:
    """Check if an item is public and from the current crate."""
    return item.get("visibility") == "public" and item.get("crate_id") == 0


def get_item_by_id(item_id: int | str, index: dict) -> dict | None:
    """Get an item from the index by ID."""
    return index.get(str(item_id))
