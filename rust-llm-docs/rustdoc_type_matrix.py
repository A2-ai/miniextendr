#!/usr/bin/env python3
"""
Type matrix: every type miniextendr defines or converts, with how it behaves
across the framework.

Reads rustdoc JSON (``miniextendr-api`` built with ``--features full``, and
``miniextendr-engine``) plus the repository sources, and writes one JSON file
with two tables:

- ``types``: miniextendr's own public structs, enums, unions, traits and type
  aliases, one row per item at its canonical path.
- ``converted``: the std and third-party types the conversion traits are
  implemented for, normalized to one row per shape (``Vec<T>``,
  ``Vec<Option<T>>``, ``HashMap<String, V>``, ``jiff::Timestamp``, ...).

Each cell is a status (``yes`` / ``partial`` / ``no`` / ``n/a`` for
capabilities, ``yes`` / ``partial`` / ``gap`` / ``n/a`` for quality) plus an
evidence list. The legend in the output states every rule.

Usage (from the repository root; ``--repo`` defaults to it):
    python3 rust-llm-docs/rustdoc_type_matrix.py --api target/doc/miniextendr_api.json \\
        --engine target/doc/miniextendr_engine.json \\
        --out rust-llm-docs/generated/type-matrix.json [--html rust-llm-docs/generated/type-matrix.html]

Python 3 standard library only. The output is deterministic: rows and
evidence are sorted, and the only date is the commit's.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tomllib
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from rustdoc_common import format_generics, format_type  # noqa: E402

REPO_URL = "https://github.com/A2-ai/miniextendr"
SCHEMA_VERSION = 1
EVIDENCE_CAP = 12

STD_CRATES = {"core", "alloc", "std"}
TYPE_KINDS = ("struct", "enum", "union", "trait", "type_alias")

# Conversion-trait families, by terminal name, checked against the trait's
# defining crate (`miniextendr_api`), never by name alone.
ARG_TRAITS = {"TryFromSexp", "TryFromRoot"}
RET_TRAITS = {"IntoR", "IntoRAs"}
COERCE_TRAITS = {"Coerce", "TryCoerce"}
SERDE_TRAITS = {"RSerializeNative", "RDeserializeNative"}
ELEMENT_TRAITS = {"RNativeType", "TryFromSexpElement", "IntoRVecElement", "IntoRNewtype"}
CLASS_TRAITS = {"TypedExternal"}

# Parameters the macros bind with no conversion trait (they read the name of
# the type, see the `macro` column for where). Used only when the type has no
# `TryFromSexp` / `TryFromRoot` impl of its own.
MACRO_PARAM_TYPES = {
    "LazyDots": "bound by `LazyDots::from_wrapper_arg` (R's `...` unforced)",
    "Dots": "`&Dots` binds R's `...`",
    "NArgs": "filled with `nargs()`, no R formal",
    "Call": "condition-call marker, no R formal",
    "CallerCall": "condition-call marker, no R formal",
    "Checked": "parameter marker, peeled to its inner type",
    "Unchecked": "parameter marker, peeled to its inner type",
    "NonNa": "parameter marker (`no_na` in the type)",
    "RngGuard": "`&RngGuard` under `#[miniextendr(rng)]`, no R formal",
    "ExternalPtrRef": "borrowed view of an external pointer argument",
    "ExternalPtrMut": "borrowed view of an external pointer argument",
    "Quoted": "unevaluated argument marker",
    "Quosure": "unevaluated argument marker",
}
# Returns the macros peel by name before converting the inner type.
MACRO_RETURN_TYPES = {
    "Invisible": "visibility marker, peeled before conversion",
    "Visible": "visibility marker, peeled before conversion",
}

PRIMITIVES = {
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64",
    "u128", "usize", "f32", "f64", "bool", "char", "str",
}
MAP_NAMES = {"HashMap", "BTreeMap", "IndexMap"}
STD_PRELUDE_NAMES = {
    "Vec", "Option", "Box", "String", "Result", "Rc", "Arc", "Cow", "HashMap",
    "BTreeMap", "HashSet", "BTreeSet", "VecDeque", "LinkedList", "BinaryHeap",
    "PathBuf", "Path", "OsString", "OsStr", "Cell", "RefCell", "Mutex",
    "RwLock", "Range", "RangeInclusive", "Duration", "SystemTime", "CString",
    "CStr", "PhantomData", "NonNull",
}
RUST_KEYWORDS = {
    "as", "break", "const", "continue", "else", "enum", "extern", "false",
    "fn", "for", "if", "in", "let", "loop", "match", "mod", "move", "mut",
    "pub", "ref", "return", "static", "struct", "trait", "true", "type",
    "unsafe", "use", "where", "while", "async", "await", "union", "macro_rules",
}
NOT_EXAMPLE_FENCES = {"text", "txt", "plain", "toml", "json", "sh", "bash", "console", "ascii", "yaml"}

STATUS_ORDER = ["yes", "partial", "no", "gap", "n/a"]


# region: columns and legend

TYPE_INFO_COLUMNS = [
    {"id": "crate", "label": "crate"},
    {"id": "module", "label": "module"},
    {"id": "kind", "label": "kind"},
    {"id": "generics", "label": "generics"},
    {"id": "feature", "label": "feature"},
]
CONVERTED_INFO_COLUMNS = [
    {"id": "origin", "label": "origin"},
    {"id": "kind", "label": "kind"},
    {"id": "bounds", "label": "bounds"},
    {"id": "feature", "label": "feature"},
]

CELL_COLUMNS = [
    {"id": "arg", "label": "arg", "group": "capability"},
    {"id": "ret", "label": "ret", "group": "capability"},
    {"id": "optvec", "label": "opt/vec", "group": "capability"},
    {"id": "coerce", "label": "coerce", "group": "capability"},
    {"id": "serde", "label": "serde", "group": "capability"},
    {"id": "altrep", "label": "altrep", "group": "capability"},
    {"id": "class", "label": "class", "group": "capability"},
    {"id": "macro", "label": "macro", "group": "macro"},
    {"id": "rustdoc", "label": "rustdoc", "group": "quality"},
    {"id": "docs", "label": "docs/", "group": "quality"},
    {"id": "rpkg", "label": "rpkg", "group": "quality"},
    {"id": "rtests", "label": "R tests", "group": "quality"},
    {"id": "rusttests", "label": "rust tests", "group": "quality"},
    {"id": "crosspkg", "label": "cross-pkg", "group": "quality"},
]

CELL_LABELS = {c["id"]: c["label"] for c in CELL_COLUMNS}

LEGEND = {
    "statuses": {
        "yes": "works / covered",
        "partial": "works with a restriction, or partly covered (see evidence)",
        "no": "does not apply to this type (capability columns: neutral, not a defect)",
        "gap": "missing coverage (quality columns: a real gap)",
        "n/a": "the rule cannot apply to this row",
    },
    "glyphs": {"yes": "●", "partial": "◐", "no": "○", "gap": "×", "n/a": "·"},
    "columns": {
        "arg": (
            "A #[miniextendr] argument: an impl of TryFromSexp or TryFromRoot (borrowed forms) "
            "for the type. yes when it holds for every instance (a bound on the type's own "
            "parameter is shown in the evidence); partial when only some concrete instances "
            "convert, or only &T / &mut T. Parameters the macros bind by name with no trait "
            "(LazyDots, &Dots, NArgs, Call, NonNa, RngGuard, ExternalPtrRef, ...) are yes "
            "'macro'. A trait row is yes when a blanket impl converts every implementor, "
            "partial when a container does (ExternalPtr<T: TypedExternal>, Vec<T>)."
        ),
        "ret": (
            "A #[miniextendr] return: an impl of IntoR or IntoRAs, same yes/partial rules as "
            "arg. Invisible / Visible are peeled by name."
        ),
        "optvec": (
            "Option<Self> and Vec<Self> (or Vec<Option<Self>>) convert in at least one "
            "direction: a concrete impl for that container of the type, or a generic container "
            "impl whose bounds on its element the type satisfies (TryFromSexpElement, "
            "IntoRVecElement, RNativeType, TypedExternal, ...). yes = both, partial = one. "
            "Converted tab: only for scalar rows; foreign-trait bounds count as satisfied "
            "there because std impls are outside this rustdoc."
        ),
        "coerce": "Coerce / TryCoerce impls with the type as source or target.",
        "serde": (
            "RSerializeNative / RDeserializeNative (blanket impls over serde's Serialize / "
            "Deserialize count). yes = both, partial = one. Converted tab: n/a unless "
            "miniextendr implements them itself; std and third-party serde impls live upstream."
        ),
        "altrep": (
            "ALTREP traits (the altrep* modules, IntoRAltrep; extension traits ending in Ext "
            "excluded). yes = an impl written for the type, partial = blanket impls only "
            "(AltrepExtract via TypedExternal)."
        ),
        "class": (
            "yes = implements TypedExternal: an ExternalPtr payload, the object of a "
            "#[miniextendr] impl. partial = an #[r_data] field type: a plain field needs "
            "TryFromSexp + IntoR + Clone and a name outside the derive's refusal list of "
            "unrooted R views; a refused view still fits Sidecar<T> (TryFromSexp + IntoR)."
        ),
        "macro": (
            "The proc macros match the type's name: a whole string literal equal to the name "
            "or a path ending in it (\"Nullable\", \"jiff::Timestamp\", seg.ident == \"Dots\") in "
            "miniextendr-macros/src code, comments, tests.rs, tests/ and snapshots/ skipped. "
            "Evidence names the file, line and enclosing fn. Converted tab: the name of the "
            "type or of one of its constructors."
        ),
        "rustdoc": (
            "yes = docs with an example code block and at least half of the public methods "
            "documented; partial = docs without an example, or fewer than half of the methods "
            "documented (fraction shown); gap = no docs. Converted tab: n/a (documented upstream)."
        ),
        "docs": (
            "Mentioned in docs/*.md, inside inline code or fenced code blocks only (prose "
            "words like 'Missing' or 'List' do not count). Converted tab: a code span or code "
            "type that normalizes to the row's shape. gap when absent."
        ),
        "rpkg": (
            "Used in the rpkg demonstration crate (rpkg/src/rust, git-tracked files, comments "
            "and strings stripped): the identifier anywhere for the types tab; an argument or "
            "return type of an rpkg #[miniextendr] fn that normalizes to the row's shape for "
            "the converted tab. gap when absent."
        ),
        "rtests": (
            "Heuristic: an rpkg #[miniextendr] fn whose signature uses the type has its R name "
            "in rpkg/tests/testthat/*.R (a method needs its name and its type's name in one "
            "file). gap = rpkg signatures use it but none is named in a test; n/a = no rpkg "
            "signature uses it."
        ),
        "rusttests": (
            "Mentioned in miniextendr-api/tests, miniextendr-engine/tests or a tests.rs / "
            "tests/ module under their src (code only). Converted tab: a type in a let, a "
            "parameter, a return or a turbofish that normalizes to the row's shape. gap when absent."
        ),
        "crosspkg": (
            "Mentioned in tests/cross-package (Rust and R, generated wrappers skipped). yes or "
            "n/a: the cross-package tests target the trait ABI, so absence is not a gap."
        ),
    },
    "rows": {
        "types": (
            "Every public struct, enum, union, trait and type alias reachable from the crate "
            "root of miniextendr-api (features: full) and miniextendr-engine, at its canonical "
            "path; other public paths (prelude, re-exports) are listed as evidence. "
            "#[doc(hidden)] items are not in rustdoc's output and are not listed."
        ),
        "converted": (
            "The for-types of the conversion traits (TryFromSexp, TryFromRoot, IntoR, IntoRAs, "
            "Coerce, TryCoerce, the serde-native traits, the element traits RNativeType / "
            "TryFromSexpElement / IntoRVecElement / IntoRNewtype, the ALTREP traits) that "
            "mention no miniextendr type. Shapes are normalized: lifetimes dropped, the sized "
            "leaves of a container folded to a positional placeholder (Vec<i32>, Vec<f64> and "
            "Vec<T> are the row Vec<T>; a map keeps its key type; tuples of any arity are "
            "(T1, ..., Tn)), so the instances are evidence. A std container around a miniextendr "
            "type credits that type's opt/vec cell instead."
        ),
        "feature": (
            "From rustdoc's cfg traces on the item (or impl) and every enclosing module; an "
            "impl inherits the module that owns its source file, else the optional dependency "
            "its type comes from. 'always' when ungated; non-feature cfgs (r_4_6, target_arch) "
            "are shown as cfg(...)."
        ),
    },
}

# endregion


# region: small helpers


def kind_of(item: dict) -> str:
    inner = item.get("inner")
    if isinstance(inner, dict):
        return next(iter(inner))
    return str(inner)


def ev(text: str, path: str | None = None, line: int | None = None) -> dict:
    out = {"text": text}
    if path:
        out["path"] = path
        if line:
            out["line"] = int(line)
    return out


def ev_key(e: dict):
    return (e.get("path") or "", e.get("line") or 0, e["text"])


def cell(status: str, label: str, evidence: list[dict] | None = None, sort: bool = True) -> dict:
    evidence = list(evidence or [])
    if sort:
        seen = set()
        uniq = []
        for e in sorted(evidence, key=ev_key):
            k = ev_key(e)
            if k not in seen:
                seen.add(k)
                uniq.append(e)
        evidence = uniq
    more = max(0, len(evidence) - EVIDENCE_CAP)
    out = {"status": status, "label": label, "evidence": evidence[:EVIDENCE_CAP]}
    if more:
        out["more"] = more
    return out


def repo_relative(path: str | None) -> str | None:
    """Keep only spans inside the repository (rustdoc may cite registry files)."""
    if not path or path.startswith("/") or path.startswith(".."):
        return None
    return path


def span_of(item: dict) -> tuple[str | None, int | None]:
    sp = item.get("span")
    if not sp:
        return None, None
    return repo_relative(sp.get("filename")), sp.get("begin", [None])[0]


# endregion


# region: cfg traces


_CFG_SPAN_FIELD = re.compile(r",\s*span:\s*[^{}\[\]]*?\(#\d+\)")
_CFG_TRAILING_SPAN = re.compile(r",\s*[^\s,()\[\]{}]+:\d+:\d+:\s*\d+:\d+\s*\(#\d+\)")
_CFG_TOKEN = re.compile(
    r'NameValue\s*\{\s*name:\s*"(?P<name>[^"]*)",\s*value:\s*(?:None|Some\("(?P<value>[^"]*)"\))\s*\}'
    r"|(?P<op>All|Any|Not|True|False)\s*\(?"
    r"|(?P<punct>[\[\](),])"
)


def parse_cfg_trace(attr_text: str):
    """Parse rustdoc's ``#[attr = CfgTrace([...])]`` text into a predicate list.

    A predicate is ``("nv", name, value)``, ``("all", [..])``, ``("any", [..])``,
    ``("not", p)`` or ``("bool", True/False)``. Returns ``None`` for any other
    attribute.
    """
    m = re.match(r"#\[attr\s*=\s*CfgTrace\((.*)\)\]\s*$", attr_text.strip(), re.S)
    if not m:
        return None
    body = _CFG_SPAN_FIELD.sub("", m.group(1))
    body = _CFG_TRAILING_SPAN.sub("", body)
    tokens = []
    pos = 0
    while pos < len(body):
        if body[pos].isspace():
            pos += 1
            continue
        t = _CFG_TOKEN.match(body, pos)
        if not t:
            raise ValueError(f"unparsed cfg trace near {body[pos:pos + 40]!r}")
        pos = t.end()
        if t.group("name") is not None:
            tokens.append(("nv", t.group("name"), t.group("value")))
        elif t.group("op"):
            tokens.append(("op", t.group("op")))
        else:
            tokens.append(("p", t.group("punct")))

    def parse_list(i):
        # tokens[i] == '['
        items = []
        i += 1
        while tokens[i] != ("p", "]"):
            pred, i = parse_pred(i)
            items.append(pred)
            if tokens[i] == ("p", ","):
                i += 1
        return items, i + 1

    def parse_pred(i):
        tok = tokens[i]
        if tok[0] == "nv":
            return ("nv", tok[1], tok[2]), i + 1
        if tok[0] == "op":
            op = tok[1]
            if op in ("True", "False"):
                return ("bool", op == "True"), i + 1
            i += 1
            if op in ("All", "Any"):
                items, i = parse_list(i)
                pred = (op.lower(), items)
            else:
                pred, i = parse_pred(i)
                pred = ("not", pred)
            while tokens[i] == ("p", ","):
                i += 1
            if tokens[i] == ("p", ")"):
                i += 1
            return pred, i
        raise ValueError(f"unexpected cfg token {tok}")

    if tokens and tokens[0] == ("p", "["):
        preds, _ = parse_list(0)
        return preds
    pred, _ = parse_pred(0)
    return [pred]


def cfg_rust(pred) -> str:
    """A predicate in Rust's ``cfg`` syntax."""
    tag = pred[0]
    if tag == "nv":
        return pred[1] if pred[2] is None else f'{pred[1]} = "{pred[2]}"'
    if tag == "bool":
        return "true" if pred[1] else "false"
    if tag == "not":
        return f"not({cfg_rust(pred[1])})"
    return f"{tag}({', '.join(cfg_rust(p) for p in pred[1])})"


def render_cfg(pred) -> str:
    """A feature by name (``a & b``, ``(a | b)`` for all / any of features), else ``cfg(...)``."""
    if pred[0] == "nv" and pred[1] == "feature":
        return pred[2]
    if pred[0] in ("all", "any") and pred[1] and all(p[0] == "nv" and p[1] == "feature" for p in pred[1]):
        names = [p[2] for p in pred[1]]
        if pred[0] == "all" or len(names) == 1:
            return " & ".join(names)
        return "(" + " | ".join(names) + ")"
    return f"cfg({cfg_rust(pred)})"


def cfg_features(pred) -> set[str]:
    tag = pred[0]
    if tag == "nv":
        return {pred[2]} if pred[1] == "feature" and pred[2] else set()
    if tag == "not" or tag == "bool":
        return set()
    out = set()
    for p in pred[1]:
        out |= cfg_features(p)
    return out


def item_cfgs(item: dict) -> list:
    preds = []
    for attr in item.get("attrs", []) or []:
        text = attr.get("other") if isinstance(attr, dict) else None
        if not text or "CfgTrace" not in text or "CfgAttrTrace" in text:
            continue
        parsed = parse_cfg_trace(text)
        if parsed:
            preds.extend(parsed)
    return preds


def feature_text(preds: list) -> str:
    rendered = []
    for p in preds:
        r = render_cfg(p)
        if r not in rendered:
            rendered.append(r)
    return " & ".join(rendered) if rendered else "always"


# endregion


# region: rustdoc crate model


class Crate:
    """One rustdoc JSON document plus the derived indexes the matrix needs."""

    def __init__(self, data: dict, label: str):
        self.data = data
        self.label = label
        self.index = data["index"]
        self.paths = data["paths"]
        self.external = data.get("external_crates", {})
        root = self.index[str(data["root"])]
        self.name = root["name"]
        self.parent = {}
        for it in self.index.values():
            if kind_of(it) == "module":
                for c in it["inner"]["module"]["items"]:
                    self.parent[c] = it["id"]
        # file -> [(begin, end, module id)] for impl -> module lookup
        self.file_modules = defaultdict(list)
        for it in self.index.values():
            if kind_of(it) == "module" and it.get("span"):
                sp = it["span"]
                self.file_modules[sp["filename"]].append((sp["begin"][0], sp["end"][0], it["id"]))
        self._chain_cache = {}

    def item(self, item_id):
        if item_id is None:
            return None
        return self.index.get(str(item_id))

    def crate_name(self, crate_id: int) -> str:
        if crate_id == 0:
            return self.name
        ext = self.external.get(str(crate_id))
        return ext["name"] if ext else f"crate{crate_id}"

    def full_path(self, item_id) -> tuple[str, list[str]] | None:
        """(crate name, path segments without the crate) of an id, local or external."""
        p = self.paths.get(str(item_id))
        if p:
            segs = list(p["path"])
            crate = self.crate_name(p["crate_id"])
            if segs and segs[0] == crate:
                segs = segs[1:]
            return crate, segs
        it = self.item(item_id)
        if it is not None and it.get("name"):
            return self.crate_name(it.get("crate_id", 0)), [it["name"]]
        return None

    def is_local(self, item_id) -> bool:
        p = self.paths.get(str(item_id))
        if p is not None:
            return p["crate_id"] == 0
        it = self.item(item_id)
        return it is not None and it.get("crate_id") == 0

    def module_chain_cfgs(self, module_id) -> list:
        if module_id in self._chain_cache:
            return self._chain_cache[module_id]
        chain = []
        cur = module_id
        seen = set()
        while cur is not None and cur not in seen:
            seen.add(cur)
            it = self.item(cur)
            if it is None:
                break
            chain = item_cfgs(it) + chain
            cur = self.parent.get(cur)
        self._chain_cache[module_id] = chain
        return chain

    def item_cfg_chain(self, item: dict) -> list:
        parent = self.parent.get(item["id"])
        return (self.module_chain_cfgs(parent) if parent is not None else []) + item_cfgs(item)

    def module_of_span(self, filename: str, line: int):
        best = None
        for begin, end, mid in self.file_modules.get(filename, []):
            if begin <= line <= end:
                if best is None or (end - begin) < (best[1] - best[0]):
                    best = (begin, end, mid)
        return best[2] if best else None


def reachable_items(crate: Crate) -> dict:
    """Map item id -> set of public paths (tuples, without the crate name).

    Walks public modules and public ``use`` items (globs included) from the
    crate root. ``--document-private-items`` puts ``pub`` items of private
    modules in the index, so visibility alone does not make an item public API.
    """
    found = defaultdict(set)
    stack = []

    def walk(mod_id, prefix):
        if mod_id in stack:
            return
        mod = crate.item(mod_id)
        if mod is None or kind_of(mod) != "module":
            return
        stack.append(mod_id)
        for cid in mod["inner"]["module"]["items"]:
            child = crate.item(cid)
            if child is None or child.get("visibility") != "public":
                continue
            k = kind_of(child)
            if k == "module":
                walk(cid, prefix + (child["name"],))
            elif k in TYPE_KINDS:
                found[cid].add(prefix + (child["name"],))
            elif k == "use":
                u = child["inner"]["use"]
                target = crate.item(u.get("id"))
                if target is None:
                    continue
                tk = kind_of(target)
                if u.get("is_glob"):
                    if tk == "module":
                        walk(target["id"], prefix)
                elif tk == "module":
                    walk(target["id"], prefix + (u["name"],))
                elif tk in TYPE_KINDS and target.get("crate_id") == 0:
                    found[target["id"]].add(prefix + (u["name"],))
        stack.pop()

    walk(crate.data["root"], ())
    return found


def canonical_path(crate: Crate, item_id, public_paths: set) -> tuple:
    fp = crate.full_path(item_id)
    if fp is not None and tuple(fp[1]) in public_paths:
        return tuple(fp[1])
    return min(public_paths, key=lambda p: (len(p), p))


# endregion


# region: type IR (shared by rustdoc types and source text)
#
# ("prim", name) | ("gen", name) | ("ref", mut, inner) | ("slice", inner)
# | ("array", inner) | ("tuple", [items]) | ("other", text)
# | ("path", last, crate, local_id, args, display)
#     crate: "std", a third-party crate name, the local crate name, or None
#     local_id: (crate label, item id) for a type of the crate being read, else None


def ir_from_rustdoc(ty, crate: Crate):
    if ty is None:
        return ("tuple", [])
    if not isinstance(ty, dict):
        return ("other", str(ty))
    if "primitive" in ty:
        return ("prim", ty["primitive"])
    if "generic" in ty:
        return ("gen", ty["generic"])
    if "borrowed_ref" in ty:
        br = ty["borrowed_ref"]
        return ("ref", bool(br.get("is_mutable")), ir_from_rustdoc(br.get("type"), crate))
    if "slice" in ty:
        return ("slice", ir_from_rustdoc(ty["slice"], crate))
    if "array" in ty:
        return ("array", ir_from_rustdoc(ty["array"].get("type"), crate))
    if "tuple" in ty:
        return ("tuple", [ir_from_rustdoc(t, crate) for t in ty["tuple"]])
    if "resolved_path" in ty:
        rp = ty["resolved_path"]
        last = rp.get("path", "?").split("::")[-1]
        fp = crate.full_path(rp.get("id"))
        local_id = None
        if fp is None:
            crate_name = None
        else:
            crate_name = fp[0]
            last = fp[1][-1] if fp[1] else last
            if crate.is_local(rp.get("id")):
                local_id = (crate.label, rp.get("id"))
        if crate_name in STD_CRATES:
            crate_name = "std"
        args = []
        ab = (rp.get("args") or {}).get("angle_bracketed") if rp.get("args") else None
        if ab:
            for a in ab.get("args", []) or []:
                if "type" in a:
                    args.append(ir_from_rustdoc(a["type"], crate))
                elif "const" in a:
                    args.append(("other", "N"))
        if local_id is not None or crate_name in (None, "std"):
            display = last
        else:
            display = f"{crate_name}::{last}"
        return ("path", last, crate_name, local_id, args, display)
    return ("other", format_type(ty, crate.index))


def ir_locals(ir) -> list:
    """Every local type id inside an IR, with the constructor chain above it."""
    out = []

    def go(node, chain):
        tag = node[0]
        if tag == "path":
            if node[3] is not None:
                out.append((node[3], tuple(chain)))
            for a in node[4]:
                go(a, chain + [node[1]])
        elif tag == "ref":
            go(node[2], chain + ["&mut" if node[1] else "&"])
        elif tag == "slice":
            go(node[1], chain + ["[]"])
        elif tag == "array":
            go(node[1], chain + ["[;]"])
        elif tag == "tuple":
            for t in node[1]:
                go(t, chain + ["()"])

    go(ir, [])
    return out


def placeholder(ctor: str, pos: int, n_args: int) -> str | None:
    if ctor in MAP_NAMES:
        return None if pos == 0 else "V"
    if ctor == "Result":
        return "T" if pos == 0 else "E"
    if ctor == "Either":
        return "L" if pos == 0 else "R"
    if n_args == 1:
        return "T"
    return ["T", "U", "W", "X", "Y", "Z"][min(pos, 5)]


def shape(ir, use_display: bool = True, slot: str | None = None, folded: list | None = None,
          fold_paths: bool = True) -> str:
    """Normalized shape of a type.

    ``slot`` is the placeholder for this position inside a container (``None``
    at the top level or for a kept map key): sized leaves (primitives other
    than ``str``, generics, and with ``fold_paths`` argument-less foreign
    paths) become the placeholder. A reference, slice or array at the top
    level folds primitives and generics only (``&T``, ``[T]``) and keeps a
    named type (``&Path``). Folded leaves are appended to ``folded``.
    """
    tag = ir[0]

    def fold(node):
        if folded is not None:
            folded.append(node)
        return slot

    if tag == "prim":
        if slot is not None and ir[1] != "str":
            return fold(ir)
        return ir[1]
    if tag == "gen":
        return fold(ir) if slot is not None else ir[1]
    if tag == "other":
        return ir[1]
    inner_paths = fold_paths if slot is not None else False
    if tag == "ref":
        return ("&mut " if ir[1] else "&") + shape(ir[2], use_display, slot or "T", folded, inner_paths)
    if tag == "slice":
        return "[" + shape(ir[1], use_display, slot or "T", folded, inner_paths) + "]"
    if tag == "array":
        return "[" + shape(ir[1], use_display, slot or "T", folded, inner_paths) + "; N]"
    if tag == "tuple":
        if not ir[1]:
            return "()"
        if folded is not None:
            folded.extend(ir[1])
        return "(T1, …, Tn)"
    # path
    _, last, crate_name, local_id, args, display = ir
    name = display if use_display else last
    if not args:
        if slot is not None and local_id is None and fold_paths:
            return fold(ir)
        return name
    parts = []
    for pos, a in enumerate(args):
        ph = placeholder(last, pos, len(args))
        parts.append(shape(a, use_display, ph, folded, True))
    return f"{name}<{', '.join(parts)}>"


def folded_leaves(ir) -> list:
    """The leaves ``shape`` replaces with placeholders."""
    out = []
    shape(ir, folded=out)
    return out


def shape_kind(ir) -> str:
    tag = ir[0]
    if tag in ("prim", "gen"):
        return "scalar"
    if tag == "path":
        return "container" if ir[4] else "scalar"
    return {"ref": "reference", "slice": "slice", "array": "array", "tuple": "tuple"}.get(tag, "other")


def top_crate(ir) -> str | None:
    tag = ir[0]
    if tag == "prim":
        return "std"
    if tag == "path":
        return ir[2]
    if tag in ("ref", "slice", "array"):
        return top_crate(ir[-1]) or "std"
    if tag in ("tuple", "gen"):
        return "std"
    return None


def constructor_names(ir) -> list[tuple[str, str | None]]:
    """(name, crate) of the leaf and every constructor in a type."""
    out = []

    def go(node):
        tag = node[0]
        if tag == "prim":
            out.append((node[1], "std"))
        elif tag == "path":
            out.append((node[1], node[2]))
            for a in node[4]:
                if a[0] == "path" and a[4]:
                    go(a)
                elif a[0] in ("ref", "slice", "array"):
                    go(a)
        elif tag in ("ref", "slice", "array"):
            inner = node[-1]
            if inner[0] in ("path",) and inner[4]:
                go(inner)
            elif inner[0] == "prim" and inner[1] == "str":
                go(inner)

    go(ir)
    return out


def unwrap_return(ir):
    """Types a return value converts as: itself, and its Result / marker payload."""
    out = [ir]
    if ir[0] == "path" and ir[1] in ("Result", "Invisible", "Visible") and ir[4]:
        out.extend(unwrap_return(ir[4][0]))
    return out


def path_names(ir) -> set[str]:
    out = set()

    def go(node):
        tag = node[0]
        if tag == "path":
            out.add(node[1])
            for a in node[4]:
                go(a)
        elif tag in ("ref", "slice", "array"):
            go(node[-1])
        elif tag == "tuple":
            for t in node[1]:
                go(t)

    go(ir)
    return out


# endregion


# region: Rust source tokenizer and type parser

_TOK = re.compile(
    r"""
    (?P<ws>\s+)
   |(?P<lc>//[^\n]*)
   |(?P<bc>/\*)
   |(?P<rstr>b?r(?P<hashes>\#*)")
   |(?P<str>b?"(?:[^"\\]|\\.)*")
   |(?P<char>b?'(?:\\(?:u\{[0-9a-fA-F]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])')
   |(?P<life>'[A-Za-z_]\w*)
   |(?P<ident>[A-Za-z_]\w*)
   |(?P<num>\d[\w.]*)
   |(?P<punct>::|->|=>|[^\s\w])
    """,
    re.X | re.S,
)


def tokenize_rust(text: str) -> list[tuple[str, str, int]]:
    """Tokens ``(kind, text, line)``; comments dropped, strings kept as ``str``."""
    out = []
    pos = 0
    line = 1
    n = len(text)
    while pos < n:
        m = _TOK.match(text, pos)
        if not m:
            pos += 1
            continue
        kind = m.lastgroup
        if kind == "hashes":
            kind = "rstr"
        start_line = line
        if kind == "bc":
            depth = 1
            j = m.end()
            while j < n and depth:
                if text.startswith("/*", j):
                    depth += 1
                    j += 2
                elif text.startswith("*/", j):
                    depth -= 1
                    j += 2
                else:
                    j += 1
            line += text.count("\n", pos, j)
            pos = j
            continue
        if kind == "rstr":
            hashes = m.group("hashes") or ""
            close = '"' + hashes
            j = text.find(close, m.end())
            j = n if j < 0 else j + len(close)
            body = text[m.end(): j - len(close)]
            out.append(("str", body, start_line))
            line += text.count("\n", pos, j)
            pos = j
            continue
        tok = m.group(0)
        line += tok.count("\n")
        pos = m.end()
        if kind in ("ws", "lc"):
            continue
        if kind == "str":
            body = tok[tok.index('"') + 1: -1]
            out.append(("str", body, start_line))
        else:
            out.append((kind, tok, start_line))
    return out


def _skip_balanced(toks, i, open_t, close_t):
    depth = 0
    n = len(toks)
    while i < n:
        t = toks[i][1]
        if t == open_t:
            depth += 1
        elif t == close_t:
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return n


def _generic_args(toks, i):
    """Parse ``<...>`` starting at ``toks[i] == '<'``; return (args, next)."""
    n = len(toks)
    args = []
    i += 1
    while i < n:
        t = toks[i]
        if t[1] == ">":
            return args, i + 1
        if t[1] == ",":
            i += 1
            continue
        if t[0] == "life":
            i += 1
            continue
        if t[0] == "ident" and i + 1 < n and toks[i + 1][1] == "=" and (i + 2 < n and toks[i + 2][1] != "="):
            _, i = parse_type(toks, i + 2)
            continue
        if t[0] == "num" or t[1] == "{" or t[1] == "-":
            if t[1] == "{":
                i = _skip_balanced(toks, i, "{", "}")
            else:
                i += 1
            args.append(("other", "N"))
            continue
        ir, j = parse_type(toks, i)
        if ir is None:
            return None, i
        args.append(ir)
        i = j
    return None, i


def parse_type(toks, i, imports: dict | None = None):
    """Parse a Rust type at ``toks[i]``; return ``(ir, next)`` or ``(None, i)``."""
    n = len(toks)
    if i >= n:
        return None, i
    kind, t, _ = toks[i]
    if t == "&":
        j = i + 1
        if j < n and toks[j][0] == "life":
            j += 1
        mut = False
        if j < n and toks[j][1] == "mut":
            mut = True
            j += 1
        inner, j = parse_type(toks, j, imports)
        if inner is None:
            return None, i
        return ("ref", mut, inner), j
    if t == "[":
        inner, j = parse_type(toks, i + 1, imports)
        if inner is None or j >= n:
            return None, i
        if toks[j][1] == "]":
            return ("slice", inner), j + 1
        if toks[j][1] == ";":
            k = j + 1
            depth = 0
            while k < n:
                if toks[k][1] in "([{":
                    depth += 1
                elif toks[k][1] in ")}":
                    depth -= 1
                elif toks[k][1] == "]":
                    if depth == 0:
                        break
                    depth -= 1
                k += 1
            return ("array", inner), k + 1
        return None, i
    if t == "(":
        j = i + 1
        if j < n and toks[j][1] == ")":
            return ("tuple", []), j + 1
        items = []
        trailing = False
        while j < n:
            item, j = parse_type(toks, j, imports)
            if item is None:
                return None, i
            items.append(item)
            if j < n and toks[j][1] == ",":
                j += 1
                if j < n and toks[j][1] == ")":
                    trailing = True
                    j += 1
                    break
                continue
            if j < n and toks[j][1] == ")":
                j += 1
                break
            return None, i
        if len(items) == 1 and not trailing:
            return items[0], j
        return ("tuple", items), j
    if t in ("dyn", "impl"):
        j = i + 1
        parts = []
        while j < n:
            if toks[j][0] == "life":
                parts.append(toks[j][1])
                j += 1
            else:
                p, j2 = parse_type(toks, j, imports)
                if p is None or p[0] != "path":
                    break
                parts.append(shape(p))
                j = j2
            if j < n and toks[j][1] == "+":
                j += 1
                continue
            break
        if not parts:
            return None, i
        return ("other", f"{t} " + " + ".join(parts)), j
    if t == "*" and i + 1 < n and toks[i + 1][1] in ("const", "mut"):
        inner, j = parse_type(toks, i + 2, imports)
        if inner is None:
            return None, i
        return ("other", "*" + toks[i + 1][1] + " " + shape(inner)), j
    if kind != "ident" and t != "::":
        return None, i
    # path
    j = i
    segs = []
    args = []
    if toks[j][1] == "::":
        j += 1
    while j < n and toks[j][0] == "ident":
        name = toks[j][1]
        if name in RUST_KEYWORDS and name not in ("crate", "self", "super"):
            return None, i
        segs.append(name)
        j += 1
        args = []
        if j < n and toks[j][1] == "<":
            got, j2 = _generic_args(toks, j)
            if got is None:
                return None, i
            args, j = got, j2
        if j + 1 < n and toks[j][1] == "::" and toks[j + 1][1] == "<":
            got, j2 = _generic_args(toks, j + 1)
            if got is None:
                return None, i
            args, j = got, j2
        if j + 1 < n and toks[j][1] == "::" and toks[j + 1][0] == "ident":
            j += 1
            continue
        break
    if not segs:
        return None, i
    last = segs[-1]
    if len(segs) == 1 and last in PRIMITIVES and not args:
        return ("prim", last), j
    crate_hint = None
    if len(segs) >= 2:
        root = segs[0]
        if root in STD_CRATES:
            crate_hint = "std"
        elif root not in ("crate", "self", "super", "Self"):
            crate_hint = root
    elif imports and last in imports:
        crate_hint = imports[last]
    elif last in STD_PRELUDE_NAMES:
        crate_hint = "std"
    if crate_hint == "miniextendr_api":
        crate_hint = None
    return ("path", last, crate_hint, None, args, last), j


def rust_imports(toks) -> dict:
    """Last name -> crate root for the ``use`` items of a file."""
    out = {}
    n = len(toks)
    i = 0
    while i < n:
        if toks[i][1] == "use" and toks[i][0] == "ident":
            j = i + 1
            stmt = []
            while j < n and toks[j][1] != ";":
                stmt.append(toks[j][1])
                j += 1
            text = "".join(stmt)
            root = text.split("::", 1)[0].lstrip(":")
            if root in STD_CRATES:
                root = "std"
            for name in re.findall(r"[A-Za-z_]\w*", text.split("::", 1)[-1]):
                if name not in ("self", "as", "super", "crate") and root not in ("crate", "self", "super"):
                    out.setdefault(name, root)
            i = j
        i += 1
    return out


def type_mentions(toks, imports=None) -> list[tuple[object, int]]:
    """Types written after ``:``, ``->``, ``for``, a turbofish or a qualified ``<``."""
    out = []
    n = len(toks)
    for i, tok in enumerate(toks):
        t = tok[1]
        start = None
        if t in (":", "->"):
            start = i + 1
        elif t == "<" and i > 0 and (toks[i - 1][1] == "::" or toks[i - 1][1] in ("(", "=", ",", "{", ";", "return")):
            start = i + 1
        elif t == "for" and tok[0] == "ident" and i > 0:
            start = i + 1
        if start is None or start >= n:
            continue
        ir, _ = parse_type(toks, start, imports)
        if ir is not None:
            out.append((ir, toks[start][2]))
    return out


def code_identifiers(toks) -> list[tuple[str, int]]:
    return [(t[1], t[2]) for t in toks if t[0] == "ident"]


# endregion


# region: repository files


class Sources:
    """Reads tracked repository files (``git ls-files``), or the tree for fixtures."""

    def __init__(self, root: Path):
        self.root = root
        self._tracked = None
        try:
            res = subprocess.run(
                ["git", "ls-files", "-z"], cwd=root, capture_output=True, check=True
            )
            self._tracked = sorted(p for p in res.stdout.decode().split("\0") if p)
        except (subprocess.CalledProcessError, FileNotFoundError):
            self._tracked = None
        self._tok_cache = {}
        self._line_cache = {}

    def files(self, prefix: str, suffixes: tuple[str, ...]) -> list[str]:
        prefix = prefix.rstrip("/") + "/"
        if self._tracked is not None:
            return [p for p in self._tracked if p.startswith(prefix) and p.endswith(suffixes)]
        base = self.root / prefix
        if not base.exists():
            return []
        return sorted(
            str(p.relative_to(self.root))
            for p in base.rglob("*")
            if p.is_file() and str(p).endswith(suffixes)
        )

    def read(self, rel: str) -> str:
        try:
            return (self.root / rel).read_text(encoding="utf-8", errors="replace")
        except OSError:
            return ""

    def line(self, rel: str, line: int) -> str:
        """One source line, stripped and shortened, for evidence text."""
        if rel not in self._line_cache:
            self._line_cache[rel] = self.read(rel).splitlines()
        lines = self._line_cache[rel]
        text = lines[line - 1].strip() if 0 < line <= len(lines) else ""
        return text if len(text) <= 120 else text[:117] + "..."

    def tokens(self, rel: str):
        if rel not in self._tok_cache:
            self._tok_cache[rel] = tokenize_rust(self.read(rel))
        return self._tok_cache[rel]


def markdown_code(text: str) -> list[tuple[str, int, bool]]:
    """``(code, line, is_inline)`` for inline code spans and fenced block lines."""
    out = []
    fence = None
    for lineno, line in enumerate(text.splitlines(), 1):
        stripped = line.strip()
        if fence is None:
            m = re.match(r"^(`{3,}|~{3,})", stripped)
            if m:
                fence = m.group(1)
                continue
            for span in re.finditer(r"(`+)(.+?)\1", line):
                out.append((span.group(2).strip(), lineno, True))
        else:
            if stripped.startswith(fence):
                fence = None
                continue
            out.append((line, lineno, False))
    return out


# endregion


# region: source scans


class MacroScan:
    """String literals of the proc-macro crate, with the enclosing fn."""

    def __init__(self, sources: Sources):
        self.hits = defaultdict(list)  # last name -> [(qualifier, path, line, fn)]
        for rel in sources.files("miniextendr-macros/src", (".rs",)):
            parts = rel.split("/")
            if parts[-1] == "tests.rs" or "tests" in parts[:-1] or "snapshots" in parts:
                continue
            toks = sources.tokens(rel)
            fn_stack = []  # (name, depth)
            depth = 0
            pending_fn = None
            item = None  # the const / static a literal outside any fn belongs to
            for idx, (kind, text, line) in enumerate(toks):
                if kind == "ident" and text == "fn" and idx + 1 < len(toks) and toks[idx + 1][0] == "ident":
                    pending_fn = toks[idx + 1][1]
                elif kind == "ident" and text in ("const", "static") and not fn_stack and idx + 1 < len(toks) and toks[idx + 1][0] == "ident" and toks[idx + 1][1] != "fn":
                    item = f"{text} {toks[idx + 1][1]}"
                elif text == ";" and pending_fn is not None and kind == "punct":
                    pending_fn = None
                elif text == "{" and kind == "punct":
                    depth += 1
                    if pending_fn is not None:
                        fn_stack.append((pending_fn, depth))
                        pending_fn = None
                elif text == "}" and kind == "punct":
                    while fn_stack and fn_stack[-1][1] >= depth:
                        fn_stack.pop()
                    depth -= 1
                elif kind == "str":
                    m = re.fullmatch(r"(?:::)?((?:[A-Za-z_]\w*::)*)([A-Za-z_]\w*)", text)
                    if m:
                        qual = m.group(1).rstrip(":").split("::")[0] if m.group(1) else None
                        where = f"fn {fn_stack[-1][0]}" if fn_stack else item
                        self.hits[m.group(2)].append((qual, rel, line, where))

    def evidence(self, name: str, crate_name: str | None = None) -> list[dict]:
        out = []
        for qual, path, line, where in self.hits.get(name, []):
            if qual and crate_name and qual not in (crate_name, "miniextendr_api", "std", "core", "alloc"):
                continue
            literal = f"{qual}::{name}" if qual else name
            out.append(ev(f'"{literal}" in {where}' if where else f'"{literal}"', path, line))
        return out


class IdentIndex:
    """Identifier -> [(path, line)] over a set of files (code tokens only)."""

    def __init__(self):
        self.where = defaultdict(list)

    def add_rust(self, sources: Sources, rel: str):
        for name, line in code_identifiers(sources.tokens(rel)):
            self.where[name].append((rel, line))

    def add_text(self, rel: str, text: str, line: int):
        for name in re.findall(r"[A-Za-z_][A-Za-z0-9_]*", text):
            self.where[name].append((rel, line))

    def get(self, name: str) -> list[tuple[str, int]]:
        return self.where.get(name, [])


class ShapeIndex:
    """Normalized type shapes found in a set of sources: match key -> hits."""

    def __init__(self):
        self.where = defaultdict(list)  # match shape -> [(crate hint, path, line, display)]

    def add(self, ir, rel: str, line: int):
        key = shape(ir, use_display=False)
        self.where[key].append((top_crate(ir), rel, line, shape(ir)))

    def get(self, key: str, crate_name: str | None) -> list[tuple[str, int, str]]:
        out = []
        for hint, rel, line, disp in self.where.get(key, []):
            if hint is not None and crate_name is not None and hint != crate_name:
                continue
            out.append((rel, line, disp))
        return out


def parse_miniextendr_items(sources: Sources, rel: str) -> list[dict]:
    """``#[miniextendr]`` fns and impl methods of one file, with their signature types."""
    toks = sources.tokens(rel)
    imports = rust_imports(toks)
    out = []
    n = len(toks)

    def attr_at(i):
        """If toks[i] starts ``#[...]``, return (attr tokens, next)."""
        if i + 1 < n and toks[i][1] == "#" and toks[i + 1][1] == "[":
            j = _skip_balanced(toks, i + 1, "[", "]")
            return toks[i + 2: j - 1], j
        return None, i

    def r_name_of(attr_toks):
        for k in range(len(attr_toks) - 2):
            if attr_toks[k][1] == "r_name" and attr_toks[k + 1][1] == "=" and attr_toks[k + 2][0] == "str":
                return attr_toks[k + 2][1]
        return None

    def parse_fn(i):
        """toks[i] == 'fn'; return (record, next) with param/return IRs."""
        if i + 1 >= n or toks[i + 1][0] != "ident":
            return None, i + 1
        name = toks[i + 1][1]
        line = toks[i + 1][2]
        j = i + 2
        if j < n and toks[j][1] == "<":
            depth = 0
            while j < n:
                if toks[j][1] == "<":
                    depth += 1
                elif toks[j][1] == ">":
                    depth -= 1
                    if depth == 0:
                        j += 1
                        break
                j += 1
        if j >= n or toks[j][1] != "(":
            return None, j
        end = _skip_balanced(toks, j, "(", ")")
        params = []
        k = j + 1
        segment = []
        depth = 0
        for tok in toks[j + 1: end - 1]:
            if tok[1] in "<([" and tok[0] == "punct":
                depth += 1
            elif tok[1] in ">)]" and tok[0] == "punct":
                depth -= 1
            if tok[1] == "," and depth == 0:
                params.append(segment)
                segment = []
            else:
                segment.append(tok)
        if segment:
            params.append(segment)
        types = []
        for seg in params:
            # skip attributes on the parameter
            s = 0
            while s + 1 < len(seg) and seg[s][1] == "#" and seg[s + 1][1] == "[":
                s = _skip_balanced(seg, s + 1, "[", "]")
            seg = seg[s:]
            colon = next((x for x, tok in enumerate(seg) if tok[1] == ":"), None)
            if colon is None:
                continue  # self receiver
            ir, _ = parse_type(seg, colon + 1, imports)
            if ir is not None:
                types.append(("arg", ir))
        k = end
        if k < n and toks[k][1] == "->":
            ir, _ = parse_type(toks, k + 1, imports)
            if ir is not None:
                types.append(("ret", ir))
        return {"name": name, "line": line, "types": types}, end

    i = 0
    while i < n:
        attr, j = attr_at(i)
        if attr is None or not attr or attr[0][1] != "miniextendr":
            i += 1
            continue
        r_name = r_name_of(attr)
        # skip further attributes and qualifiers
        while True:
            more, j2 = attr_at(j)
            if more is None:
                break
            j = j2
        while j < n and toks[j][1] in ("pub", "unsafe", "extern", "const", "async") or (
            j < n and toks[j][0] == "str"
        ):
            if toks[j][1] == "pub" and j + 1 < n and toks[j + 1][1] == "(":
                j = _skip_balanced(toks, j + 1, "(", ")")
            else:
                j += 1
        if j < n and toks[j][1] == "fn":
            rec, j = parse_fn(j)
            if rec:
                rec.update(kind="fn", r_name=r_name or rec["name"], impl_type=None, path=rel)
                out.append(rec)
            i = j
            continue
        if j < n and toks[j][1] == "impl":
            k = j + 1
            header = []
            while k < n and toks[k][1] != "{":
                header.append(toks[k])
                k += 1
            names = [t[1] for t in header if t[0] == "ident" and t[1] not in ("for", "where", "impl")]
            if any(t[1] == "for" for t in header):
                after = header[[t[1] for t in header].index("for") + 1:]
                names = [t[1] for t in after if t[0] == "ident"]
            impl_type = names[0] if names else None
            body_end = _skip_balanced(toks, k, "{", "}")
            depth = 0
            m = k
            while m < body_end:
                t = toks[m][1]
                if t == "{":
                    depth += 1
                elif t == "}":
                    depth -= 1
                elif t == "fn" and depth == 1:
                    method_attrs = None
                    back = m - 1
                    while back > k and toks[back][1] in ("pub", "unsafe", "const", "async", ")", "crate", "("):
                        back -= 1
                    if toks[back][1] == "]":
                        # walk to this attribute's start to read r_name
                        b = back
                        depth_b = 0
                        while b > k:
                            if toks[b][1] == "]":
                                depth_b += 1
                            elif toks[b][1] == "[":
                                depth_b -= 1
                                if depth_b == 0:
                                    break
                            b -= 1
                        method_attrs = toks[b + 1: back]
                    rec, m2 = parse_fn(m)
                    if rec:
                        r = r_name_of(method_attrs or []) or rec["name"]
                        rec.update(kind="method", r_name=r, impl_type=impl_type, path=rel)
                        out.append(rec)
                    m = max(m2, m + 1)
                    continue
                m += 1
            i = body_end
            continue
        i = j + 1
    return out


class SourceScans:
    """All repository scans the quality and macro columns read."""

    def __init__(self, sources: Sources):
        self.sources = sources
        self.macros = MacroScan(sources)

        # docs/*.md: identifiers and type shapes inside code
        self.docs_idents = IdentIndex()
        self.docs_shapes = ShapeIndex()
        for rel in sources.files("docs", (".md",)):
            if rel.count("/") != 1:
                continue
            for code, line, inline in markdown_code(sources.read(rel)):
                toks = tokenize_rust(code)
                for name, _ in code_identifiers(toks):
                    self.docs_idents.where[name].append((rel, line))
                if inline:
                    ir, end = parse_type(toks, 0)
                    if ir is not None and end == len(toks):
                        self.docs_shapes.add(ir, rel, line)
                for ir, _ in type_mentions(toks):
                    self.docs_shapes.add(ir, rel, line)

        # rpkg demonstration crate
        self.rpkg_idents = IdentIndex()
        self.rpkg_items = []
        for rel in sources.files("rpkg/src/rust", (".rs",)):
            if rel.endswith("wasm_registry.rs"):
                continue
            self.rpkg_idents.add_rust(sources, rel)
            self.rpkg_items.extend(parse_miniextendr_items(sources, rel))
        self.rpkg_sig_shapes = ShapeIndex()  # match key -> rpkg fn signature hits
        self.rpkg_fn_by_hit = {}
        for idx, item in enumerate(self.rpkg_items):
            for role, ir in item["types"]:
                for t in (unwrap_return(ir) if role == "ret" else [ir]):
                    self.rpkg_sig_shapes.add(t, item["path"], item["line"])
                    self.rpkg_fn_by_hit.setdefault((item["path"], item["line"]), idx)

        # R tests: identifier sets per testthat file
        self.r_test_files = {}
        for rel in sources.files("rpkg/tests/testthat", (".R", ".r")):
            text = sources.read(rel)
            self.r_test_files[rel] = set(re.findall(r"[A-Za-z.][A-Za-z0-9._]*", text))

        # Rust tests
        self.rust_test_idents = IdentIndex()
        self.rust_test_shapes = ShapeIndex()
        test_files = []
        for crate_dir in ("miniextendr-api", "miniextendr-engine"):
            test_files += sources.files(f"{crate_dir}/tests", (".rs",))
            for rel in sources.files(f"{crate_dir}/src", (".rs",)):
                parts = rel.split("/")
                if parts[-1] == "tests.rs" or "tests" in parts[2:-1]:
                    test_files.append(rel)
        for rel in sorted(set(test_files)):
            self.rust_test_idents.add_rust(sources, rel)
            toks = sources.tokens(rel)
            for ir, line in type_mentions(toks, rust_imports(toks)):
                self.rust_test_shapes.add(ir, rel, line)

        # cross-package tests
        self.cross_idents = IdentIndex()
        self.cross_shapes = ShapeIndex()
        for rel in sources.files("tests/cross-package", (".rs", ".R")):
            if rel.endswith("-wrappers.R"):
                continue
            if rel.endswith(".rs"):
                self.cross_idents.add_rust(sources, rel)
                toks = sources.tokens(rel)
                for ir, line in type_mentions(toks, rust_imports(toks)):
                    self.cross_shapes.add(ir, rel, line)
            else:
                for lineno, line in enumerate(sources.read(rel).splitlines(), 1):
                    code = line.split("#", 1)[0]
                    self.cross_idents.add_text(rel, code, lineno)

    def tested(self, item: dict) -> list[str]:
        """testthat files naming an rpkg item (a method needs its type's name too)."""
        out = []
        for rel, names in sorted(self.r_test_files.items()):
            if item["r_name"] not in names:
                continue
            if item["kind"] == "method" and item["impl_type"] and item["impl_type"] not in names:
                continue
            out.append(rel)
        return out


# endregion


# region: impl records


class ImplRecord:
    def __init__(self, crate: Crate, item: dict, column: str, trait_path: tuple[str, list[str]]):
        imp = item["inner"]["impl"]
        self.crate = crate
        self.id = item["id"]
        self.column = column
        self.trait = trait_path[1][-1]
        self.trait_key = (trait_path[0], tuple(trait_path[1]))
        self.blanket = imp.get("blanket_impl") is not None
        self.for_ir = ir_from_rustdoc(imp["for"], crate)
        self.for_text = format_type(imp["for"], crate.index)
        targs = (imp["trait"].get("args") or {}).get("angle_bracketed") if imp["trait"].get("args") else None
        self.trait_args = [ir_from_rustdoc(a["type"], crate) for a in (targs or {}).get("args", []) if "type" in a]
        self.trait_text = imp["trait"]["path"].split("::")[-1]
        if self.trait_args:
            self.trait_text += "<" + ", ".join(
                format_type(a["type"], crate.index) for a in targs["args"] if "type" in a
            ) + ">"
        params, where = format_generics(imp.get("generics", {}), crate.index)
        self.generics_text = params + where
        self.bounds = impl_bounds(imp.get("generics", {}), crate)
        self.path, self.line = span_of(item)
        cfgs = item_cfgs(item)
        sp = item.get("span")
        if sp:
            mod = crate.module_of_span(sp["filename"], sp["begin"][0])
            if mod is not None:
                cfgs = crate.module_chain_cfgs(mod) + cfgs
        self.cfgs = cfgs

    def describe(self) -> str:
        where = f"  [{self.generics_text}]" if self.generics_text else ""
        kind = " (blanket)" if self.blanket else ""
        return f"{self.trait_text} for {self.for_text}{where}{kind}"

    def evidence(self) -> dict:
        return ev(self.describe(), self.path, self.line)


def impl_bounds(generics: dict, crate: Crate) -> dict:
    """Generic param name -> set of (crate, trait path) bounds."""
    out = defaultdict(set)

    def add(name, bound):
        tb = bound.get("trait_bound") if isinstance(bound, dict) else None
        if not tb:
            return
        if tb.get("modifier") == "maybe":
            return
        fp = crate.full_path(tb["trait"].get("id"))
        if fp:
            out[name].add((fp[0], tuple(fp[1])))

    for p in generics.get("params", []) or []:
        k = p.get("kind", {})
        if "type" in k:
            out.setdefault(p["name"], set())
            for b in k["type"].get("bounds", []) or []:
                add(p["name"], b)
    for wp in generics.get("where_predicates", []) or []:
        bp = wp.get("bound_predicate")
        if bp and isinstance(bp.get("type"), dict) and "generic" in bp["type"]:
            for b in bp.get("bounds", []) or []:
                add(bp["type"]["generic"], b)
    return dict(out)


def trait_column(crate_name: str, segs: list[str]) -> str | None:
    if crate_name != "miniextendr_api" or not segs:
        return None
    last = segs[-1]
    if last in ARG_TRAITS:
        return "arg"
    if last in RET_TRAITS:
        return "ret"
    if last in COERCE_TRAITS:
        return "coerce"
    if last in SERDE_TRAITS:
        return "serde"
    if last in ELEMENT_TRAITS:
        return "element"
    if last in CLASS_TRAITS:
        return "class"
    if last == "IntoRAltrep" or (any(s.startswith("altrep") for s in segs[:-1]) and not last.endswith("Ext")):
        return "altrep"
    return None


def collect_impls(crate: Crate) -> list[ImplRecord]:
    out = []
    for item in crate.index.values():
        if kind_of(item) != "impl":
            continue
        imp = item["inner"]["impl"]
        if not imp.get("trait") or imp.get("is_synthetic") or imp.get("is_negative"):
            continue
        fp = crate.full_path(imp["trait"].get("id"))
        if fp is None:
            continue
        col = trait_column(fp[0], fp[1])
        if col is None:
            continue
        out.append(ImplRecord(crate, item, col, fp))
    out.sort(key=lambda r: (r.path or "", r.line or 0, r.describe()))
    return out


# endregion


# region: cell rules


def status_from_flags(both: bool, one: bool) -> str:
    return "yes" if both else ("partial" if one else "no")


def self_relation(rec: ImplRecord, key) -> str | None:
    """How ``key`` (a local type id) is the for-type of an impl."""
    ir = rec.for_ir
    if ir[0] == "path" and ir[3] == key:
        concrete = [a for a in ir[4] if a[0] != "gen"]
        return "instance" if concrete else "self"
    if ir[0] == "ref" and ir[2][0] == "path" and ir[2][3] == key:
        return "borrowed"
    return None


def capability_cell(recs_self: list, recs_borrowed: list, recs_instance: list, label_yes="yes") -> dict:
    evidence = [r.evidence() for r in recs_self + recs_borrowed + recs_instance]
    if recs_self:
        return cell("yes", label_yes, evidence)
    if recs_instance:
        return cell("partial", "inst", evidence)
    if recs_borrowed:
        return cell("partial", "&T", evidence)
    return cell("no", "no", [])


def bounds_satisfied(rec: ImplRecord, param: str, traits: set, foreign_ok: bool) -> bool:
    need = rec.bounds.get(param, set())
    for b in need:
        if b in traits:
            continue
        if foreign_ok and b[0] != "miniextendr_api":
            continue
        if b[0] in STD_CRATES and b[1][-1] in ("Sized",):
            continue
        return False
    return True


def container_element(ir):
    """For Option<X> / Vec<X> / Vec<Option<X>>: (container label, element IR)."""
    if ir[0] != "path" or ir[2] != "std" or len(ir[4]) != 1:
        return None
    inner = ir[4][0]
    if ir[1] == "Option":
        return "Option", inner
    if ir[1] == "Vec":
        if inner[0] == "path" and inner[1] == "Option" and inner[2] == "std" and len(inner[4]) == 1:
            return "Vec<Option>", inner[4][0]
        return "Vec", inner
    return None


def optvec_cell(matches_element, traits: set, impls: list, foreign_ok: bool) -> dict:
    """``matches_element(ir)`` says whether an element IR is the subject."""
    found = defaultdict(list)  # "Option"/"Vec" -> evidence
    for rec in impls:
        if rec.column not in ("arg", "ret"):
            continue
        ce = container_element(rec.for_ir)
        if ce is None:
            continue
        label, elem = ce
        group = "Option" if label == "Option" else "Vec"
        if elem[0] == "gen":
            if not rec.bounds.get(elem[1]):
                continue  # unbounded generic containers are about other element families
            if bounds_satisfied(rec, elem[1], traits, foreign_ok):
                found[group].append(rec.evidence())
        elif matches_element(elem):
            found[group].append(rec.evidence())
    has_opt, has_vec = bool(found["Option"]), bool(found["Vec"])
    label = "+".join(x for x, ok in (("opt", has_opt), ("vec", has_vec)) if ok) or "no"
    return cell(status_from_flags(has_opt and has_vec, has_opt or has_vec), label, found["Option"] + found["Vec"])


# endregion


# region: the types tab


def rustdoc_cell(crate: Crate, item: dict) -> dict:
    docs = (item.get("docs") or "").strip()
    k = kind_of(item)
    methods = []
    if k in ("struct", "enum", "union"):
        for iid in item["inner"][k].get("impls", []):
            imp = crate.item(iid)
            if imp is None:
                continue
            ii = imp["inner"]["impl"]
            if ii.get("trait") or ii.get("is_synthetic") or ii.get("blanket_impl"):
                continue
            for mid in ii.get("items", []):
                m = crate.item(mid)
                if m is not None and kind_of(m) == "function" and m.get("visibility") == "public":
                    methods.append(m)
    elif k == "trait":
        for mid in item["inner"]["trait"].get("items", []):
            m = crate.item(mid)
            if m is not None and kind_of(m) == "function":
                methods.append(m)
    documented = sum(1 for m in methods if (m.get("docs") or "").strip())
    fences = re.findall(r"^\s*(?:```|~~~)\s*([\w,]*)", docs, re.M)
    has_example = any(
        f.split(",")[0].lower() not in NOT_EXAMPLE_FENCES for f in fences[::2]
    )
    path, line = span_of(item)
    evidence = []
    if docs:
        evidence.append(ev(f"docs: {len(docs.splitlines())} lines", path, line))
        evidence.append(ev("example code block" if has_example else "no example code block"))
    if methods:
        evidence.append(ev(f"public methods documented: {documented}/{len(methods)}"))
    if not docs:
        return cell("gap", "none", [ev("no doc comment", path, line)] + evidence, sort=False)
    ratio_ok = not methods or documented * 2 >= len(methods)
    if has_example and ratio_ok:
        return cell("yes", "ex", evidence, sort=False)
    label = "docs" if ratio_ok else f"{documented}/{len(methods)}"
    return cell("partial", label, evidence, sort=False)


def hits_cell(sources: Sources, hits: list, empty_status: str, what: str = "mentions",
              note: str | None = None) -> dict:
    """yes with the first hit of each file (as a source excerpt), else ``empty_status``."""
    if not hits:
        return cell(empty_status, "none" if empty_status == "gap" else "–", [])
    hits = sorted({(h[0], h[1]) for h in hits})
    files = sorted({rel for rel, _ in hits})
    lead = [ev(f"{len(hits)} {what} in {len(files)} files")]
    if note:
        lead.append(ev(note))
    evidence = []
    seen = set()
    for rel, line in hits:
        if rel not in seen:
            seen.add(rel)
            evidence.append(ev(sources.line(rel, line) or rel, rel, line))
    return cell("yes", str(len(files)), lead + evidence, sort=False)


def rtests_cell(scans: SourceScans, items: list[dict]) -> dict:
    if not items:
        return cell("n/a", "–", [ev("no rpkg #[miniextendr] signature uses it")])
    tested, untested = [], []
    for it in items:
        files = scans.tested(it)
        what = f"{it['impl_type']}::{it['name']}" if it["kind"] == "method" else it["name"]
        if files:
            tested.append(ev(f"{what} (R: {it['r_name']}) in {', '.join(files[:3])}", it["path"], it["line"]))
        else:
            untested.append(ev(f"{what} (R: {it['r_name']}) not named in testthat", it["path"], it["line"]))
    if tested:
        lead = [ev(f"{len(tested)} of {len(items)} rpkg fns using it are named in tests")]
        return cell("yes", f"{len(tested)}/{len(items)}", lead + sorted(tested, key=ev_key) + sorted(untested, key=ev_key), sort=False)
    return cell("gap", f"0/{len(items)}", sorted(untested, key=ev_key), sort=False)


def build_types_tab(crates: list[Crate], impls: list[ImplRecord], scans: SourceScans, feature_of_dep: dict) -> list[dict]:
    rows = []
    # name -> how many tab-1 items carry it (ambiguity for name-based scans)
    entries = []
    for crate in crates:
        reach = reachable_items(crate)
        for item_id, public_paths in reach.items():
            item = crate.item(item_id)
            if item is None or item.get("crate_id") != 0:
                continue
            entries.append((crate, item, public_paths))
    name_count = defaultdict(int)
    for _, item, _ in entries:
        name_count[item["name"]] += 1

    by_key = defaultdict(list)  # (label, id) -> impl records relating to it
    for rec in impls:
        seen = set()
        for local, chain in ir_locals(rec.for_ir):
            if (local, chain) not in seen:
                seen.add((local, chain))
                by_key[local].append((rec, "for", chain))
        for targ in rec.trait_args:
            for local, chain in ir_locals(targ):
                by_key[local].append((rec, "target", chain))

    # traits: impls bounded on them
    bounded_on = defaultdict(list)  # (crate, path tuple) -> [(rec, param)]
    for rec in impls:
        for param, bounds in rec.bounds.items():
            for b in bounds:
                bounded_on[b].append((rec, param))

    # rpkg signature users per name
    rpkg_users = defaultdict(list)
    for it in scans.rpkg_items:
        names = set()
        for _, ir in it["types"]:
            names |= path_names(ir)
        for nm in names:
            rpkg_users[nm].append(it)

    refused_views = r_data_refusals(scans.sources)

    alias_rows = []
    row_cells_by_key = {}
    for crate, item, public_paths in entries:
        k = kind_of(item)
        canon = canonical_path(crate, item["id"], public_paths)
        key = (crate.label, item["id"])
        name = item["name"]
        path, line = span_of(item)
        params_text, where_text = format_generics(item["inner"][k].get("generics", {}) if k != "type_alias" else item["inner"]["type_alias"].get("generics", {}), crate.index)
        cfgs = crate.item_cfg_chain(item)
        row = {
            "id": f"{crate.name}::{'::'.join(canon)}",
            "name": name,
            "path": f"{crate.name}::{'::'.join(canon)}",
            "info": {
                "crate": crate.label,
                "module": "::".join(canon[:-1]) or "(root)",
                "kind": k.replace("type_alias", "alias"),
                "generics": (params_text + where_text) or "",
                "feature": feature_text(cfgs),
            },
            "features": sorted(set().union(*[cfg_features(p) for p in cfgs]) if cfgs else set()),
            "source": ev(f"defined at {path}:{line}", path, line) if path else None,
            "reexports": ["::".join((crate.name,) + p) for p in sorted(public_paths - {canon})],
            "cells": {},
        }
        cells = row["cells"]
        related = by_key.get(key, [])
        own_traits = set()
        if k in ("struct", "enum", "union"):
            for iid in item["inner"][k].get("impls", []):
                imp = crate.item(iid)
                if imp is None:
                    continue
                tr = imp["inner"]["impl"].get("trait")
                if tr:
                    fp = crate.full_path(tr.get("id"))
                    if fp:
                        own_traits.add((fp[0], tuple(fp[1])))
            own_blankets = []
            for iid in item["inner"][k].get("impls", []):
                imp = crate.item(iid)
                if imp is None or not imp["inner"]["impl"].get("blanket_impl"):
                    continue
                tr = imp["inner"]["impl"].get("trait")
                fp = crate.full_path(tr.get("id")) if tr else None
                col = trait_column(fp[0], fp[1]) if fp else None
                if col:
                    own_blankets.append(ImplRecord(crate, imp, col, fp))

            by_relation = defaultdict(list)  # (column, "self" | "instance" | "borrowed") -> impls
            for rec, role, _chain in related:
                relation = self_relation(rec, key) if role == "for" else None
                if relation:
                    by_relation[(rec.column, relation)].append(rec)
            blankets = defaultdict(list)
            for rec in own_blankets:
                blankets[rec.column].append(rec)

            def recs(column, relation, _by=by_relation):
                return _by[(column, relation)]

            # arg
            arg = capability_cell(recs("arg", "self") + blankets["arg"], recs("arg", "borrowed"), recs("arg", "instance"))
            if arg["status"] == "no" and name in MACRO_PARAM_TYPES:
                arg = cell("yes", "macro", [ev(MACRO_PARAM_TYPES[name])] + scans.macros.evidence(name))
            cells["arg"] = arg
            ret = capability_cell(recs("ret", "self") + blankets["ret"], recs("ret", "borrowed"), recs("ret", "instance"))
            if name in MACRO_RETURN_TYPES:
                extra = [ev(MACRO_RETURN_TYPES[name])] + scans.macros.evidence(name)
                if ret["status"] == "no":
                    ret = cell("yes", "macro", extra)
                else:
                    ret = cell(ret["status"], ret["label"], ret["evidence"] + extra)
            cells["ret"] = ret
            cells["optvec"] = optvec_cell(
                lambda e, _key=key: e[0] == "path" and e[3] == _key, own_traits, impls, foreign_ok=False
            )
            # coerce: source or target
            src = [rec for rec, role, chain in related if rec.column == "coerce" and role == "for" and not chain]
            tgt = [rec for rec, role, chain in related if rec.column == "coerce" and role == "target" and not chain]
            label = "+".join(x for x, ok in (("src", src), ("tgt", tgt)) if ok) or "no"
            cells["coerce"] = cell("yes" if (src or tgt) else "no", label, [r.evidence() for r in src + tgt])
            # serde
            ser = [r for r in recs("serde", "self") + blankets["serde"] if r.trait == "RSerializeNative"]
            de = [r for r in recs("serde", "self") + blankets["serde"] if r.trait == "RDeserializeNative"]
            label = "+".join(x for x, ok in (("ser", ser), ("de", de)) if ok) or "no"
            cells["serde"] = cell(status_from_flags(bool(ser and de), bool(ser or de)), label, [r.evidence() for r in ser + de])
            # altrep
            written = recs("altrep", "self") + recs("altrep", "instance")
            blanket_alt = blankets["altrep"]
            if written:
                cells["altrep"] = cell("yes", "yes", [r.evidence() for r in written + blanket_alt])
            elif blanket_alt:
                cells["altrep"] = cell("partial", "blanket", [r.evidence() for r in blanket_alt])
            else:
                cells["altrep"] = cell("no", "no", [])
            # class
            typed = recs("class", "self") + recs("class", "instance") + blankets["class"]
            if typed:
                cells["class"] = cell("yes", "ptr", [r.evidence() for r in typed])
            else:
                # #[r_data] converts with TryFromSexp itself: a TryFromRoot view does not fit
                has_from = any(r.trait == "TryFromSexp" for r in recs("arg", "self") + blankets["arg"])
                has_into = any(r.trait == "IntoR" for r in recs("ret", "self") + blankets["ret"])
                has_clone = any(t[0] in STD_CRATES and t[1][-1] == "Clone" for t in own_traits)
                if has_from and has_into:
                    if name in refused_views:
                        cells["class"] = cell("partial", "sidecar", [
                            ev("#[r_data] refuses it as a plain field (unrooted R view); a Sidecar<T> field holds it", *refused_views[name]),
                        ])
                    elif has_clone:
                        cells["class"] = cell("partial", "field", [ev("#[r_data] field: TryFromSexp + IntoR + Clone")])
                    else:
                        cells["class"] = cell("partial", "sidecar", [ev("no Clone: only a Sidecar<T> #[r_data] field (TryFromSexp + IntoR)")])
                else:
                    cells["class"] = cell("no", "no", [])
        elif k == "trait":
            fp = crate.full_path(item["id"])
            tkey = (fp[0], tuple(fp[1])) if fp else None
            own_column = trait_column(fp[0], fp[1]) if fp else None
            for column in ("arg", "ret", "coerce", "serde", "altrep", "class"):
                if own_column == column:
                    cells[column] = cell("yes", "is", [ev(f"{name} defines the {CELL_LABELS[column]} column: every implementor has it")])
                    continue
                blanket_all, container = [], []
                for rec, param in bounded_on.get(tkey, []):
                    if rec.column != column:
                        continue
                    if rec.for_ir == ("gen", param):
                        blanket_all.append(rec)
                    else:
                        container.append(rec)
                if blanket_all:
                    cells[column] = cell("yes", "impls", [r.evidence() for r in blanket_all + container])
                elif container:
                    cells[column] = cell("partial", "bound", [r.evidence() for r in container])
                else:
                    cells[column] = cell("n/a", "–", [])
            cells["optvec"] = cell("n/a", "–", [])
        else:  # type alias
            alias_rows.append((row, crate, item))
            for column in ("arg", "ret", "optvec", "coerce", "serde", "altrep", "class"):
                cells[column] = cell("n/a", "–", [])

        # macro
        mev = scans.macros.evidence(name)
        cells["macro"] = cell("yes", str(len(mev)), mev) if mev else cell("no", "no", [])
        # quality
        cells["rustdoc"] = rustdoc_cell(crate, item)
        note = f"several public items are named {name}; mentions are not told apart" if name_count[name] > 1 else None
        sources = scans.sources
        cells["docs"] = hits_cell(sources, scans.docs_idents.get(name), "gap", note=note)
        if crate.label == "miniextendr-engine":
            why = [ev("miniextendr-engine embeds R in a Rust binary; an R package cannot use it")]
            cells["rpkg"] = cell("n/a", "–", why)
            cells["rtests"] = cell("n/a", "–", why)
        else:
            cells["rpkg"] = hits_cell(sources, scans.rpkg_idents.get(name), "gap", note=note)
            cells["rtests"] = rtests_cell(scans, rpkg_users.get(name, []))
        cells["rusttests"] = hits_cell(sources, scans.rust_test_idents.get(name), "gap", note=note)
        cells["crosspkg"] = hits_cell(sources, scans.cross_idents.get(name), "n/a", note=note)
        row_cells_by_key[key] = cells
        rows.append(row)

    # type aliases of a miniextendr type take its capability cells
    for row, crate, item in alias_rows:
        target = ir_from_rustdoc(item["inner"]["type_alias"].get("type"), crate)
        target_text = format_type(item["inner"]["type_alias"].get("type"), crate.index)
        if target[0] == "path" and target[3] in row_cells_by_key:
            src = row_cells_by_key[target[3]]
            for column in ("arg", "ret", "optvec", "coerce", "serde", "altrep", "class"):
                c = src[column]
                row["cells"][column] = cell(c["status"], c["label"], [ev(f"via the aliased type {target_text}")] + c["evidence"], sort=False)
        else:
            for column in ("arg", "ret", "optvec", "coerce", "serde", "altrep", "class"):
                row["cells"][column] = cell("n/a", "–", [ev(f"alias of {target_text}; see the converted tab")])

    rows.sort(key=lambda r: (r["info"]["crate"], r["path"]))
    return rows


def r_data_refusals(sources: Sources) -> dict:
    """Type names ``#[derive(ExternalPtr)]`` refuses as plain ``#[r_data]`` fields."""
    rel = "miniextendr-macros/src/externalptr_derive.rs"
    toks = sources.tokens(rel)
    out = {}
    n = len(toks)
    for i, tok in enumerate(toks):
        if tok[1] == "fn" and i + 1 < n and toks[i + 1][1] == "contains_unrooted_r_view":
            j = i
            # the first match arm whose body is `true`: string literals before `=> true`
            lits = []
            while j < n:
                if toks[j][0] == "str":
                    lits.append(toks[j])
                elif toks[j][1] == "=>":
                    if j + 1 < n and toks[j + 1][1] == "true" and lits:
                        for lit in lits:
                            out[lit[1]] = (rel, lit[2])
                        return out
                    lits = []
                j += 1
    return out


# endregion


# region: the converted tab


def converted_feature(recs: list[ImplRecord], feature_of_dep: dict) -> tuple[str, list[str]]:
    """Feature text of a converted row: 'always' when any impl is ungated."""
    texts = set()
    names = set()
    ungated = False
    for rec in recs:
        text = feature_text(rec.cfgs)
        feats = set().union(*[cfg_features(p) for p in rec.cfgs]) if rec.cfgs else set()
        if text == "always":
            dep = sorted({feature_of_dep[c] for _, c in constructor_names(rec.for_ir) if c in feature_of_dep})
            if dep:
                text = " & ".join(dep)
                feats = set(dep)
        if text == "always":
            ungated = True
        else:
            texts.add(text)
            names |= feats
    if ungated or not texts:
        return "always", []
    return " | ".join(sorted(texts)), sorted(names)


def generic_cover(recs: list[ImplRecord], column: str) -> dict:
    """yes when an impl covers the whole shape (only generic leaves), partial for instances only."""
    sel = [r for r in recs if r.column == column]
    if not sel:
        return cell("no", "no", [])
    instances = sorted({r.for_text for r in sel})
    lead = ev(f"{len(instances)} instance(s): " + ", ".join(instances[:8]) + (" …" if len(instances) > 8 else ""))
    covering = [r for r in sel if all(leaf[0] == "gen" for leaf in folded_leaves(r.for_ir))]
    rest = [r for r in sel if r not in covering]
    evidence = [lead] + sorted((r.evidence() for r in covering), key=ev_key) + sorted(
        (r.evidence() for r in rest), key=ev_key
    )
    full = bool(covering)
    return cell("yes" if full else "partial", "yes" if full else "inst", evidence, sort=False)


def build_converted_tab(api: Crate, impls: list[ImplRecord], scans: SourceScans, feature_of_dep: dict) -> list[dict]:
    api_impls = [r for r in impls if r.crate is api]
    groups = defaultdict(list)
    for rec in api_impls:
        if rec.column == "class" or rec.for_ir[0] == "gen" or ir_locals(rec.for_ir):
            continue
        groups[shape(rec.for_ir)].append(rec)

    class_by_shape = defaultdict(list)
    coerce_targets = defaultdict(list)
    for rec in api_impls:
        if rec.column == "class" and rec.for_ir[0] != "gen" and not ir_locals(rec.for_ir):
            class_by_shape[shape(rec.for_ir)].append(rec)
        if rec.column == "coerce":
            for t in rec.trait_args:
                if t[0] != "gen" and not ir_locals(t):
                    coerce_targets[shape(t)].append(rec)

    # every miniextendr trait a std / third-party type implements, for opt/vec bounds
    traits_of_shape = defaultdict(set)
    for item in api.index.values():
        if kind_of(item) != "impl":
            continue
        imp = item["inner"]["impl"]
        if not imp.get("trait") or imp.get("is_synthetic") or imp.get("blanket_impl"):
            continue
        fp = api.full_path(imp["trait"].get("id"))
        if fp is None:
            continue
        ir = ir_from_rustdoc(imp["for"], api)
        if ir[0] in ("prim", "path"):
            traits_of_shape[(shape(ir), top_crate(ir))].add((fp[0], tuple(fp[1])))

    rows = []
    for key in sorted(groups):
        recs = groups[key]
        ir0 = recs[0].for_ir
        match_key = shape(ir0, use_display=False)
        crate_name = top_crate(ir0)
        kind = shape_kind(ir0)
        feature, feature_names = converted_feature(recs, feature_of_dep)
        bounds = set()
        for rec in recs:
            for param, bs in rec.bounds.items():
                for b in bs:
                    if b[1][-1] != "Sized":
                        bounds.add(f"{param}: {b[1][-1]}")
        row = {
            "id": key,
            "name": key,
            "path": key,
            "info": {
                "origin": crate_name or "?",
                "kind": kind,
                "bounds": ", ".join(sorted(bounds)),
                "feature": feature,
            },
            "features": feature_names,
            "source": None,
            "reexports": [],
            "cells": {},
        }
        cells = row["cells"]

        cells["arg"] = generic_cover(recs, "arg")
        cells["ret"] = generic_cover(recs, "ret")
        if kind == "scalar":
            traits = traits_of_shape.get((key, crate_name), set())

            def matches(elem, _k=match_key, _c=crate_name):
                return shape(elem, use_display=False) == _k and top_crate(elem) == _c

            cells["optvec"] = optvec_cell(matches, traits, api_impls, foreign_ok=True)
        else:
            cells["optvec"] = cell("n/a", "–", [])
        src = [r for r in recs if r.column == "coerce"]
        tgt = coerce_targets.get(key, [])
        label = "+".join(x for x, ok in (("src", src), ("tgt", tgt)) if ok) or "no"
        cells["coerce"] = cell("yes" if (src or tgt) else "no", label, [r.evidence() for r in src + tgt])
        ser = [r for r in recs if r.trait == "RSerializeNative"]
        de = [r for r in recs if r.trait == "RDeserializeNative"]
        if ser or de:
            label = "+".join(x for x, ok in (("ser", ser), ("de", de)) if ok)
            cells["serde"] = cell(status_from_flags(bool(ser and de), True), label, [r.evidence() for r in ser + de])
        else:
            cells["serde"] = cell("n/a", "–", [ev("serde support comes from the type's own Serialize / Deserialize impls upstream")])
        alt = [r for r in recs if r.column == "altrep"]
        if alt:
            written = [r for r in alt if not r.blanket]
            cells["altrep"] = cell("yes" if written else "partial", "yes" if written else "blanket", [r.evidence() for r in alt])
        else:
            cells["altrep"] = cell("no", "no", [])
        cls = class_by_shape.get(key, [])
        cells["class"] = cell("yes", "ptr", [r.evidence() for r in cls]) if cls else cell("no", "no", [])
        # macro: names of the type and its constructors
        mev = []
        for nm, cr in constructor_names(ir0):
            mev += scans.macros.evidence(nm, cr if cr not in (None, "std") else None)
        cells["macro"] = cell("yes", str(len(mev)), mev) if mev else cell("no", "no", [])
        cells["rustdoc"] = cell("n/a", "–", [ev("std / third-party type: documented upstream")])
        sources = scans.sources
        cells["docs"] = hits_cell(sources, scans.docs_shapes.get(match_key, crate_name), "gap")
        rpkg_hits = scans.rpkg_sig_shapes.get(match_key, crate_name)
        cells["rpkg"] = hits_cell(sources, rpkg_hits, "gap", what="#[miniextendr] signature mentions")
        users = []
        seen = set()
        for rel, line, _ in rpkg_hits:
            idx = scans.rpkg_fn_by_hit.get((rel, line))
            if idx is not None and idx not in seen:
                seen.add(idx)
                users.append(scans.rpkg_items[idx])
        cells["rtests"] = rtests_cell(scans, users)
        cells["rusttests"] = hits_cell(sources, scans.rust_test_shapes.get(match_key, crate_name), "gap")
        cells["crosspkg"] = hits_cell(sources, scans.cross_shapes.get(match_key, crate_name), "n/a")
        rows.append(row)
    return rows


# endregion


# region: driver


def optional_dep_features(repo: Path) -> dict:
    """Crate name (``-`` as ``_``) -> the feature that enables it, for optional deps."""
    try:
        manifest = tomllib.loads((repo / "miniextendr-api" / "Cargo.toml").read_text())
    except (OSError, tomllib.TOMLDecodeError):
        return {}
    deps = manifest.get("dependencies", {})
    features = manifest.get("features", {})
    out = {}
    for dep, spec in deps.items():
        if not (isinstance(spec, dict) and spec.get("optional")):
            continue
        crate = (spec.get("package") or dep).replace("-", "_")
        feat = dep if dep in features else None
        for fname, members in features.items():
            if f"dep:{dep}" in members and (feat is None or fname == dep):
                feat = fname
        out[crate] = feat or dep
    return out


def git_meta(repo: Path) -> dict:
    def run(*args):
        try:
            return subprocess.run(["git", *args], cwd=repo, capture_output=True, text=True, check=True).stdout.strip()
        except (subprocess.CalledProcessError, FileNotFoundError):
            return ""

    sha = run("rev-parse", "HEAD")
    date = run("log", "-1", "--format=%cI")
    status = run("status", "--porcelain", "--untracked-files=no")
    return {"commit": sha or "unknown", "generated_at": date or "unknown", "dirty": bool(status)}


def summarize(rows: list[dict]) -> dict:
    out = {}
    for col in CELL_COLUMNS:
        counts = defaultdict(int)
        for r in rows:
            counts[r["cells"][col["id"]]["status"]] += 1
        out[col["id"]] = {s: counts[s] for s in STATUS_ORDER if counts[s]}
    return out


def build(api_json: Path, engine_json: Path | None, repo: Path, meta: dict | None = None) -> dict:
    api = Crate(json.loads(Path(api_json).read_text()), "miniextendr-api")
    crates = [api]
    if engine_json:
        crates.append(Crate(json.loads(Path(engine_json).read_text()), "miniextendr-engine"))
    sources = Sources(repo)
    scans = SourceScans(sources)
    impls = []
    for c in crates:
        impls.extend(collect_impls(c))
    feature_of_dep = optional_dep_features(repo)
    types_rows = build_types_tab(crates, impls, scans, feature_of_dep)
    converted_rows = build_converted_tab(api, impls, scans, feature_of_dep)
    meta = meta or git_meta(repo)
    return {
        "schema": SCHEMA_VERSION,
        "generated_at": meta["generated_at"],
        "commit": meta["commit"],
        "dirty": meta["dirty"],
        "repo_url": REPO_URL,
        "sources": {
            "rustdoc": [f"target/doc/{Path(api_json).name}"] + ([f"target/doc/{Path(engine_json).name}"] if engine_json else []),
            "format_version": api.data.get("format_version"),
        },
        "legend": LEGEND,
        "cell_columns": CELL_COLUMNS,
        "tabs": [
            {
                "id": "types",
                "title": "miniextendr types",
                "info_columns": TYPE_INFO_COLUMNS,
                "rows": types_rows,
                "summary": summarize(types_rows),
            },
            {
                "id": "converted",
                "title": "converted types",
                "info_columns": CONVERTED_INFO_COLUMNS,
                "rows": converted_rows,
                "summary": summarize(converted_rows),
            },
        ],
    }


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--api", required=True, help="rustdoc JSON of miniextendr-api (--features full)")
    ap.add_argument("--engine", help="rustdoc JSON of miniextendr-engine")
    ap.add_argument("--repo", default=str(Path(__file__).resolve().parent.parent), help="repository root")
    ap.add_argument("--out", required=True, help="output JSON path")
    ap.add_argument("--html", help="also render the interactive HTML page to this path")
    args = ap.parse_args(argv)
    data = build(Path(args.api), Path(args.engine) if args.engine else None, Path(args.repo))
    text = json.dumps(data, indent=1, ensure_ascii=False, sort_keys=False) + "\n"
    Path(args.out).write_text(text, encoding="utf-8")
    counts = ", ".join(f"{t['id']}: {len(t['rows'])} rows" for t in data["tabs"])
    print(f"Wrote {args.out} ({counts})")
    if args.html:
        from rustdoc_type_matrix_html import render_html

        Path(args.html).write_text(render_html(data), encoding="utf-8")
        print(f"Wrote {args.html}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
